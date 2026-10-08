//! Pattern-matrix specialization checks both usefulness and exhaustiveness.
use super::*;
use crate::token::Kind;
#[derive(Clone)]
enum Pat {
    Any,
    Constructor(String, Vec<Pat>),
}
impl Analyzer<'_> {
    pub(super) fn match_coverage(
        &mut self,
        m: ModuleId,
        n: NodeId,
        ty: TypeId,
        patterns: &[NodeId],
    ) {
        let mut matrix = vec![];
        for p in patterns {
            let pat = self.cover_pattern(m, *p, 0);
            if !self.useful(&matrix, &[pat.clone()], &[ty], self.span(m, *p), 0) && !self.exhausted
            {
                self.error(
                    "T002",
                    self.span(m, *p),
                    "unreachable match arm: earlier patterns cover every matching value",
                );
            }
            matrix.push(vec![pat]);
        }
        if self.useful(&matrix, &[Pat::Any], &[ty], self.span(m, n), 0) && !self.exhausted {
            self.error(
                "T002",
                self.span(m, n),
                "non-exhaustive match: some alternatives remain uncovered",
            );
        }
    }
    fn cover_pattern(&mut self, m: ModuleId, n: NodeId, depth: usize) -> Pat {
        if !self.tick(self.span(m, n), depth) {
            return Pat::Any;
        }
        match self.node(m, n) {
            NodeKind::ConstructorPattern { path, fields } => Pat::Constructor(
                self.path(m, &path).last().cloned().unwrap_or_default(),
                fields
                    .into_iter()
                    .map(|n| self.cover_pattern(m, n, depth + 1))
                    .collect(),
            ),
            NodeKind::TuplePattern { fields } => Pat::Constructor(
                "tuple".into(),
                fields
                    .into_iter()
                    .map(|n| self.cover_pattern(m, n, depth + 1))
                    .collect(),
            ),
            NodeKind::LiteralPattern { kind } => {
                let raw = self.sources[m.0].slice(self.span(m, n)).unwrap_or("");
                let key = match kind {
                    Kind::True => "true".into(),
                    Kind::False => "false".into(),
                    Kind::Int => {
                        let s = raw.replace('_', "");
                        let (base, d) = if let Some(d) = s.strip_prefix("0x") {
                            (16, d)
                        } else if let Some(d) = s.strip_prefix("0o") {
                            (8, d)
                        } else if let Some(d) = s.strip_prefix("0b") {
                            (2, d)
                        } else {
                            (10, s.as_str())
                        };
                        format!("int:{}", u128::from_str_radix(d, base).unwrap_or(0))
                    }
                    _ => format!(
                        "{kind:?}:{}",
                        super::constants::decode_literal(raw).unwrap_or_else(|| raw.into())
                    ),
                };
                Pat::Constructor(key, vec![])
            }
            _ => Pat::Any,
        }
    }
    fn constructors(&self, t: TypeId) -> Option<Vec<(String, Vec<TypeId>)>> {
        let t = match self.result.types.get(t) {
            Some(Type::Reference { target, .. }) => *target,
            _ => t,
        };
        match self.result.types.get(t) {
            Some(Type::Bool) => Some(vec![("true".into(), vec![]), ("false".into(), vec![])]),
            Some(Type::Tuple(ts)) => Some(vec![("tuple".into(), ts.clone())]),
            Some(Type::Nominal(s)) => self.enums.get(s).map(|vs| {
                vs.iter()
                    .map(|v| (v.name.clone(), v.payload.clone()))
                    .collect()
            }),
            Some(Type::Builtin { name, arguments }) if name == "Option" => Some(vec![
                ("Some".into(), vec![arguments[0]]),
                ("None".into(), vec![]),
            ]),
            Some(Type::Builtin { name, arguments }) if name == "Result" => Some(vec![
                ("Ok".into(), vec![arguments[0]]),
                ("Err".into(), vec![arguments[1]]),
            ]),
            _ => None,
        }
    }
    fn useful(
        &mut self,
        matrix: &[Vec<Pat>],
        row: &[Pat],
        types: &[TypeId],
        span: Span,
        depth: usize,
    ) -> bool {
        if !self.tick(span, depth) {
            return false;
        }
        if row.len() != types.len() {
            return true;
        }
        if row.is_empty() {
            return matrix.is_empty();
        }
        if matrix.is_empty() {
            return true;
        }
        match &row[0] {
            Pat::Constructor(key, fields) => {
                let ts = self
                    .constructors(types[0])
                    .and_then(|cs| cs.into_iter().find(|(k, _)| k == key).map(|(_, ts)| ts))
                    .unwrap_or_default();
                if ts.len() != fields.len() {
                    return true;
                }
                let specialized = specialize(matrix, key, fields.len());
                let mut candidate = fields.clone();
                candidate.extend_from_slice(&row[1..]);
                let mut new_types = ts;
                new_types.extend_from_slice(&types[1..]);
                self.useful(&specialized, &candidate, &new_types, span, depth + 1)
            }
            Pat::Any => {
                if let Some(constructors) = self.constructors(types[0]) {
                    for (key, ts) in constructors {
                        let specialized = specialize(matrix, &key, ts.len());
                        let mut candidate = vec![Pat::Any; ts.len()];
                        candidate.extend_from_slice(&row[1..]);
                        let mut new_types = ts;
                        new_types.extend_from_slice(&types[1..]);
                        if self.useful(&specialized, &candidate, &new_types, span, depth + 1) {
                            return true;
                        }
                    }
                    false
                } else {
                    let defaults: Vec<_> = matrix
                        .iter()
                        .filter(|r| matches!(r.first(), Some(Pat::Any)))
                        .map(|r| r[1..].to_vec())
                        .collect();
                    self.useful(&defaults, &row[1..], &types[1..], span, depth + 1)
                }
            }
        }
    }
}
fn specialize(matrix: &[Vec<Pat>], key: &str, arity: usize) -> Vec<Vec<Pat>> {
    matrix
        .iter()
        .filter_map(|row| match row.first() {
            Some(Pat::Any) => {
                let mut r = vec![Pat::Any; arity];
                r.extend_from_slice(&row[1..]);
                Some(r)
            }
            Some(Pat::Constructor(k, fs)) if k == key && fs.len() == arity => {
                let mut r = fs.clone();
                r.extend_from_slice(&row[1..]);
                Some(r)
            }
            _ => None,
        })
        .collect()
}
