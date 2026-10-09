//! May-be-unhandled Result values, tracked independently of type inference.
use super::*;
use crate::token::Kind;
type Pending = BTreeSet<SymbolId>;
#[derive(Default)]
struct Exits {
    next: Option<Pending>,
    breaks: Option<Pending>,
    continues: Option<Pending>,
}
fn join(a: &mut Option<Pending>, b: Option<Pending>) {
    if let Some(b) = b {
        a.get_or_insert_with(Pending::new).extend(b);
    }
}
struct Context {
    module: ModuleId,
    declarations: BTreeMap<usize, SymbolId>,
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
            declarations,
        };
        let mut pending = Pending::new();
        for p in parameters {
            if let Some(s) = c.declarations.get(&p.0) {
                if self.owns_result(self.result.symbols[s.0].ty, span) {
                    pending.insert(*s);
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
        for s in pending {
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
    fn obligation_expression(
        &mut self,
        c: &Context,
        n: NodeId,
        pending: &mut Pending,
        consume: bool,
        depth: usize,
    ) -> bool {
        let m = c.module;
        if !self.tick(self.span(m, n), depth) {
            return false;
        }
        match self.node(m, n) {
            NodeKind::Path { .. } => self.result.modules[m.0]
                .resolutions
                .get(&n.0)
                .copied()
                .is_some_and(|s| {
                    let owns = pending.contains(&s);
                    if consume {
                        pending.remove(&s);
                    }
                    owns
                }),
            NodeKind::Group { value } => {
                self.obligation_expression(c, value, pending, consume, depth + 1)
            }
            NodeKind::Tuple { elements } | NodeKind::Sequence { elements } => {
                let mut owns = false;
                for value in elements {
                    owns |= self.obligation_expression(c, value, pending, consume, depth + 1);
                }
                owns
            }
            NodeKind::Construct { fields, .. } => {
                let mut owns = false;
                for field in fields {
                    if let NodeKind::FieldValue { value, .. } = self.node(m, field) {
                        owns |= self.obligation_expression(c, value, pending, consume, depth + 1);
                    }
                }
                owns
            }
            NodeKind::Call { callee, arguments } => {
                let mut transferred = false;
                for arg in arguments {
                    transferred |= self.obligation_expression(c, arg, pending, true, depth + 1);
                }
                if let NodeKind::Path { components } = self.node(m, callee) {
                    let parts = self.path(m, &components);
                    if parts == ["discard"] {
                        return false;
                    }
                    if parts.first().is_some_and(|p| p == "Result") {
                        return true;
                    }
                    if !self.result.modules[m.0].resolutions.contains_key(&callee.0) {
                        // Constructors preserve payload obligations; empty variants do not invent them.
                        return transferred;
                    }
                }
                self.owns_result(self.obligation_type(m, n), self.span(m, n))
            }
            NodeKind::Propagate { value } => {
                self.obligation_expression(c, value, pending, true, depth + 1);
                // Err is a function exit; all other owners still require handling.
                self.unhandled(m, n, pending);
                self.owns_result(self.obligation_type(m, n), self.span(m, n))
            }
            NodeKind::Unary { operand, .. } => {
                self.obligation_expression(c, operand, pending, false, depth + 1);
                false
            }
            NodeKind::Binary {
                operator,
                left,
                right,
            } => {
                self.obligation_expression(c, left, pending, false, depth + 1);
                if matches!(operator, Kind::AndAnd | Kind::OrOr) {
                    let before = pending.clone();
                    self.obligation_expression(c, right, pending, false, depth + 1);
                    pending.extend(before);
                } else {
                    self.obligation_expression(c, right, pending, false, depth + 1);
                }
                false
            }
            NodeKind::Member { receiver, .. } | NodeKind::Index { receiver, .. } => {
                self.obligation_expression(c, receiver, pending, false, depth + 1);
                if let NodeKind::Index { index, .. } = self.node(m, n) {
                    self.obligation_expression(c, index, pending, false, depth + 1);
                }
                // Partial moves are excluded by the candidate. Do not clear the whole owner.
                self.owns_result(self.obligation_type(m, n), self.span(m, n))
            }
            _ => false,
        }
    }
    fn obligation_statement(
        &mut self,
        c: &Context,
        n: NodeId,
        mut pending: Pending,
        depth: usize,
    ) -> Exits {
        let m = c.module;
        self.work = self.work.saturating_add(pending.len());
        if !self.tick(self.span(m, n), depth) {
            return Exits::default();
        }
        match self.node(m, n) {
            NodeKind::Block { statements } => {
                let mut locals = Pending::new();
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
                    let dropped = state.intersection(&locals).copied().collect();
                    self.unhandled(m, n, &dropped);
                    state.retain(|s| !locals.contains(s));
                }
                return exits;
            }
            NodeKind::Binding { value, .. } => {
                let owns = self.obligation_expression(c, value, &mut pending, true, depth + 1);
                if owns {
                    if let Some(s) = c.declarations.get(&n.0) {
                        pending.insert(*s);
                    }
                }
            }
            NodeKind::Assignment { place, value } => {
                let owns = self.obligation_expression(c, value, &mut pending, true, depth + 1);
                let mut target = place;
                while let NodeKind::Group { value } = self.node(m, target) {
                    target = value;
                }
                if let Some(s) = self.result.modules[m.0].resolutions.get(&target.0).copied() {
                    if pending.remove(&s) {
                        self.error(
                            "T003",
                            self.span(m, place),
                            "assignment overwrites an unhandled Result obligation",
                        );
                    }
                    if owns {
                        pending.insert(s);
                    }
                } else if owns {
                    self.error("T003", self.span(m, place), "Result obligation requires whole-owner transfer; partial storage is not tracked");
                }
            }
            NodeKind::Return { value } => {
                if let Some(value) = value {
                    self.obligation_expression(c, value, &mut pending, true, depth + 1);
                }
                self.unhandled(m, n, &pending);
                return Exits::default();
            }
            NodeKind::ExpressionStatement { value } => {
                if self.obligation_expression(c, value, &mut pending, true, depth + 1) {
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
                return self.obligation_loop(c, body, pending, false, depth + 1)
            }
            NodeKind::While { condition, body } => {
                self.obligation_expression(c, condition, &mut pending, false, depth + 1);
                let literal = self.obligation_boolean(m, condition);
                if literal != Some(false) {
                    return self.obligation_loop(
                        c,
                        body,
                        pending,
                        literal != Some(true),
                        depth + 1,
                    );
                }
            }
            NodeKind::For { iterable, body, .. } => {
                // Retain the aggregate owner: a break may leave unvisited Results.
                self.obligation_expression(c, iterable, &mut pending, false, depth + 1);
                if let Some(s) = c.declarations.get(&n.0) {
                    if self.owns_result(self.result.symbols[s.0].ty, self.span(m, n)) {
                        pending.insert(*s);
                    }
                }
                return self.obligation_loop(c, body, pending, true, depth + 1);
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
                        let mut state = pending.clone();
                        let mut bindings = Pending::new();
                        self.obligation_pattern(
                            c,
                            pattern,
                            self.obligation_type(m, subject),
                            owns,
                            &mut state,
                            &mut bindings,
                            depth + 1,
                        );
                        let mut arm = self.obligation_statement(c, body, state, depth + 1);
                        for state in [&mut arm.next, &mut arm.breaks, &mut arm.continues]
                            .into_iter()
                            .flatten()
                        {
                            let dropped = state.intersection(&bindings).copied().collect();
                            self.unhandled(m, body, &dropped);
                            state.retain(|s| !bindings.contains(s));
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
        owns: bool,
        pending: &mut Pending,
        bindings: &mut Pending,
        depth: usize,
    ) {
        let m = c.module;
        if !owns || !self.tick(self.span(m, n), depth) {
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
                    pending.insert(*s);
                    bindings.insert(*s);
                }
            }
            NodeKind::TuplePattern { fields } => {
                if let Some(Type::Tuple(types)) = self.result.types.get(ty).cloned() {
                    for (p, t) in fields.into_iter().zip(types) {
                        let nested = self.owns_result(t, self.span(m, p));
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
                for (p, t) in fields.into_iter().zip(types) {
                    let nested = self.owns_result(t, self.span(m, p));
                    self.obligation_pattern(c, p, t, nested, pending, bindings, depth + 1);
                }
            }
            _ => {}
        }
    }
    fn obligation_loop(
        &mut self,
        c: &Context,
        body: NodeId,
        initial: Pending,
        may_skip: bool,
        depth: usize,
    ) -> Exits {
        let mut header = initial;
        let mut exit = None;
        loop {
            if !self.tick(self.span(c.module, body), depth) {
                break;
            }
            let flow = self.obligation_statement(c, body, header.clone(), depth + 1);
            join(&mut exit, flow.breaks);
            let mut back = flow.next;
            join(&mut back, flow.continues);
            let mut next = header.clone();
            if let Some(back) = back {
                next.extend(back);
            }
            if next == header {
                break;
            }
            header = next;
        }
        if may_skip {
            join(&mut exit, Some(header));
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
