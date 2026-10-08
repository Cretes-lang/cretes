//! Type well-formedness and public interface checks, independent of layout sizing.
use super::*;
impl Analyzer<'_> {
    pub(super) fn validate_types(&mut self) {
        // Walk each interned type once. Nominal definitions are checked separately;
        // following their children here would repeatedly expand recursive graphs.
        let types: Vec<_> = self
            .result
            .types
            .iter()
            .map(|(id, t)| (id, t.clone()))
            .collect();
        let mut spans = BTreeMap::new();
        for m in &self.result.modules {
            for (n, t) in &m.node_types {
                spans.entry(*t).or_insert(m.ast.nodes[*n].span);
            }
        }
        for s in &self.result.symbols {
            spans.entry(s.ty).or_insert(s.span);
        }
        for (id, ty) in types {
            let Some(&span) = spans.get(&id) else {
                continue;
            };
            match ty {
                Type::Builtin { name, arguments } => {
                    if arguments
                        .iter()
                        .any(|t| matches!(self.result.types.get(*t), Some(Type::Reference { .. })))
                    {
                        self.error("B002", span, "aggregate storage cannot contain references");
                    }
                    if matches!(name.as_str(), "Map" | "Set") && !self.hash_key(arguments[0], span)
                    {
                        self.error("T001",span,"Map/Set key requires built-in hashing: integer, bool, char, text, Bytes or tuples thereof");
                    }
                }
                Type::Tuple(ts)
                    if ts.iter().any(|t| {
                        matches!(self.result.types.get(*t), Some(Type::Reference { .. }))
                    }) =>
                {
                    self.error("B002", span, "tuples cannot store references")
                }
                _ => {}
            }
        }
        for i in 0..self.result.symbols.len() {
            if !self.result.symbols[i].public {
                continue;
            }
            let id = SymbolId(i);
            let span = self.result.symbols[i].span;
            let mut exposed = vec![];
            match self.result.symbols[i].kind {
                SymbolKind::Function => {
                    if let Some(s) = self.result.signatures.get(&id) {
                        exposed.extend(s.parameters.iter().map(|t| (*t, span)));
                        exposed.push((s.result, span));
                    }
                }
                SymbolKind::Record => {
                    if let Some(fs) = self.result.records.get(&id) {
                        exposed.extend(fs.iter().filter(|f| f.public).map(|f| (f.ty, f.span)));
                    }
                }
                SymbolKind::Enum => {
                    if let Some(vs) = self.result.enums.get(&id) {
                        for v in vs {
                            exposed.extend(v.payload.iter().map(|t| (*t, v.span)));
                        }
                    }
                }
                SymbolKind::Alias | SymbolKind::Constant => {
                    exposed.push((self.result.symbols[i].ty, span))
                }
                _ => {}
            }
            for (ty, span) in exposed {
                self.public_type(ty, span);
            }
        }
    }
    fn hash_key(&mut self, root: TypeId, span: Span) -> bool {
        let mut stack = vec![root];
        let mut seen = BTreeSet::new();
        while let Some(t) = stack.pop() {
            if !seen.insert(t) {
                continue;
            }
            if !self.tick(span, 0) {
                return false;
            }
            match self.result.types.get(t) {
                Some(
                    Type::Error
                    | Type::Integer { .. }
                    | Type::Usize
                    | Type::Bool
                    | Type::Char
                    | Type::Text
                    | Type::Bytes,
                ) => {}
                Some(Type::Tuple(ts)) => stack.extend(ts),
                _ => return false,
            }
        }
        true
    }
    fn public_type(&mut self, root: TypeId, span: Span) {
        let mut stack = vec![root];
        let mut seen = BTreeSet::new();
        while let Some(t) = stack.pop() {
            if !seen.insert(t) {
                continue;
            }
            if !self.tick(span, 0) {
                return;
            }
            match self.result.types.get(t).cloned() {
                Some(Type::Nominal(s)) if !self.result.symbols[s.0].public => self.error(
                    "M001",
                    span,
                    format!(
                        "public API exposes private type `{}`",
                        self.result.symbols[s.0].name
                    ),
                ),
                Some(Type::Tuple(ts)) | Some(Type::Builtin { arguments: ts, .. }) => {
                    stack.extend(ts)
                }
                Some(Type::Reference { target, .. }) => stack.push(target),
                _ => {}
            }
        }
    }
}
