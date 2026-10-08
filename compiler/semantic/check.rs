use super::*;
use crate::token::Kind;
/// Reachable exits from a statement; an empty set means divergence.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Flow(u8);
impl Flow {
    const NEXT: Self = Self(1);
    const RETURN: Self = Self(2);
    const BREAK: Self = Self(4);
    const CONTINUE: Self = Self(8);
    fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
    fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    fn then(self, other: Self) -> Self {
        if self.contains(Self::NEXT) {
            Self((self.0 & !Self::NEXT.0) | other.0)
        } else {
            self
        }
    }
    fn loop_exit(self, may_skip: bool) -> Self {
        Self(
            (self.0 & Self::RETURN.0)
                | if may_skip || self.contains(Self::BREAK) {
                    Self::NEXT.0
                } else {
                    0
                },
        )
    }
}
pub(super) struct FunctionContext {
    pub(super) module: ModuleId,
    pub(super) result: TypeId,
    pub(super) loops: usize,
}
impl Analyzer<'_> {
    pub(super) fn check_function(&mut self, id: SymbolId) {
        let m = self.result.symbols[id.0].module;
        let n = self.result.symbols[id.0].node;
        let NodeKind::Function {
            parameters, body, ..
        } = self.node(m, n)
        else {
            return;
        };
        let Some(sig) = self.result.signatures.get(&id).cloned() else {
            return;
        };
        let scope = self.child_scope(m, self.result.modules[m.0].scope);
        for (p, t) in parameters.into_iter().zip(sig.parameters) {
            if let NodeKind::Parameter { name, mutable, .. } = self.node(m, p) {
                let s = self.insert(m, scope, p, &name, SymbolKind::Parameter, false, mutable);
                self.result.symbols[s.0].ty = t;
                self.states[s.0] = 2;
            }
        }
        let mut ctx = FunctionContext {
            module: m,
            result: sig.result,
            loops: 0,
        };
        let flow = self.block(&mut ctx, scope, body, false, 0);
        if flow.contains(Flow::NEXT) && !self.is_unit(sig.result) {
            self.error(
                "T001",
                self.span(m, body),
                "not every reachable path returns a value or diverges",
            );
        }
    }
    fn child_scope(&mut self, m: ModuleId, parent: ScopeId) -> ScopeId {
        let id = ScopeId(self.result.scopes.len());
        self.result.scopes.push(Scope {
            parent: Some(parent),
            module: m,
            bindings: BTreeMap::new(),
        });
        id
    }
    fn unit(&mut self) -> TypeId {
        self.result.types.intern(Type::Tuple(vec![]))
    }
    fn is_unit(&self, t: TypeId) -> bool {
        matches!(self.result.types.get(t),Some(Type::Tuple(ts)) if ts.is_empty())
    }
    fn compatible(&mut self, m: ModuleId, n: NodeId, actual: TypeId, expected: TypeId) {
        if actual != expected && actual != Types::ERROR && expected != Types::ERROR {
            self.error(
                "T001",
                self.span(m, n),
                format!(
                    "expected {}, found {}",
                    self.type_name(expected, 0),
                    self.type_name(actual, 0)
                ),
            );
        }
    }
    fn type_name(&self, t: TypeId, depth: usize) -> String {
        if depth > 16 {
            return "…".into();
        }
        match self.result.types.get(t) {
            Some(Type::Integer { signed, bits }) => {
                format!("{}{bits}", if *signed { "i" } else { "u" })
            }
            Some(Type::Float(b)) => format!("f{b}"),
            Some(Type::Usize) => "usize".into(),
            Some(Type::Bool) => "bool".into(),
            Some(Type::Char) => "char".into(),
            Some(Type::Text) => "text".into(),
            Some(Type::Bytes) => "Bytes".into(),
            Some(Type::Tuple(ts)) => format!(
                "({})",
                ts.iter()
                    .map(|t| self.type_name(*t, depth + 1))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Some(Type::Builtin { name, arguments }) => format!(
                "{name}[{}]",
                arguments
                    .iter()
                    .map(|t| self.type_name(*t, depth + 1))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Some(Type::Reference { mutable, target }) => format!(
                "&{}{}",
                if *mutable { "mut " } else { "" },
                self.type_name(*target, depth + 1)
            ),
            Some(Type::Nominal(s)) => self.result.symbols[s.0].name.clone(),
            _ => "<error>".into(),
        }
    }
    fn block(
        &mut self,
        ctx: &mut FunctionContext,
        parent: ScopeId,
        n: NodeId,
        nested: bool,
        depth: usize,
    ) -> Flow {
        let m = ctx.module;
        if !self.tick(self.span(m, n), depth) {
            return Flow::NEXT;
        }
        let scope = if nested {
            self.child_scope(m, parent)
        } else {
            parent
        };
        let NodeKind::Block { statements } = self.node(m, n) else {
            return Flow::NEXT;
        };
        let mut flow = Flow::NEXT;
        for s in statements {
            let next = self.statement(ctx, scope, s, depth + 1);
            flow = flow.then(next);
        }
        flow
    }
    fn statement(
        &mut self,
        ctx: &mut FunctionContext,
        scope: ScopeId,
        n: NodeId,
        depth: usize,
    ) -> Flow {
        let m = ctx.module;
        if !self.tick(self.span(m, n), depth) {
            return Flow::NEXT;
        }
        match self.node(m, n) {
            NodeKind::Binding {
                mutable,
                name,
                ty,
                value,
            } => {
                let expected = ty.map(|t| self.resolve_type(m, scope, t, depth + 1));
                let actual = self.expression(ctx, scope, value, expected, depth + 1);
                let s = self.insert(m, scope, n, &name, SymbolKind::Local, false, mutable);
                self.result.symbols[s.0].ty = expected.unwrap_or(actual);
                self.states[s.0] = 2;
            }
            NodeKind::Constant { name, ty, value } => {
                let t = self.resolve_type(m, scope, ty, depth + 1);
                self.expression(ctx, scope, value, Some(t), depth + 1);
                let s = self.insert(m, scope, n, &name, SymbolKind::Constant, false, false);
                self.result.symbols[s.0].ty = t;
                self.states[s.0] = 2;
                self.check_constant(s, depth + 1);
            }
            NodeKind::Block { .. } => return self.block(ctx, scope, n, true, depth + 1),
            NodeKind::Return { value } => {
                let t = if let Some(v) = value {
                    self.expression(ctx, scope, v, Some(ctx.result), depth + 1)
                } else {
                    self.unit()
                };
                self.compatible(m, n, t, ctx.result);
                return Flow::RETURN;
            }
            NodeKind::Assignment { place, value } => {
                let t = self.expression(ctx, scope, place, None, depth + 1);
                if !self.writable(m, place) {
                    self.error(
                        "T001",
                        self.span(m, place),
                        "assignment requires a writable place",
                    );
                }
                self.expression(ctx, scope, value, Some(t), depth + 1);
            }
            NodeKind::ExpressionStatement { value } => {
                let t = self.expression(ctx, scope, value, None, depth + 1);
                let unit = self.unit();
                self.compatible(m, value, t, unit);
            }
            NodeKind::If {
                condition,
                then_block,
                otherwise,
            } => {
                let boolean = self.result.types.intern(Type::Bool);
                self.expression(ctx, scope, condition, Some(boolean), depth + 1);
                let a = self.block(ctx, scope, then_block, true, depth + 1);
                let b = otherwise
                    .map(|e| self.statement(ctx, scope, e, depth + 1))
                    .unwrap_or(Flow::NEXT);
                return match self.boolean_literal(m, condition) {
                    Some(true) => a,
                    Some(false) => b,
                    None => a.union(b),
                };
            }
            NodeKind::While { condition, body } => {
                let boolean = self.result.types.intern(Type::Bool);
                self.expression(ctx, scope, condition, Some(boolean), depth + 1);
                ctx.loops += 1;
                let body_flow = self.block(ctx, scope, body, true, depth + 1);
                ctx.loops -= 1;
                return match self.boolean_literal(m, condition) {
                    Some(false) => Flow::NEXT,
                    Some(true) => body_flow.loop_exit(false),
                    None => body_flow.loop_exit(true),
                };
            }
            NodeKind::Loop { body } => {
                ctx.loops += 1;
                let body_flow = self.block(ctx, scope, body, true, depth + 1);
                ctx.loops -= 1;
                return body_flow.loop_exit(false);
            }
            NodeKind::For {
                name,
                iterable,
                body,
            } => {
                let t = self.expression(ctx, scope, iterable, None, depth + 1);
                let (base, reference) = match self.result.types.get(t) {
                    Some(Type::Reference { target, mutable }) => (*target, Some(*mutable)),
                    _ => (t, None),
                };
                let element = match self.result.types.get(base).cloned() {
                    Some(Type::Builtin { name, arguments }) if name == "Seq" => arguments[0],
                    Some(Type::Bytes) => self.result.types.primitive("u8").unwrap(),
                    _ => {
                        self.error(
                            "T001",
                            self.span(m, iterable),
                            "for requires Seq, Bytes or a reference to either",
                        );
                        Types::ERROR
                    }
                };
                let element = if let Some(mutable) = reference {
                    self.result.types.intern(Type::Reference {
                        mutable,
                        target: element,
                    })
                } else {
                    element
                };
                let child = self.child_scope(m, scope);
                let s = self.insert(m, child, n, &name, SymbolKind::Local, false, false);
                self.result.symbols[s.0].ty = element;
                self.states[s.0] = 2;
                ctx.loops += 1;
                let body_flow = self.block(ctx, child, body, false, depth + 1);
                ctx.loops -= 1;
                return body_flow.loop_exit(true);
            }
            NodeKind::Break => {
                if ctx.loops == 0 {
                    self.error("T001", self.span(m, n), "break outside a loop");
                }
                return Flow::BREAK;
            }
            NodeKind::Continue => {
                if ctx.loops == 0 {
                    self.error("T001", self.span(m, n), "continue outside a loop");
                }
                return Flow::CONTINUE;
            }
            NodeKind::Match { subject, arms } => {
                let t = self.expression(ctx, scope, subject, None, depth + 1);
                let mut patterns = vec![];
                let mut flow = Flow(0);
                for arm in arms {
                    if let NodeKind::Arm { pattern, body } = self.node(m, arm) {
                        patterns.push(pattern);
                        let child = self.child_scope(m, scope);
                        self.pattern(ctx, child, pattern, t, depth + 1);
                        flow = flow.union(self.block(ctx, child, body, false, depth + 1));
                    }
                }
                self.match_coverage(m, n, t, &patterns);
                return if patterns.is_empty() {
                    Flow::NEXT
                } else {
                    flow
                };
            }
            NodeKind::Error => {}
            _ => self.error(
                "T001",
                self.span(m, n),
                "unexpected node in statement analysis",
            ),
        }
        Flow::NEXT
    }
    fn boolean_literal(&self, m: ModuleId, mut n: NodeId) -> Option<bool> {
        loop {
            match self.node(m, n) {
                NodeKind::Group { value } => n = value,
                NodeKind::Literal { kind: Kind::True } => return Some(true),
                NodeKind::Literal { kind: Kind::False } => return Some(false),
                _ => return None,
            }
        }
    }
    pub(super) fn integer(&self, t: TypeId) -> Option<(bool, u8)> {
        match self.result.types.get(t) {
            Some(Type::Integer { signed, bits }) => Some((*signed, *bits)),
            Some(Type::Usize) => Some((false, self.options.target_pointer_bits)),
            _ => None,
        }
    }
    fn number(&self, t: TypeId) -> bool {
        self.integer(t).is_some() || matches!(self.result.types.get(t), Some(Type::Float(_)))
    }
    pub(super) fn expression(
        &mut self,
        ctx: &mut FunctionContext,
        scope: ScopeId,
        n: NodeId,
        expected: Option<TypeId>,
        depth: usize,
    ) -> TypeId {
        let m = ctx.module;
        if !self.tick(self.span(m, n), depth) {
            return Types::ERROR;
        }
        let ty = match self.node(m, n) {
            NodeKind::Literal { kind } => self.literal(m, n, kind, expected, false),
            NodeKind::Path { components } => {
                let parts = self.path(m, &components);
                if let Some(s) = self.lookup(m, scope, &parts, self.span(m, n)) {
                    self.result.modules[m.0].resolutions.insert(n.0, s);
                    if matches!(
                        self.result.symbols[s.0].kind,
                        SymbolKind::Function
                            | SymbolKind::Record
                            | SymbolKind::Enum
                            | SymbolKind::Alias
                    ) {
                        self.error("T001", self.span(m, n), "name does not denote a value");
                        Types::ERROR
                    } else {
                        self.result.symbols[s.0].ty
                    }
                } else {
                    Types::ERROR
                }
            }
            NodeKind::Group { value } => self.expression(ctx, scope, value, expected, depth + 1),
            NodeKind::Tuple { elements } => {
                let es = match expected.and_then(|t| self.result.types.get(t)) {
                    Some(Type::Tuple(ts)) => ts.clone(),
                    _ => vec![],
                };
                let mut ts = vec![];
                for (i, e) in elements.into_iter().enumerate() {
                    ts.push(self.expression(ctx, scope, e, es.get(i).copied(), depth + 1));
                }
                self.result.types.intern(Type::Tuple(ts))
            }
            NodeKind::Sequence { elements } => {
                let mut element = match expected.and_then(|t| self.result.types.get(t)) {
                    Some(Type::Builtin { name, arguments }) if name == "Seq" => Some(arguments[0]),
                    _ => None,
                };
                for e in elements {
                    let t = self.expression(ctx, scope, e, element, depth + 1);
                    if element.is_none() {
                        element = Some(t);
                    }
                }
                if let Some(t) = element {
                    self.result.types.intern(Type::Builtin {
                        name: "Seq".into(),
                        arguments: vec![t],
                    })
                } else {
                    self.error(
                        "T001",
                        self.span(m, n),
                        "empty sequence requires an expected element type",
                    );
                    Types::ERROR
                }
            }
            NodeKind::Unary {
                operator,
                mutable,
                operand,
            } => {
                if operator == Kind::Minus
                    && matches!(self.node(m, operand), NodeKind::Literal { kind: Kind::Int })
                {
                    let t = self.literal(m, operand, Kind::Int, expected, true);
                    self.result.modules[m.0].node_types.insert(operand.0, t);
                    t
                } else {
                    let hint = if matches!(operator, Kind::Minus | Kind::Tilde | Kind::Bang) {
                        expected
                    } else {
                        None
                    };
                    let t = self.expression(ctx, scope, operand, hint, depth + 1);
                    match operator {
                        Kind::Amp => {
                            if !self.place(m, operand) || (mutable && !self.writable(m, operand)) {
                                self.error(
                                    "B002",
                                    self.span(m, n),
                                    "borrow requires an existing place with the requested access",
                                );
                            }
                            self.result
                                .types
                                .intern(Type::Reference { mutable, target: t })
                        }
                        Kind::Star => match self.result.types.get(t) {
                            Some(Type::Reference { target, .. }) => *target,
                            _ => {
                                self.error(
                                    "T001",
                                    self.span(m, n),
                                    "dereference requires a reference",
                                );
                                Types::ERROR
                            }
                        },
                        Kind::Bang => {
                            let b = self.result.types.intern(Type::Bool);
                            self.compatible(m, n, t, b);
                            b
                        }
                        Kind::Minus => {
                            if !matches!(
                                self.result.types.get(t),
                                Some(Type::Integer { signed: true, .. } | Type::Float(_))
                            ) {
                                self.error(
                                    "T001",
                                    self.span(m, n),
                                    "unary minus requires signed integer or float",
                                );
                            }
                            t
                        }
                        Kind::Tilde => {
                            if self.integer(t).is_none() {
                                self.error(
                                    "T001",
                                    self.span(m, n),
                                    "bitwise complement requires integer",
                                );
                            }
                            t
                        }
                        _ => Types::ERROR,
                    }
                }
            }
            NodeKind::Binary {
                operator,
                left,
                right,
            } => {
                let comparison = matches!(
                    operator,
                    Kind::EqEq | Kind::NotEq | Kind::Lt | Kind::Le | Kind::Gt | Kind::Ge
                );
                let hint = if comparison { None } else { expected };
                // If only the left operand is an untyped literal, the typed right operand supplies context.
                let (a, b) = if matches!(
                    self.node(m, left),
                    NodeKind::Literal {
                        kind: Kind::Int | Kind::Float
                    }
                ) && !matches!(self.node(m, right), NodeKind::Literal { .. })
                {
                    let b = self.expression(ctx, scope, right, hint, depth + 1);
                    let a = self.expression(ctx, scope, left, Some(b), depth + 1);
                    (a, b)
                } else {
                    let a = self.expression(ctx, scope, left, hint, depth + 1);
                    let b = self.expression(ctx, scope, right, Some(a), depth + 1);
                    (a, b)
                };
                self.compatible(m, n, b, a);
                let legal = match operator {
                    Kind::Plus | Kind::Minus | Kind::Star | Kind::Slash => self.number(a),
                    Kind::Percent
                    | Kind::Shl
                    | Kind::Shr
                    | Kind::Amp
                    | Kind::Pipe
                    | Kind::Caret => self.integer(a).is_some(),
                    Kind::AndAnd | Kind::OrOr => {
                        matches!(self.result.types.get(a), Some(Type::Bool))
                    }
                    Kind::Lt | Kind::Le | Kind::Gt | Kind::Ge => {
                        self.number(a)
                            || matches!(self.result.types.get(a), Some(Type::Char | Type::Text))
                    }
                    Kind::EqEq | Kind::NotEq => self.equality(a, 0),
                    _ => false,
                };
                if !legal && a != Types::ERROR {
                    self.error(
                        "T001",
                        self.span(m, n),
                        "operator is not defined for these operand types",
                    );
                }
                if comparison {
                    self.result.types.intern(Type::Bool)
                } else {
                    a
                }
            }
            NodeKind::Call { callee, arguments } => {
                self.call(ctx, scope, n, callee, arguments, expected, depth + 1)
            }
            NodeKind::Construct { path, fields } => {
                let parts = self.path(m, &path);
                if let Some(s) = self.lookup(m, scope, &parts, self.span(m, n)) {
                    let t = self.result.symbols[s.0].ty;
                    let nominal = match self.result.types.get(t) {
                        Some(Type::Nominal(id)) => *id,
                        _ => s,
                    };
                    if let Some(fs) = self.result.records.get(&nominal).cloned() {
                        let mut seen = BTreeSet::new();
                        for f in fields {
                            if let NodeKind::FieldValue { name, value } = self.node(m, f) {
                                let text = self.spelling(m, &name);
                                if !seen.insert(text.clone()) {
                                    self.error("T001", name.span, "duplicate construction field");
                                }
                                if let Some(field) = fs.iter().find(|f| f.name == text) {
                                    if !field.public && self.result.symbols[nominal.0].module != m {
                                        self.error("M001", name.span, "private construction field");
                                    }
                                    self.expression(ctx, scope, value, Some(field.ty), depth + 1);
                                } else {
                                    self.error("T001", name.span, "unknown construction field");
                                    self.expression(ctx, scope, value, None, depth + 1);
                                }
                            }
                        }
                        if seen.len() != fs.len() || fs.iter().any(|f| !seen.contains(&f.name)) {
                            self.error(
                                "T001",
                                self.span(m, n),
                                "record construction must supply every field exactly once",
                            );
                        }
                        t
                    } else {
                        self.error(
                            "T001",
                            self.span(m, n),
                            "construction requires a record type",
                        );
                        Types::ERROR
                    }
                } else {
                    Types::ERROR
                }
            }
            NodeKind::Member { receiver, name } => {
                let t = self.expression(ctx, scope, receiver, None, depth + 1);
                let text = self.spelling(m, &name);
                let field = match self.result.types.get(t) {
                    Some(Type::Nominal(s)) => self
                        .result
                        .records
                        .get(s)
                        .and_then(|fs| fs.iter().find(|f| f.name == text))
                        .map(|f| (*s, f.clone())),
                    _ => None,
                };
                if let Some((s, f)) = field {
                    if !f.public && self.result.symbols[s.0].module != m {
                        self.error("M001", name.span, "record field is private");
                    }
                    f.ty
                } else {
                    self.error("T001", name.span, "unknown field or non-record receiver");
                    Types::ERROR
                }
            }
            NodeKind::Index { receiver, index } => {
                let t = self.expression(ctx, scope, receiver, None, depth + 1);
                let usize = self.result.types.intern(Type::Usize);
                self.expression(ctx, scope, index, Some(usize), depth + 1);
                match self.result.types.get(t).cloned() {
                    Some(Type::Builtin { name, arguments }) if name == "Seq" => arguments[0],
                    Some(Type::Bytes) => self.result.types.primitive("u8").unwrap(),
                    _ => {
                        self.error("T001", self.span(m, n), "indexing requires Seq or Bytes");
                        Types::ERROR
                    }
                }
            }
            NodeKind::Propagate { value } => {
                let t = self.expression(ctx, scope, value, None, depth + 1);
                match (
                    self.result.types.get(t).cloned(),
                    self.result.types.get(ctx.result).cloned(),
                ) {
                    (
                        Some(Type::Builtin { name, arguments }),
                        Some(Type::Builtin {
                            name: out,
                            arguments: outer,
                        }),
                    ) if name == "Result" && out == "Result" => {
                        self.compatible(m, n, arguments[1], outer[1]);
                        arguments[0]
                    }
                    _ => {
                        self.error("T001",self.span(m,n),"propagation requires Result and a Result-returning function with the same error type");
                        Types::ERROR
                    }
                }
            }
            NodeKind::Error => Types::ERROR,
            _ => {
                self.error("T001", self.span(m, n), "unexpected expression node");
                Types::ERROR
            }
        };
        if let Some(e) = expected {
            self.compatible(m, n, ty, e);
        }
        self.result.modules[m.0].node_types.insert(n.0, ty);
        let category = if !self.place(m, n) {
            ValueCategory::Value
        } else if self.writable(m, n) {
            ValueCategory::WritablePlace
        } else {
            ValueCategory::ReadOnlyPlace
        };
        self.result.modules[m.0]
            .value_categories
            .insert(n.0, category);
        ty
    }
    fn literal(
        &mut self,
        m: ModuleId,
        n: NodeId,
        kind: Kind,
        expected: Option<TypeId>,
        negative: bool,
    ) -> TypeId {
        match kind {
            Kind::Int => {
                let t = expected
                    .filter(|t| self.integer(*t).is_some())
                    .unwrap_or_else(|| self.result.types.primitive("i64").unwrap());
                let (signed, bits) = self.integer(t).unwrap();
                let raw = self.sources[m.0]
                    .slice(self.span(m, n))
                    .unwrap_or("")
                    .replace('_', "");
                let (radix, digits) = if let Some(digits) = raw.strip_prefix("0x") {
                    (16, digits)
                } else if let Some(digits) = raw.strip_prefix("0b") {
                    (2, digits)
                } else if let Some(digits) = raw.strip_prefix("0o") {
                    (8, digits)
                } else {
                    (10, raw.as_str())
                };
                let maximum = if signed {
                    (1u128 << (bits - 1)) - u128::from(!negative)
                } else {
                    (1u128 << bits) - 1
                };
                if negative && !signed
                    || u128::from_str_radix(digits, radix).map_or(true, |v| v > maximum)
                {
                    self.error(
                        "T001",
                        self.span(m, n),
                        "integer literal is outside its contextual type range",
                    );
                }
                t
            }
            Kind::Float => {
                let t = expected
                    .filter(|t| matches!(self.result.types.get(*t), Some(Type::Float(_))))
                    .unwrap_or_else(|| self.result.types.primitive("f64").unwrap());
                let raw = self.sources[m.0]
                    .slice(self.span(m, n))
                    .unwrap_or("")
                    .replace('_', "");
                let finite = if matches!(self.result.types.get(t), Some(Type::Float(32))) {
                    raw.parse::<f32>().is_ok_and(|v| v.is_finite())
                } else {
                    raw.parse::<f64>().is_ok_and(|v| v.is_finite())
                };
                if !finite {
                    self.error(
                        "T001",
                        self.span(m, n),
                        "floating literal overflows its contextual type",
                    );
                }
                t
            }
            Kind::True | Kind::False => self.result.types.intern(Type::Bool),
            Kind::String => self.result.types.intern(Type::Text),
            Kind::Char => self.result.types.intern(Type::Char),
            Kind::Bytes => self.result.types.intern(Type::Bytes),
            _ => Types::ERROR,
        }
    }
    // Call syntax, context and work depth remain separate from symbol metadata.
    #[allow(clippy::too_many_arguments)]
    fn call(
        &mut self,
        ctx: &mut FunctionContext,
        scope: ScopeId,
        n: NodeId,
        callee: NodeId,
        args: Vec<NodeId>,
        expected: Option<TypeId>,
        depth: usize,
    ) -> TypeId {
        let m = ctx.module;
        let NodeKind::Path { components } = self.node(m, callee) else {
            self.expression(ctx, scope, callee, None, depth + 1);
            self.error(
                "T001",
                self.span(m, callee),
                "only direct named functions and constructors are callable",
            );
            return Types::ERROR;
        };
        let parts = self.path(m, &components);
        if parts == ["discard"] {
            if args.len() != 2 {
                self.error(
                    "T001",
                    self.span(m, n),
                    "discard requires a value and a non-empty literal reason",
                );
            }
            for (i, arg) in args.iter().enumerate() {
                self.expression(ctx, scope, *arg, None, depth + 1);
                if i == 1
                    && (!matches!(self.node(m, *arg), NodeKind::Literal { kind: Kind::String })
                        || self.sources[m.0].slice(self.span(m, *arg)) == Some("\"\""))
                {
                    self.error(
                        "T001",
                        self.span(m, *arg),
                        "discard reason must be a non-empty string literal",
                    );
                }
            }
            return self.unit();
        }
        if parts.len() == 2 && (parts[0] == "Option" || parts[0] == "Result") {
            let name = parts[0].clone();
            let variant = parts[1].as_str();
            let count = if name == "Option" { 1 } else { 2 };
            let mut ts = match expected.and_then(|t| self.result.types.get(t)) {
                Some(Type::Builtin { name: n, arguments }) if n == &name => arguments.clone(),
                _ => vec![Types::ERROR; count],
            };
            let index = match (name.as_str(), variant) {
                ("Option", "Some") | ("Result", "Ok") => Some(0),
                ("Result", "Err") => Some(1),
                ("Option", "None") => None,
                _ => {
                    self.error("T001", self.span(m, n), "unknown built-in variant");
                    return Types::ERROR;
                }
            };
            if args.len() != usize::from(index.is_some()) {
                self.error("T001", self.span(m, n), "wrong constructor argument count");
            }
            for arg in args {
                let hint = index.and_then(|i| (ts[i] != Types::ERROR).then_some(ts[i]));
                let t = self.expression(ctx, scope, arg, hint, depth + 1);
                if let Some(i) = index {
                    ts[i] = t;
                }
            }
            if ts.contains(&Types::ERROR) {
                self.error(
                    "T001",
                    self.span(m, n),
                    "constructor requires context to infer all type arguments",
                );
            }
            return self.result.types.intern(Type::Builtin {
                name,
                arguments: ts,
            });
        }
        // Qualified enum construction has one more path component than its type.
        if parts.len() >= 2 {
            let type_parts = &parts[..parts.len() - 1];
            let type_symbol = if type_parts.len() == 1 {
                self.result.scopes[self.result.modules[m.0].scope.0]
                    .bindings
                    .get(&type_parts[0])
                    .copied()
            } else {
                self.result.modules[m.0]
                    .imports
                    .get(&type_parts[0])
                    .and_then(|target| {
                        self.result.scopes[self.result.modules[target.0].scope.0]
                            .bindings
                            .get(&type_parts[1])
                    })
                    .copied()
            };
            if let Some(s) = type_symbol {
                let t = self.result.symbols[s.0].ty;
                if let Some(Type::Nominal(e)) = self.result.types.get(t) {
                    if let Some(vs) = self.result.enums.get(e) {
                        let v = vs
                            .iter()
                            .find(|v| v.name == *parts.last().unwrap())
                            .cloned();
                        if !self.result.symbols[s.0].public && self.result.symbols[s.0].module != m
                        {
                            self.error("M001", self.span(m, n), "enum type is private");
                        }
                        if let Some(v) = v {
                            if v.payload.len() != args.len() {
                                self.error("T001", self.span(m, n), "wrong variant payload count");
                            }
                            for (i, arg) in args.into_iter().enumerate() {
                                self.expression(
                                    ctx,
                                    scope,
                                    arg,
                                    v.payload.get(i).copied(),
                                    depth + 1,
                                );
                            }
                            return t;
                        }
                        self.error("T001", self.span(m, n), "unknown enum variant");
                        return Types::ERROR;
                    }
                }
            }
        }
        let Some(s) = self.lookup(m, scope, &parts, self.span(m, callee)) else {
            return Types::ERROR;
        };
        self.result.modules[m.0].resolutions.insert(callee.0, s);
        if let Some(sig) = self.result.signatures.get(&s).cloned() {
            if args.len() != sig.parameters.len() {
                self.error(
                    "T001",
                    self.span(m, n),
                    format!(
                        "expected {} arguments, found {}",
                        sig.parameters.len(),
                        args.len()
                    ),
                );
            }
            for (i, arg) in args.into_iter().enumerate() {
                self.expression(ctx, scope, arg, sig.parameters.get(i).copied(), depth + 1);
            }
            sig.result
        } else {
            self.error(
                "T001",
                self.span(m, callee),
                "resolved name is not callable",
            );
            Types::ERROR
        }
    }
    fn pattern(
        &mut self,
        ctx: &mut FunctionContext,
        scope: ScopeId,
        n: NodeId,
        ty: TypeId,
        depth: usize,
    ) {
        let m = ctx.module;
        if !self.tick(self.span(m, n), depth) {
            return;
        }
        let (base, reference) = match self.result.types.get(ty) {
            Some(Type::Reference { mutable, target }) => (*target, Some(*mutable)),
            _ => (ty, None),
        };
        match self.node(m, n) {
            NodeKind::WildcardPattern => {}
            NodeKind::BindingPattern { name } => {
                let s = self.insert(m, scope, n, &name, SymbolKind::Pattern, false, false);
                self.result.symbols[s.0].ty = ty;
                self.states[s.0] = 2;
            }
            NodeKind::LiteralPattern { kind } => {
                if kind == Kind::Float {
                    self.error("T001", self.span(m, n), "floating patterns are excluded");
                }
                let t = self.literal(m, n, kind, Some(base), false);
                self.compatible(m, n, t, base);
            }
            NodeKind::TuplePattern { fields } => {
                if let Some(Type::Tuple(ts)) = self.result.types.get(base).cloned() {
                    if fields.len() != ts.len() {
                        self.error("T001", self.span(m, n), "tuple pattern arity mismatch");
                    }
                    for (f, t) in fields.into_iter().zip(ts) {
                        let t = if let Some(mutable) = reference {
                            self.result
                                .types
                                .intern(Type::Reference { mutable, target: t })
                        } else {
                            t
                        };
                        self.pattern(ctx, scope, f, t, depth + 1);
                    }
                } else {
                    self.error(
                        "T001",
                        self.span(m, n),
                        "tuple pattern requires tuple subject",
                    );
                }
            }
            NodeKind::ConstructorPattern { path, fields } => {
                let parts = self.path(m, &path);
                let name = parts.last().cloned().unwrap_or_default();
                if matches!(self.result.types.get(base), Some(Type::Nominal(_))) {
                    if parts.len() < 2 {
                        self.error(
                            "T001",
                            self.span(m, n),
                            "variant pattern requires a qualified type",
                        );
                    } else if let Some(s) =
                        self.lookup(m, scope, &parts[..parts.len() - 1], self.span(m, n))
                    {
                        self.compatible(m, n, self.result.symbols[s.0].ty, base);
                    }
                }
                let payload = match self.result.types.get(base).cloned() {
                    Some(Type::Nominal(s)) => self
                        .result
                        .enums
                        .get(&s)
                        .and_then(|vs| vs.iter().find(|v| v.name == name))
                        .map(|v| v.payload.clone()),
                    Some(Type::Builtin {
                        name: family,
                        arguments,
                    }) if parts.first() == Some(&family) => {
                        match (family.as_str(), name.as_str()) {
                            ("Option", "Some") | ("Result", "Ok") => Some(vec![arguments[0]]),
                            ("Option", "None") => Some(vec![]),
                            ("Result", "Err") => Some(vec![arguments[1]]),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                if let Some(ts) = payload {
                    if fields.len() != ts.len() {
                        self.error("T001", self.span(m, n), "variant pattern arity mismatch");
                    }
                    for (f, t) in fields.into_iter().zip(ts) {
                        let t = if let Some(mutable) = reference {
                            self.result
                                .types
                                .intern(Type::Reference { mutable, target: t })
                        } else {
                            t
                        };
                        self.pattern(ctx, scope, f, t, depth + 1);
                    }
                } else {
                    self.error(
                        "T001",
                        self.span(m, n),
                        "pattern does not name a variant of the subject type",
                    );
                }
            }
            _ => self.error("T001", self.span(m, n), "invalid pattern"),
        }
        self.result.modules[m.0].node_types.insert(n.0, ty);
    }
    fn place(&self, m: ModuleId, mut n: NodeId) -> bool {
        loop {
            match self.node(m, n) {
                NodeKind::Path { .. } => {
                    return self.result.modules[m.0]
                        .resolutions
                        .get(&n.0)
                        .is_some_and(|s| {
                            matches!(
                                self.result.symbols[s.0].kind,
                                SymbolKind::Local | SymbolKind::Parameter | SymbolKind::Pattern
                            )
                        })
                }
                NodeKind::Group { value } => n = value,
                NodeKind::Member { receiver, .. } | NodeKind::Index { receiver, .. } => {
                    n = receiver
                }
                NodeKind::Unary {
                    operator: Kind::Star,
                    ..
                } => return true,
                _ => return false,
            }
        }
    }
    fn writable(&self, m: ModuleId, mut n: NodeId) -> bool {
        loop {
            match self.node(m, n) {
                NodeKind::Path { .. } => {
                    return self.result.modules[m.0]
                        .resolutions
                        .get(&n.0)
                        .is_some_and(|s| self.result.symbols[s.0].mutable)
                }
                NodeKind::Group { value } => n = value,
                NodeKind::Member { receiver, .. } | NodeKind::Index { receiver, .. } => {
                    n = receiver
                }
                NodeKind::Unary {
                    operator: Kind::Star,
                    operand,
                    ..
                } => {
                    return self.result.modules[m.0]
                        .node_types
                        .get(&operand.0)
                        .is_some_and(|t| {
                            matches!(
                                self.result.types.get(*t),
                                Some(Type::Reference { mutable: true, .. })
                            )
                        })
                }
                _ => return false,
            }
        }
    }
    fn equality(&self, root: TypeId, _depth: usize) -> bool {
        // Shared type subgraphs are visited once instead of expanding every field path.
        let mut stack = vec![(root, false)];
        let mut seen = BTreeSet::new();
        while let Some((t, requires_copy)) = stack.pop() {
            if !seen.insert((t, requires_copy)) {
                continue;
            }
            match self.result.types.get(t) {
                Some(
                    Type::Bool | Type::Char | Type::Integer { .. } | Type::Usize | Type::Float(_),
                ) => {}
                Some(Type::Text | Type::Bytes) if !requires_copy => {}
                Some(Type::Tuple(ts)) => stack.extend(ts.iter().map(|t| (*t, true))),
                Some(Type::Builtin { name, arguments }) if name == "Option" || name == "Result" => {
                    stack.extend(arguments.iter().map(|t| (*t, true)))
                }
                Some(Type::Nominal(s)) => {
                    if let Some(fs) = self.result.records.get(s) {
                        stack.extend(fs.iter().map(|f| (f.ty, true)));
                    } else if let Some(vs) = self.result.enums.get(s) {
                        for v in vs {
                            stack.extend(v.payload.iter().map(|t| (*t, true)));
                        }
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        true
    }
}
