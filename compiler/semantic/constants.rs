//! Restricted, fuel-bounded constant evaluation. No calls or host code execution.
use super::*;
use crate::token::Kind;
#[derive(Clone, Debug, PartialEq)]
pub enum ConstantValue {
    Integer(i128),
    Float32(f32),
    Float64(f64),
    Bool(bool),
    Char(char),
    Tuple(Vec<ConstantValue>),
}
impl Analyzer<'_> {
    pub(super) fn check_constant(&mut self, s: SymbolId, depth: usize) -> Option<ConstantValue> {
        if let Some(v) = self.result.constants.get(&s) {
            return Some(v.clone());
        }
        let symbol = &self.result.symbols[s.0];
        let (m, n, scope, ty) = (symbol.module, symbol.node, symbol.scope, symbol.ty);
        if !self.tick(self.span(m, n), depth) {
            return None;
        }
        if self.constant_states.get(&s) == Some(&1) {
            self.error("C001", self.span(m, n), "cyclic constant dependency");
            return None;
        }
        if self.constant_states.get(&s) == Some(&2) {
            return None;
        }
        self.constant_states.insert(s, 1);
        let NodeKind::Constant { value, .. } = self.node(m, n) else {
            return None;
        };
        let mut ctx = check::FunctionContext {
            module: m,
            result: ty,
            loops: 0,
        };
        self.expression(&mut ctx, scope, value, Some(ty), depth + 1);
        let valid = self.constant_type(ty, 0);
        if !self.constant_syntax(m, scope, value, depth + 1) {
            self.constant_states.insert(s, 2);
            return None;
        }
        let result = if valid {
            self.eval_constant(m, scope, value, depth + 1)
        } else {
            self.error(
                "C001",
                self.span(m, n),
                "constant type must be a scalar, unit or tuple of constant types",
            );
            None
        };
        self.constant_states.insert(s, 2);
        if let Some(ref v) = result {
            self.result.constants.insert(s, v.clone());
        }
        result
    }
    // Validate even unevaluated operands: short circuiting skips arithmetic, not the language's constant-expression restriction.
    fn constant_syntax(&mut self, m: ModuleId, scope: ScopeId, n: NodeId, depth: usize) -> bool {
        if !self.tick(self.span(m, n), depth) {
            return false;
        }
        let valid = match self.node(m, n) {
            NodeKind::Literal { kind } => matches!(
                kind,
                Kind::Int | Kind::Float | Kind::True | Kind::False | Kind::Char
            ),
            NodeKind::Group { value } => self.constant_syntax(m, scope, value, depth + 1),
            NodeKind::Tuple { elements } => elements.into_iter().fold(true, |ok, e| {
                self.constant_syntax(m, scope, e, depth + 1) && ok
            }),
            NodeKind::Unary {
                operator, operand, ..
            } => {
                matches!(operator, Kind::Minus | Kind::Bang | Kind::Tilde)
                    && self.constant_syntax(m, scope, operand, depth + 1)
            }
            NodeKind::Binary { left, right, .. } => {
                let a = self.constant_syntax(m, scope, left, depth + 1);
                let b = self.constant_syntax(m, scope, right, depth + 1);
                a && b
            }
            NodeKind::Path { components } => {
                let parts = self.path(m, &components);
                self.lookup(m, scope, &parts, self.span(m, n))
                    .is_some_and(|s| self.result.symbols[s.0].kind == SymbolKind::Constant)
            }
            _ => false,
        };
        if !valid {
            self.error(
                "C001",
                self.span(m, n),
                "expression is not permitted in a constant initializer",
            );
        }
        valid
    }
    fn constant_type(&self, root: TypeId, _depth: usize) -> bool {
        let mut stack = vec![root];
        let mut seen = BTreeSet::new();
        while let Some(t) = stack.pop() {
            if !seen.insert(t) {
                continue;
            }
            match self.result.types.get(t) {
                Some(
                    Type::Integer { .. } | Type::Usize | Type::Float(_) | Type::Bool | Type::Char,
                ) => {}
                Some(Type::Tuple(ts)) => stack.extend(ts),
                _ => return false,
            }
        }
        true
    }
    fn eval_constant(
        &mut self,
        m: ModuleId,
        scope: ScopeId,
        n: NodeId,
        depth: usize,
    ) -> Option<ConstantValue> {
        if !self.tick(self.span(m, n), depth) {
            return None;
        }
        let ty = self.result.modules[m.0]
            .node_types
            .get(&n.0)
            .copied()
            .unwrap_or(Types::ERROR);
        let value = match self.node(m, n) {
            NodeKind::Literal { kind } => {
                let text = self.sources[m.0].slice(self.span(m, n)).unwrap_or("");
                let text = if matches!(kind, Kind::Int | Kind::Float) {
                    text.replace('_', "")
                } else {
                    text.to_owned()
                };
                match kind {
                    Kind::True => Some(ConstantValue::Bool(true)),
                    Kind::False => Some(ConstantValue::Bool(false)),
                    Kind::Int => parse_integer(&text).map(ConstantValue::Integer),
                    Kind::Float => match self.result.types.get(ty) {
                        Some(Type::Float(32)) => text.parse().ok().map(ConstantValue::Float32),
                        _ => text.parse().ok().map(ConstantValue::Float64),
                    },
                    Kind::Char => decode_char(&text).map(ConstantValue::Char),
                    _ => None,
                }
            }
            NodeKind::Group { value } => self.eval_constant(m, scope, value, depth + 1),
            NodeKind::Tuple { elements } => elements
                .into_iter()
                .map(|e| self.eval_constant(m, scope, e, depth + 1))
                .collect::<Option<Vec<_>>>()
                .map(ConstantValue::Tuple),
            NodeKind::Path { components } => {
                let parts = self.path(m, &components);
                let symbol = self.lookup(m, scope, &parts, self.span(m, n));
                if let Some(s) = symbol {
                    if self.result.symbols[s.0].kind == SymbolKind::Constant {
                        self.check_constant(s, depth + 1)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            NodeKind::Unary {
                operator, operand, ..
            } => {
                // The positive magnitude of a signed minimum is intentionally not range checked alone.
                let v = if operator == Kind::Minus
                    && matches!(self.node(m, operand), NodeKind::Literal { kind: Kind::Int })
                {
                    parse_integer(
                        &self.sources[m.0]
                            .slice(self.span(m, operand))
                            .unwrap_or("")
                            .replace('_', ""),
                    )
                    .map(ConstantValue::Integer)
                } else {
                    self.eval_constant(m, scope, operand, depth + 1)
                };
                match (operator, v) {
                    (Kind::Minus, Some(ConstantValue::Integer(x))) => {
                        x.checked_neg().map(ConstantValue::Integer)
                    }
                    (Kind::Tilde, Some(ConstantValue::Integer(x))) => {
                        self.integer(ty).map(|(signed, bits)| {
                            ConstantValue::Integer(if signed {
                                !x
                            } else {
                                (!x) & ((1i128 << bits) - 1)
                            })
                        })
                    }
                    (Kind::Bang, Some(ConstantValue::Bool(x))) => Some(ConstantValue::Bool(!x)),
                    (Kind::Minus, Some(ConstantValue::Float32(x))) => {
                        Some(ConstantValue::Float32(-x))
                    }
                    (Kind::Minus, Some(ConstantValue::Float64(x))) => {
                        Some(ConstantValue::Float64(-x))
                    }
                    _ => None,
                }
            }
            NodeKind::Binary {
                operator,
                left,
                right,
            } => {
                let a = self.eval_constant(m, scope, left, depth + 1)?;
                if (operator == Kind::AndAnd && a == ConstantValue::Bool(false))
                    || (operator == Kind::OrOr && a == ConstantValue::Bool(true))
                {
                    Some(a)
                } else {
                    let b = self.eval_constant(m, scope, right, depth + 1)?;
                    let operand_type = self.result.modules[m.0]
                        .node_types
                        .get(&left.0)
                        .copied()
                        .unwrap_or(Types::ERROR);
                    self.constant_binary(operator, a, b, operand_type)
                }
            }
            _ => None,
        };
        let valid = value.as_ref().is_some_and(|v| self.constant_range(v, ty));
        if !valid {
            self.error(
                "C001",
                self.span(m, n),
                "invalid constant expression, overflow, division or shift",
            );
            None
        } else {
            value
        }
    }
    fn constant_range(&self, v: &ConstantValue, t: TypeId) -> bool {
        if let ConstantValue::Integer(x) = v {
            if let Some((signed, bits)) = self.integer(t) {
                let low = if signed { -(1i128 << (bits - 1)) } else { 0 };
                let high = if signed {
                    (1i128 << (bits - 1)) - 1
                } else {
                    (1i128 << bits) - 1
                };
                return *x >= low && *x <= high;
            }
            return false;
        }
        !matches!(self.result.types.get(t), None | Some(Type::Error))
    }
    fn constant_binary(
        &self,
        op: Kind,
        a: ConstantValue,
        b: ConstantValue,
        t: TypeId,
    ) -> Option<ConstantValue> {
        use ConstantValue::*;
        match (a, b) {
            (Integer(x), Integer(y)) => {
                let bounds = self.integer(t)?;
                let (signed, bits) = bounds;
                let min = if signed { -(1i128 << (bits - 1)) } else { 0 };
                let arithmetic = match op {
                    Kind::Plus => x.checked_add(y),
                    Kind::Minus => x.checked_sub(y),
                    Kind::Star => x.checked_mul(y),
                    Kind::Slash if !(signed && x == min && y == -1) => x.checked_div(y),
                    Kind::Percent if !(signed && x == min && y == -1) => x.checked_rem(y),
                    Kind::Amp => Some(x & y),
                    Kind::Pipe => Some(x | y),
                    Kind::Caret => Some(x ^ y),
                    Kind::Shl if y >= 0 && y < i128::from(bits) => {
                        x.checked_mul(1i128 << (y as u32))
                    }
                    Kind::Shr if y >= 0 && y < i128::from(bits) => Some(x >> (y as u32)),
                    _ => None,
                };
                if let Some(v) = arithmetic {
                    return Some(Integer(v));
                }
                match op {
                    Kind::EqEq => Some(Bool(x == y)),
                    Kind::NotEq => Some(Bool(x != y)),
                    Kind::Lt => Some(Bool(x < y)),
                    Kind::Le => Some(Bool(x <= y)),
                    Kind::Gt => Some(Bool(x > y)),
                    Kind::Ge => Some(Bool(x >= y)),
                    _ => None,
                }
            }
            (Float32(x), Float32(y)) => float32(op, x, y),
            (Float64(x), Float64(y)) => float64(op, x, y),
            (Bool(x), Bool(y)) => match op {
                Kind::AndAnd => Some(Bool(x && y)),
                Kind::OrOr => Some(Bool(x || y)),
                Kind::EqEq => Some(Bool(x == y)),
                Kind::NotEq => Some(Bool(x != y)),
                _ => None,
            },
            (Char(x), Char(y)) => match op {
                Kind::EqEq => Some(Bool(x == y)),
                Kind::NotEq => Some(Bool(x != y)),
                Kind::Lt => Some(Bool(x < y)),
                Kind::Le => Some(Bool(x <= y)),
                Kind::Gt => Some(Bool(x > y)),
                Kind::Ge => Some(Bool(x >= y)),
                _ => None,
            },
            _ => None,
        }
    }
}
fn parse_integer(s: &str) -> Option<i128> {
    let (base, d) = if let Some(d) = s.strip_prefix("0x") {
        (16, d)
    } else if let Some(d) = s.strip_prefix("0b") {
        (2, d)
    } else if let Some(d) = s.strip_prefix("0o") {
        (8, d)
    } else {
        (10, s)
    };
    i128::from_str_radix(d, base).ok()
}
fn decode_char(s: &str) -> Option<char> {
    let s = s.strip_prefix('\'')?.strip_suffix('\'')?;
    if let Some(s) = s.strip_prefix("\\u{") {
        return char::from_u32(u32::from_str_radix(s.strip_suffix('}')?, 16).ok()?);
    }
    match s {
        "\\n" => Some('\n'),
        "\\r" => Some('\r'),
        "\\t" => Some('\t'),
        "\\0" => Some('\0'),
        "\\\\" => Some('\\'),
        "\\\"" => Some('"'),
        "\\'" => Some('\''),
        _ => {
            let mut chars = s.chars();
            let c = chars.next()?;
            chars.next().is_none().then_some(c)
        }
    }
}
macro_rules! floats {
    ($name:ident,$ty:ty,$variant:ident) => {
        fn $name(op: Kind, x: $ty, y: $ty) -> Option<ConstantValue> {
            use ConstantValue::*;
            match op {
                Kind::Plus => Some($variant(x + y)),
                Kind::Minus => Some($variant(x - y)),
                Kind::Star => Some($variant(x * y)),
                Kind::Slash => Some($variant(x / y)),
                Kind::EqEq => Some(Bool(x == y)),
                Kind::NotEq => Some(Bool(x != y)),
                Kind::Lt => Some(Bool(x < y)),
                Kind::Le => Some(Bool(x <= y)),
                Kind::Gt => Some(Bool(x > y)),
                Kind::Ge => Some(Bool(x >= y)),
                _ => None,
            }
        }
    };
}
floats!(float32, f32, Float32);
floats!(float64, f64, Float64);
/// Decode literal identity for pattern equivalence; source spelling stays in spans.
pub(super) fn decode_literal(s: &str) -> Option<String> {
    let s = s.strip_prefix('b').unwrap_or(s);
    let mut chars = s.chars();
    let quote = chars.next()?;
    let s = s.strip_suffix(quote)?.get(quote.len_utf8()..)?;
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        out.push(match chars.next()? {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '0' => '\0',
            '\\' => '\\',
            '\'' => '\'',
            '"' => '"',
            'x' => {
                let a = chars.next()?.to_digit(16)?;
                let b = chars.next()?.to_digit(16)?;
                char::from_u32(a * 16 + b)?
            }
            'u' => {
                if chars.next()? != '{' {
                    return None;
                }
                let mut v = 0u32;
                loop {
                    let c = chars.next()?;
                    if c == '}' {
                        break;
                    }
                    v = v.checked_mul(16)?.checked_add(c.to_digit(16)?)?;
                }
                char::from_u32(v)?
            }
            _ => return None,
        });
    }
    Some(out)
}
