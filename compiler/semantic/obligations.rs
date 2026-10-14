//! May-be-unhandled Result values, tracked independently of type inference.
use super::*;
use crate::token::Kind;
use std::{cell::Cell, rc::Rc};
// A shape records obligations actually present in a constructed value. Unknown
// parameters/call results are expanded lazily using their projected type.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Part {
    Field(String),
    Tuple(usize),
    Payload(String, usize),
    Element(usize),
    Result,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Value {
    unknown: bool,
    height: usize,
    parts: BTreeMap<Part, Value>,
}
impl Value {
    fn unknown() -> Self {
        Self {
            unknown: true,
            height: 0,
            parts: BTreeMap::new(),
        }
    }
    fn cost(&self) -> usize {
        let mut cost = 1usize;
        let mut stack = vec![self];
        while let Some(value) = stack.pop() {
            for (part, child) in &value.parts {
                cost = cost.saturating_add(1).saturating_add(match part {
                    Part::Field(name) | Part::Payload(name, _) => name.len(),
                    _ => 0,
                });
                stack.push(child);
            }
        }
        cost
    }
    fn owns(&self) -> bool {
        self.unknown || !self.parts.is_empty()
    }
    fn insert(&mut self, key: Part, mut value: Value) {
        if value.owns() {
            // Recursive containers can grow at loop back edges without growing
            // the source AST. Widen deep projections to a may-own value so the
            // abstract domain and recursive clone/drop depth remain bounded.
            if value.height >= 64 {
                value = Self::unknown();
            }
            let child = self.parts.entry(key).or_default();
            child.merge(value);
            self.height = self.height.max(child.height + 1);
        }
    }
    fn merge(&mut self, other: Value) {
        if self.unknown {
            return;
        }
        if other.unknown {
            *self = other;
            return;
        }
        for (key, value) in other.parts {
            self.insert(key, value);
        }
    }
}
type Pending = BTreeMap<SymbolId, Value>;
#[derive(Default)]
struct Exits {
    next: Option<Pending>,
    breaks: Option<Pending>,
    continues: Option<Pending>,
}
fn join(a: &mut Option<Pending>, b: Option<Pending>) {
    if let Some(b) = b {
        let a = a.get_or_insert_with(Pending::new);
        for (s, value) in b {
            a.entry(s).or_default().merge(value);
        }
    }
}
struct Context {
    module: ModuleId,
    declarations: Rc<BTreeMap<usize, SymbolId>>,
    temporaries: Cell<usize>,
    remaining_iterators: Vec<Span>,
}
impl Analyzer<'_> {
    pub(super) fn check_result_obligations(&mut self, function: SymbolId) {
        let m = self.result.symbols[function.0].module;
        let node = self.result.symbols[function.0].node;
        let span = self.span(m, node);
        let NodeKind::Function {
            parameters, body, ..
        } = self.node(m, node)
        else {
            return;
        };
        self.work = self.work.saturating_add(self.result.symbols.len());
        if !self.tick(span, 0) {
            return;
        }
        let declarations = self
            .result
            .symbols
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                s.module == m
                    && s.span.start >= span.start
                    && s.span.end <= span.end
                    && matches!(
                        s.kind,
                        SymbolKind::Parameter | SymbolKind::Local | SymbolKind::Pattern
                    )
            })
            .map(|(i, s)| (s.node.0, SymbolId(i)))
            .collect();
        let c = Context {
            module: m,
            declarations: Rc::new(declarations),
            temporaries: Cell::new(0),
            remaining_iterators: vec![],
        };
        let mut pending = Pending::new();
        for p in parameters {
            if let Some(s) = c.declarations.get(&p.0) {
                if self.owns_result(self.result.symbols[s.0].ty, span) {
                    pending.insert(*s, Value::unknown());
                }
            }
        }
        let exits = self.obligation_statement(&c, body, pending, 0);
        if let Some(pending) = exits.next {
            self.unhandled(m, body, &pending);
        }
    }
    fn owns_result(&mut self, ty: TypeId, span: Span) -> bool {
        let mut seen = BTreeSet::new();
        let mut stack = vec![ty];
        while let Some(t) = stack.pop() {
            if !self.tick(span, 0) {
                return false;
            }
            if !seen.insert(t) {
                continue;
            }
            match self.result.types.get(t) {
                Some(Type::Builtin { name, .. }) if name == "Result" => return true,
                Some(Type::Builtin { arguments, .. }) | Some(Type::Tuple(arguments)) => {
                    stack.extend(arguments)
                }
                Some(Type::Nominal(s)) => {
                    if let Some(fs) = self.result.records.get(s) {
                        stack.extend(fs.iter().map(|f| f.ty));
                    }
                    if let Some(vs) = self.result.enums.get(s) {
                        for v in vs {
                            stack.extend(&v.payload);
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }
    fn obligation_type(&self, m: ModuleId, n: NodeId) -> TypeId {
        self.result.modules[m.0]
            .node_types
            .get(&n.0)
            .copied()
            .unwrap_or(Types::ERROR)
    }
    fn unhandled(&mut self, m: ModuleId, n: NodeId, pending: &Pending) {
        for s in pending.keys() {
            self.error(
                "T003",
                self.span(m, n),
                format!(
                    "Result obligation owned by `{}` is not handled before this exit",
                    self.result.symbols[s.0].name
                ),
            );
        }
    }
    fn unknown_obligation(&mut self, ty: TypeId, span: Span) -> Value {
        if self.owns_result(ty, span) {
            Value::unknown()
        } else {
            Value::default()
        }
    }
    fn project_obligation(&mut self, value: &Value, part: Part, ty: TypeId, span: Span) -> Value {
        self.work = self.work.saturating_add(value.cost());
        if !self.tick(span, 0) {
            return Value::default();
        }
        if value.unknown {
            self.unknown_obligation(ty, span)
        } else if part == Part::Element(usize::MAX) {
            let mut result = Value::default();
            for (key, value) in &value.parts {
                if matches!(key, Part::Element(_)) {
                    result.merge(value.clone());
                }
            }
            result
        } else {
            value.parts.get(&part).cloned().unwrap_or_default()
        }
    }
    fn obligation_expression(
        &mut self,
        c: &Context,
        n: NodeId,
        pending: &mut Pending,
        consume: bool,
        depth: usize,
    ) -> Value {
        let m = c.module;
        if !self.tick(self.span(m, n), depth) {
            return Value::default();
        }
        match self.node(m, n) {
            NodeKind::Path { .. } => {
                let symbol = self.result.modules[m.0].resolutions.get(&n.0).copied();
                if let Some(value) = symbol.and_then(|s| pending.get(&s)) {
                    self.work = self.work.saturating_add(value.cost());
                    if !self.tick(self.span(m, n), depth) {
                        return Value::default();
                    }
                }
                symbol
                    .and_then(|s| {
                        if consume {
                            pending.remove(&s)
                        } else {
                            pending.get(&s).cloned()
                        }
                    })
                    .unwrap_or_default()
            }
            NodeKind::Group { value } => {
                self.obligation_expression(c, value, pending, consume, depth + 1)
            }
            NodeKind::Tuple { elements } => {
                let saved = c.temporaries.get();
                let mut result = Value::default();
                for (i, value) in elements.into_iter().enumerate() {
                    result.insert(
                        Part::Tuple(i),
                        self.obligation_expression(c, value, pending, consume, depth + 1),
                    );
                    c.temporaries.set(saved + usize::from(result.owns()));
                }
                c.temporaries.set(saved);
                result
            }
            NodeKind::Sequence { elements } => {
                let saved = c.temporaries.get();
                let mut result = Value::default();
                for (i, value) in elements.into_iter().enumerate() {
                    result.insert(
                        Part::Element(i),
                        self.obligation_expression(c, value, pending, consume, depth + 1),
                    );
                    c.temporaries.set(saved + usize::from(result.owns()));
                }
                c.temporaries.set(saved);
                result
            }
            NodeKind::Construct { fields, .. } => {
                let saved = c.temporaries.get();
                let mut result = Value::default();
                for field in fields {
                    if let NodeKind::FieldValue { name, value } = self.node(m, field) {
                        result.insert(
                            Part::Field(self.spelling(m, &name)),
                            self.obligation_expression(c, value, pending, consume, depth + 1),
                        );
                        c.temporaries.set(saved + usize::from(result.owns()));
                    }
                }
                c.temporaries.set(saved);
                result
            }
            NodeKind::Call { callee, arguments } => {
                let saved = c.temporaries.get();
                let mut values = vec![];
                for arg in arguments {
                    let value = self.obligation_expression(c, arg, pending, true, depth + 1);
                    if value.owns() {
                        c.temporaries.set(saved + 1);
                    }
                    values.push(value);
                }
                c.temporaries.set(saved);
                if let NodeKind::Path { components } = self.node(m, callee) {
                    let parts = self.path(m, &components);
                    if parts == ["discard"] {
                        return Value::default();
                    }
                    if !self.result.modules[m.0].resolutions.contains_key(&callee.0) {
                        let mut result = Value::default();
                        if parts.first().is_some_and(|p| p == "Result") {
                            result.insert(Part::Result, Value::unknown());
                        }
                        let variant = parts.last().cloned().unwrap_or_default();
                        for (i, value) in values.into_iter().enumerate() {
                            result.insert(Part::Payload(variant.clone(), i), value);
                        }
                        return result;
                    }
                }
                self.unknown_obligation(self.obligation_type(m, n), self.span(m, n))
            }
            NodeKind::Propagate { value } => {
                let shape = self.obligation_expression(c, value, pending, true, depth + 1);
                self.unhandled(m, n, pending);
                self.unhandled_iterators(c, self.span(m, n));
                if c.temporaries.get() > 0 {
                    self.error("T003", self.span(m, n), "propagation may drop an unhandled Result in an earlier argument or aggregate element");
                }
                self.project_obligation(
                    &shape,
                    Part::Payload("Ok".into(), 0),
                    self.obligation_type(m, n),
                    self.span(m, n),
                )
            }
            NodeKind::Unary { operand, .. } => {
                self.obligation_expression(c, operand, pending, false, depth + 1);
                Value::default()
            }
            NodeKind::Binary {
                operator,
                left,
                right,
            } => {
                self.obligation_expression(c, left, pending, false, depth + 1);
                if matches!(operator, Kind::AndAnd | Kind::OrOr) {
                    // Only join a skipped RHS when that execution path exists.
                    // Type checking still validates both operands independently.
                    let left_value = self.obligation_boolean(m, left);
                    let run_on = operator == Kind::AndAnd;
                    if left_value == Some(run_on) {
                        self.obligation_expression(c, right, pending, false, depth + 1);
                    } else if left_value.is_none() {
                        let before = pending.clone();
                        self.obligation_expression(c, right, pending, false, depth + 1);
                        for (s, v) in before {
                            pending.entry(s).or_default().merge(v);
                        }
                    }
                } else {
                    self.obligation_expression(c, right, pending, false, depth + 1);
                }
                Value::default()
            }
            NodeKind::Member { receiver, name } => {
                let shape = self.obligation_expression(c, receiver, pending, false, depth + 1);
                self.project_obligation(
                    &shape,
                    Part::Field(self.spelling(m, &name)),
                    self.obligation_type(m, n),
                    self.span(m, n),
                )
            }
            NodeKind::Index { receiver, index } => {
                let shape = self.obligation_expression(c, receiver, pending, false, depth + 1);
                self.obligation_expression(c, index, pending, false, depth + 1);
                self.project_obligation(
                    &shape,
                    Part::Element(usize::MAX),
                    self.obligation_type(m, n),
                    self.span(m, n),
                )
            }
            _ => Value::default(),
        }
    }
    fn obligation_place(
        &mut self,
        c: &Context,
        n: NodeId,
        pending: &mut Pending,
        depth: usize,
    ) -> Option<(SymbolId, Vec<(Part, TypeId)>)> {
        let m = c.module;
        if !self.tick(self.span(m, n), depth) {
            return None;
        }
        match self.node(m, n) {
            NodeKind::Path { .. } => self.result.modules[m.0]
                .resolutions
                .get(&n.0)
                .copied()
                .map(|s| (s, vec![])),
            NodeKind::Group { value } => self.obligation_place(c, value, pending, depth + 1),
            NodeKind::Member { receiver, name } => {
                let mut place = self.obligation_place(c, receiver, pending, depth + 1)?;
                place.1.push((
                    Part::Field(self.spelling(m, &name)),
                    self.obligation_type(m, n),
                ));
                Some(place)
            }
            NodeKind::Index { receiver, index } => {
                let place = self.obligation_place(c, receiver, pending, depth + 1);
                self.obligation_expression(c, index, pending, false, depth + 1);
                let offset = match self.node(m, index) {
                    NodeKind::Literal { kind: Kind::Int } => self.sources[m.0]
                        .slice(self.span(m, index))
                        .and_then(|s| s.replace('_', "").parse::<usize>().ok())
                        .unwrap_or(usize::MAX),
                    _ => usize::MAX,
                };
                place.map(|mut p| {
                    p.1.push((Part::Element(offset), self.obligation_type(m, n)));
                    p
                })
            }
            _ => {
                self.obligation_expression(c, n, pending, false, depth + 1);
                None
            }
        }
    }
    fn expand_obligation(&mut self, ty: TypeId, span: Span) -> Value {
        let mut shape = Value::default();
        match self.result.types.get(ty).cloned() {
            Some(Type::Tuple(types)) => {
                for (i, ty) in types.into_iter().enumerate() {
                    shape.insert(Part::Tuple(i), self.unknown_obligation(ty, span));
                }
            }
            Some(Type::Nominal(s)) => {
                if let Some(fields) = self.result.records.get(&s).cloned() {
                    for f in fields {
                        shape.insert(Part::Field(f.name), self.unknown_obligation(f.ty, span));
                    }
                } else {
                    return Value::unknown();
                }
            }
            Some(Type::Builtin { name, arguments }) if name == "Seq" => {
                shape.insert(
                    Part::Element(usize::MAX),
                    self.unknown_obligation(arguments[0], span),
                );
            }
            _ => return Value::unknown(),
        }
        shape
    }
    fn replace_obligation(
        &mut self,
        shape: &mut Value,
        ty: TypeId,
        path: &[(Part, TypeId)],
        value: Value,
        span: Span,
        depth: usize,
    ) -> bool {
        if !self.tick(span, depth) {
            return false;
        }
        let Some(((key, child_ty), rest)) = path.split_first() else {
            let dropped = shape.owns();
            *shape = value;
            return dropped;
        };
        if shape.unknown {
            *shape = self.expand_obligation(ty, span);
        }
        // An unresolved/indirect place must not silently erase a potential obligation.
        if shape.unknown {
            return true;
        }
        if matches!(key, Part::Element(_))
            && (shape.parts.contains_key(&Part::Element(usize::MAX))
                || *key == Part::Element(usize::MAX))
        {
            let mut possible =
                self.project_obligation(shape, Part::Element(usize::MAX), *child_ty, span);
            let dropped =
                self.replace_obligation(&mut possible, *child_ty, rest, value, span, depth + 1);
            shape.insert(Part::Element(usize::MAX), possible);
            return dropped;
        }
        let mut child = shape.parts.remove(key).unwrap_or_default();
        let dropped = self.replace_obligation(&mut child, *child_ty, rest, value, span, depth + 1);
        shape.insert(key.clone(), child);
        dropped
    }
    fn obligation_statement(
        &mut self,
        c: &Context,
        n: NodeId,
        mut pending: Pending,
        depth: usize,
    ) -> Exits {
        let m = c.module;
        self.work = self.work.saturating_add(
            pending
                .values()
                .fold(0usize, |cost, v| cost.saturating_add(v.cost())),
        );
        if !self.tick(self.span(m, n), depth) {
            return Exits::default();
        }
        match self.node(m, n) {
            NodeKind::Block { statements } => {
                let mut locals = BTreeSet::new();
                let mut exits = Exits {
                    next: Some(pending),
                    ..Exits::default()
                };
                for statement in statements {
                    let Some(state) = exits.next.take() else {
                        break;
                    };
                    if let Some(s) = c.declarations.get(&statement.0) {
                        if matches!(self.node(m, statement), NodeKind::Binding { .. }) {
                            locals.insert(*s);
                        }
                    }
                    let flow = self.obligation_statement(c, statement, state, depth + 1);
                    exits.next = flow.next;
                    join(&mut exits.breaks, flow.breaks);
                    join(&mut exits.continues, flow.continues);
                }
                for state in [&mut exits.next, &mut exits.breaks, &mut exits.continues]
                    .into_iter()
                    .flatten()
                {
                    let dropped = state
                        .iter()
                        .filter(|(s, _)| locals.contains(s))
                        .map(|(s, v)| (*s, v.clone()))
                        .collect();
                    self.unhandled(m, n, &dropped);
                    state.retain(|s, _| !locals.contains(s));
                }
                return exits;
            }
            NodeKind::Binding { value, .. } => {
                let owns = self.obligation_expression(c, value, &mut pending, true, depth + 1);
                if owns.owns() {
                    if let Some(s) = c.declarations.get(&n.0) {
                        pending.insert(*s, owns);
                    }
                }
            }
            NodeKind::Assignment { place, value } => {
                // Resolve/evaluate index expressions before the RHS, as LANG-EXPR-001 requires.
                let target = self.obligation_place(c, place, &mut pending, depth + 1);
                let owns = self.obligation_expression(c, value, &mut pending, true, depth + 1);
                if let Some((symbol, path)) = target {
                    let ty = self.result.symbols[symbol.0].ty;
                    let mut shape = pending.remove(&symbol).unwrap_or_default();
                    if self.replace_obligation(
                        &mut shape,
                        ty,
                        &path,
                        owns,
                        self.span(m, place),
                        depth + 1,
                    ) {
                        self.error(
                            "T003",
                            self.span(m, place),
                            "assignment overwrites an unhandled Result obligation",
                        );
                    }
                    if shape.owns() {
                        pending.insert(symbol, shape);
                    }
                } else if owns.owns() {
                    self.error("T003", self.span(m, place), "Result storage through this reference requires safety-stage obligation tracking");
                }
            }
            NodeKind::Return { value } => {
                if let Some(value) = value {
                    self.obligation_expression(c, value, &mut pending, true, depth + 1);
                }
                self.unhandled(m, n, &pending);
                self.unhandled_iterators(c, self.span(m, n));
                return Exits::default();
            }
            NodeKind::ExpressionStatement { value } => {
                if self
                    .obligation_expression(c, value, &mut pending, true, depth + 1)
                    .owns()
                {
                    self.error("T003", self.span(m, value), "Result value must be handled, returned, propagated or explicitly discarded");
                }
            }
            NodeKind::If {
                condition,
                then_block,
                otherwise,
            } => {
                self.obligation_expression(c, condition, &mut pending, false, depth + 1);
                let literal = self.obligation_boolean(m, condition);
                let mut a = if literal != Some(false) {
                    self.obligation_statement(c, then_block, pending.clone(), depth + 1)
                } else {
                    Exits::default()
                };
                if literal != Some(true) {
                    let b = otherwise
                        .map(|n| self.obligation_statement(c, n, pending.clone(), depth + 1))
                        .unwrap_or(Exits {
                            next: Some(pending),
                            ..Exits::default()
                        });
                    join(&mut a.next, b.next);
                    join(&mut a.breaks, b.breaks);
                    join(&mut a.continues, b.continues);
                }
                return a;
            }
            NodeKind::Loop { body } => {
                return self.obligation_loop(c, body, pending, None, depth + 1)
            }
            NodeKind::While { condition, body } => {
                return self.obligation_loop(c, body, pending, Some(condition), depth + 1);
            }
            NodeKind::For { iterable, body, .. } => {
                return self.obligation_for(c, n, iterable, body, pending, depth + 1);
            }
            NodeKind::Break => {
                return Exits {
                    breaks: Some(pending),
                    ..Exits::default()
                }
            }
            NodeKind::Continue => {
                return Exits {
                    continues: Some(pending),
                    ..Exits::default()
                }
            }
            NodeKind::Match { subject, arms } => {
                let owns = self.obligation_expression(c, subject, &mut pending, true, depth + 1);
                let mut exits = Exits::default();
                for arm in arms {
                    if let NodeKind::Arm { pattern, body } = self.node(m, arm) {
                        self.work = self.work.saturating_add(owns.cost());
                        if !self.tick(self.span(m, arm), depth) {
                            return Exits::default();
                        }
                        let mut state = pending.clone();
                        let mut bindings = BTreeSet::new();
                        self.obligation_pattern(
                            c,
                            pattern,
                            self.obligation_type(m, subject),
                            owns.clone(),
                            &mut state,
                            &mut bindings,
                            depth + 1,
                        );
                        let mut arm = self.obligation_statement(c, body, state, depth + 1);
                        for state in [&mut arm.next, &mut arm.breaks, &mut arm.continues]
                            .into_iter()
                            .flatten()
                        {
                            let dropped = state
                                .iter()
                                .filter(|(s, _)| bindings.contains(s))
                                .map(|(s, v)| (*s, v.clone()))
                                .collect();
                            self.unhandled(m, body, &dropped);
                            state.retain(|s, _| !bindings.contains(s));
                        }
                        join(&mut exits.next, arm.next);
                        join(&mut exits.breaks, arm.breaks);
                        join(&mut exits.continues, arm.continues);
                    }
                }
                return exits;
            }
            _ => {}
        }
        Exits {
            next: Some(pending),
            ..Exits::default()
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn obligation_pattern(
        &mut self,
        c: &Context,
        n: NodeId,
        ty: TypeId,
        owns: Value,
        pending: &mut Pending,
        bindings: &mut BTreeSet<SymbolId>,
        depth: usize,
    ) {
        let m = c.module;
        if !owns.owns() || !self.tick(self.span(m, n), depth) {
            return;
        }
        match self.node(m, n) {
            NodeKind::WildcardPattern => {
                if self.owns_result(ty, self.span(m, n)) {
                    self.error(
                        "T003",
                        self.span(m, n),
                        "wildcard cannot discard an owned Result obligation",
                    );
                }
            }
            NodeKind::BindingPattern { .. } => {
                if let Some(s) = c.declarations.get(&n.0) {
                    pending.insert(*s, owns);
                    bindings.insert(*s);
                }
            }
            NodeKind::TuplePattern { fields } => {
                if let Some(Type::Tuple(types)) = self.result.types.get(ty).cloned() {
                    for (i, (p, t)) in fields.into_iter().zip(types).enumerate() {
                        let nested =
                            self.project_obligation(&owns, Part::Tuple(i), t, self.span(m, p));
                        self.obligation_pattern(c, p, t, nested, pending, bindings, depth + 1);
                    }
                }
            }
            NodeKind::ConstructorPattern { path, fields } => {
                let name = self.path(m, &path).last().cloned().unwrap_or_default();
                let types = match self.result.types.get(ty) {
                    Some(Type::Builtin {
                        name: family,
                        arguments,
                    }) if family == "Result" => vec![arguments[usize::from(name == "Err")]],
                    Some(Type::Builtin {
                        name: family,
                        arguments,
                    }) if family == "Option" && name == "Some" => vec![arguments[0]],
                    Some(Type::Nominal(s)) => self
                        .result
                        .enums
                        .get(s)
                        .and_then(|vs| vs.iter().find(|v| v.name == name))
                        .map(|v| v.payload.clone())
                        .unwrap_or_default(),
                    _ => vec![],
                };
                for (i, (p, t)) in fields.into_iter().zip(types).enumerate() {
                    let nested = self.project_obligation(
                        &owns,
                        Part::Payload(name.clone(), i),
                        t,
                        self.span(m, p),
                    );
                    self.obligation_pattern(c, p, t, nested, pending, bindings, depth + 1);
                }
            }
            _ => {}
        }
    }
    fn unhandled_iterators(&mut self, c: &Context, exit: Span) {
        for _ in &c.remaining_iterators {
            self.error(
                "T003",
                exit,
                "early exit may drop unvisited Result obligations in owned iteration",
            );
        }
    }
    fn obligation_for(
        &mut self,
        c: &Context,
        n: NodeId,
        iterable: NodeId,
        body: NodeId,
        mut pending: Pending,
        depth: usize,
    ) -> Exits {
        let m = c.module;
        let borrowed = matches!(
            self.result.types.get(self.obligation_type(m, iterable)),
            Some(Type::Reference { .. })
        );
        let shape = self.obligation_expression(c, iterable, &mut pending, !borrowed, depth + 1);
        let binding = c.declarations.get(&n.0).copied();
        let element = binding
            .map(|s| {
                self.project_obligation(
                    &shape,
                    Part::Element(usize::MAX),
                    self.result.symbols[s.0].ty,
                    self.span(m, n),
                )
            })
            .unwrap_or_default();
        let may_drop_remaining = !borrowed
            && (shape.unknown
                || shape
                    .parts
                    .iter()
                    .any(|(key, value)| matches!(key, Part::Element(i) if *i > 0) && value.owns()));
        let mut inner = Context {
            module: m,
            declarations: Rc::clone(&c.declarations),
            temporaries: Cell::new(c.temporaries.get()),
            remaining_iterators: c.remaining_iterators.clone(),
        };
        if may_drop_remaining {
            inner.remaining_iterators.push(self.span(m, n));
        }
        let mut header = pending.clone();
        let mut exit = Some(pending);
        loop {
            if !self.tick(self.span(m, n), depth) {
                break;
            }
            let mut state = header.clone();
            if let Some(s) = binding {
                if element.owns() && !borrowed {
                    state.insert(s, element.clone());
                }
            }
            let mut flow = self.obligation_statement(&inner, body, state, depth + 1);
            if may_drop_remaining && flow.breaks.is_some() {
                self.error(
                    "T003",
                    self.span(m, n),
                    "break may drop unvisited Result obligations in owned iteration",
                );
            }
            for state in [&mut flow.next, &mut flow.continues, &mut flow.breaks]
                .into_iter()
                .flatten()
            {
                if let Some(s) = binding {
                    if let Some(value) = state.remove(&s) {
                        self.unhandled(m, body, &BTreeMap::from([(s, value)]));
                    }
                }
            }
            join(&mut exit, flow.breaks);
            let mut back = flow.next;
            join(&mut back, flow.continues);
            join(&mut exit, back.clone());
            let mut next = Some(header.clone());
            join(&mut next, back);
            let next = next.unwrap_or_default();
            if next == header {
                break;
            }
            header = next;
        }
        Exits {
            next: exit,
            ..Exits::default()
        }
    }
    fn obligation_loop(
        &mut self,
        c: &Context,
        body: NodeId,
        initial: Pending,
        condition: Option<NodeId>,
        depth: usize,
    ) -> Exits {
        let mut header = initial;
        let mut exit = None;
        loop {
            if !self.tick(self.span(c.module, body), depth) {
                break;
            }
            let mut state = header.clone();
            if let Some(condition) = condition {
                self.obligation_expression(c, condition, &mut state, false, depth + 1);
                let literal = self.obligation_boolean(c.module, condition);
                if literal != Some(true) {
                    join(&mut exit, Some(state.clone()));
                }
                if literal == Some(false) {
                    break;
                }
            }
            let flow = self.obligation_statement(c, body, state, depth + 1);
            join(&mut exit, flow.breaks);
            let mut back = flow.next;
            join(&mut back, flow.continues);
            let mut next = header.clone();
            if let Some(back) = back {
                for (s, v) in back {
                    next.entry(s).or_default().merge(v);
                }
            }
            if next == header {
                break;
            }
            header = next;
        }
        Exits {
            next: exit,
            ..Exits::default()
        }
    }
    fn obligation_boolean(&self, m: ModuleId, mut n: NodeId) -> Option<bool> {
        loop {
            match self.node(m, n) {
                NodeKind::Group { value } => n = value,
                NodeKind::Literal { kind: Kind::True } => return Some(true),
                NodeKind::Literal { kind: Kind::False } => return Some(false),
                _ => return None,
            }
        }
    }
}
