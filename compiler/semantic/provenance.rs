//! Direct-reference return contracts. Full owner/loan validity is a later stage.
use super::*;
use crate::token::Kind;
type Origins = BTreeSet<SymbolId>;
type State = BTreeMap<SymbolId, Origins>;
const UNKNOWN: SymbolId = SymbolId(usize::MAX);
#[derive(Default)]
struct Exits {
    next: Option<State>,
    breaks: Option<State>,
    continues: Option<State>,
}
fn merge(into: &mut Option<State>, other: Option<State>) {
    if let Some(other) = other {
        if let Some(into) = into {
            for (s, origins) in other {
                into.entry(s).or_default().extend(origins);
            }
        } else {
            *into = Some(other);
        }
    }
}
struct Contract {
    module: ModuleId,
    source: SymbolId,
    declarations: BTreeMap<usize, SymbolId>,
}
impl Analyzer<'_> {
    pub(super) fn check_return_provenance(&mut self, function: SymbolId) {
        let symbol = &self.result.symbols[function.0];
        let m = symbol.module;
        let function_span = self.span(m, symbol.node);
        let NodeKind::Function {
            parameters, body, ..
        } = self.node(m, symbol.node)
        else {
            return;
        };
        let Some(sig) = self.result.signatures.get(&function) else {
            return;
        };
        let Some(index) = sig.from else {
            return;
        };
        self.work = self.work.saturating_add(self.result.symbols.len());
        if !self.tick(function_span, 0) {
            return;
        }
        let declarations: BTreeMap<_, _> = self
            .result
            .symbols
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                s.module == m
                    && s.span.start >= function_span.start
                    && s.span.end <= function_span.end
                    && matches!(self.result.types.get(s.ty), Some(Type::Reference { .. }))
                    && matches!(
                        s.kind,
                        SymbolKind::Parameter | SymbolKind::Local | SymbolKind::Pattern
                    )
            })
            .map(|(i, s)| (s.node.0, SymbolId(i)))
            .collect();
        let Some(source) = parameters
            .get(index)
            .and_then(|p| declarations.get(&p.0))
            .copied()
        else {
            return;
        };
        let contract = Contract {
            module: m,
            source,
            declarations,
        };
        let mut state = State::new();
        for parameter in parameters {
            if let Some(s) = contract.declarations.get(&parameter.0) {
                state.insert(*s, [*s].into());
            }
        }
        self.provenance_statement(&contract, body, state, 0);
    }
    fn reference_origins(
        &mut self,
        c: &Contract,
        n: NodeId,
        state: &State,
        depth: usize,
    ) -> Origins {
        let m = c.module;
        if !self.tick(self.span(m, n), depth) {
            return [UNKNOWN].into();
        }
        match self.node(m, n) {
            NodeKind::Group { value } => self.reference_origins(c, value, state, depth + 1),
            NodeKind::Path { .. } => self.result.modules[m.0]
                .resolutions
                .get(&n.0)
                .map(|s| state.get(s).cloned().unwrap_or_else(|| [*s].into()))
                .unwrap_or_else(|| [UNKNOWN].into()),
            NodeKind::Unary {
                operator: Kind::Amp,
                operand,
                ..
            } => self.place_origins(c, operand, state, depth + 1),
            NodeKind::Call { callee, arguments } => {
                let from = self.result.modules[m.0]
                    .resolutions
                    .get(&callee.0)
                    .and_then(|s| self.result.signatures.get(s))
                    .and_then(|s| s.from);
                from.and_then(|i| arguments.get(i))
                    .map(|n| self.reference_origins(c, *n, state, depth + 1))
                    .unwrap_or_else(|| [UNKNOWN].into())
            }
            _ => [UNKNOWN].into(),
        }
    }
    fn place_origins(&mut self, c: &Contract, n: NodeId, state: &State, depth: usize) -> Origins {
        if !self.tick(self.span(c.module, n), depth) {
            return [UNKNOWN].into();
        }
        match self.node(c.module, n) {
            NodeKind::Group { value } => self.place_origins(c, value, state, depth + 1),
            NodeKind::Member { receiver, .. } | NodeKind::Index { receiver, .. } => {
                self.place_origins(c, receiver, state, depth + 1)
            }
            NodeKind::Unary {
                operator: Kind::Star,
                operand,
                ..
            } => self.reference_origins(c, operand, state, depth + 1),
            NodeKind::Path { .. } => self.result.modules[c.module.0]
                .resolutions
                .get(&n.0)
                .map(|s| [*s].into())
                .unwrap_or_else(|| [UNKNOWN].into()),
            _ => [UNKNOWN].into(),
        }
    }
    fn provenance_statement(
        &mut self,
        c: &Contract,
        n: NodeId,
        mut state: State,
        depth: usize,
    ) -> Exits {
        let m = c.module;
        // Charge copied dataflow state, not just AST visits, to the shared fuel.
        let state_work = state
            .values()
            .fold(state.len(), |n, origins| n.saturating_add(origins.len()));
        self.work = self.work.saturating_add(state_work);
        if !self.tick(self.span(m, n), depth) {
            return Exits::default();
        }
        match self.node(m, n) {
            NodeKind::Block { statements } => {
                let mut exits = Exits {
                    next: Some(state),
                    ..Exits::default()
                };
                for statement in statements {
                    let Some(state) = exits.next.take() else {
                        break;
                    };
                    let next = self.provenance_statement(c, statement, state, depth + 1);
                    exits.next = next.next;
                    merge(&mut exits.breaks, next.breaks);
                    merge(&mut exits.continues, next.continues);
                }
                return exits;
            }
            NodeKind::Binding { value, .. } => {
                if let Some(s) = c.declarations.get(&n.0) {
                    let origins = self.reference_origins(c, value, &state, depth + 1);
                    state.insert(*s, origins);
                }
            }
            NodeKind::Assignment { place, value } => {
                let mut target = place;
                while let NodeKind::Group { value } = self.node(m, target) {
                    target = value;
                }
                if let Some(s) = self.result.modules[m.0].resolutions.get(&target.0).copied() {
                    if matches!(
                        self.result.types.get(self.result.symbols[s.0].ty),
                        Some(Type::Reference { .. })
                    ) {
                        let origins = self.reference_origins(c, value, &state, depth + 1);
                        state.insert(s, origins);
                    }
                }
            }
            NodeKind::Return { value } => {
                if let Some(value) = value {
                    let origins = self.reference_origins(c, value, &state, depth + 1);
                    if origins != [c.source].into() {
                        self.error("B002", self.span(m, value), "returned reference does not derive exclusively from the declared `from` parameter");
                    }
                }
                return Exits::default();
            }
            NodeKind::If {
                condition,
                then_block,
                otherwise,
            } => {
                let literal = self.provenance_boolean(m, condition);
                let mut exits = if literal != Some(false) {
                    self.provenance_statement(c, then_block, state.clone(), depth + 1)
                } else {
                    Exits::default()
                };
                if literal != Some(true) {
                    let other = otherwise
                        .map(|n| self.provenance_statement(c, n, state.clone(), depth + 1))
                        .unwrap_or(Exits {
                            next: Some(state),
                            ..Exits::default()
                        });
                    merge(&mut exits.next, other.next);
                    merge(&mut exits.breaks, other.breaks);
                    merge(&mut exits.continues, other.continues);
                }
                return exits;
            }
            NodeKind::Loop { body } => {
                return self.provenance_loop(c, body, state, false, depth + 1)
            }
            NodeKind::While { condition, body } => {
                let literal = self.provenance_boolean(m, condition);
                if literal != Some(false) {
                    return self.provenance_loop(c, body, state, literal != Some(true), depth + 1);
                }
            }
            NodeKind::For { iterable, body, .. } => {
                if let Some(s) = c.declarations.get(&n.0) {
                    let origins = self.reference_origins(c, iterable, &state, depth + 1);
                    state.insert(*s, origins);
                }
                return self.provenance_loop(c, body, state, true, depth + 1);
            }
            NodeKind::Match { subject, arms } => {
                let origins = self.reference_origins(c, subject, &state, depth + 1);
                let mut exits = Exits::default();
                for arm in arms {
                    if let NodeKind::Arm { pattern, body } = self.node(m, arm) {
                        let mut arm_state = state.clone();
                        let mut pending = vec![pattern];
                        while let Some(pattern) = pending.pop() {
                            if !self.tick(self.span(m, pattern), depth) {
                                break;
                            }
                            match self.node(m, pattern) {
                                NodeKind::BindingPattern { .. } => {
                                    if let Some(s) = c.declarations.get(&pattern.0) {
                                        arm_state.insert(*s, origins.clone());
                                    }
                                }
                                NodeKind::ConstructorPattern { fields, .. }
                                | NodeKind::TuplePattern { fields } => pending.extend(fields),
                                _ => {}
                            }
                        }
                        let arm = self.provenance_statement(c, body, arm_state, depth + 1);
                        merge(&mut exits.next, arm.next);
                        merge(&mut exits.breaks, arm.breaks);
                        merge(&mut exits.continues, arm.continues);
                    }
                }
                return exits;
            }
            NodeKind::Break => {
                return Exits {
                    breaks: Some(state),
                    ..Exits::default()
                }
            }
            NodeKind::Continue => {
                return Exits {
                    continues: Some(state),
                    ..Exits::default()
                }
            }
            _ => {}
        }
        Exits {
            next: Some(state),
            ..Exits::default()
        }
    }
    fn provenance_loop(
        &mut self,
        c: &Contract,
        body: NodeId,
        initial: State,
        may_skip: bool,
        depth: usize,
    ) -> Exits {
        let mut header = initial.clone();
        let mut exit = None;
        loop {
            if !self.tick(self.span(c.module, body), depth) {
                break;
            }
            let flow = self.provenance_statement(c, body, header.clone(), depth + 1);
            merge(&mut exit, flow.breaks);
            let mut back = flow.next;
            merge(&mut back, flow.continues);
            let mut next = Some(header.clone());
            merge(&mut next, back);
            let next = next.unwrap_or_default();
            if next == header {
                break;
            }
            header = next;
        }
        if may_skip {
            merge(&mut exit, Some(header));
        }
        Exits {
            next: exit,
            ..Exits::default()
        }
    }
    fn provenance_boolean(&self, m: ModuleId, mut n: NodeId) -> Option<bool> {
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
