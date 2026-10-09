//! Provisional semantic analysis against the published Phase 3 candidate.
//! Inputs are an explicit logical module map. This layer never opens files or runs code.
mod constants;
mod entry;
pub use entry::analyze_entry;
mod obligations;
mod patterns;
mod provenance;
mod types;
mod validation;
use crate::{
    ast::{Ast, Name, NodeId, NodeKind},
    diagnostic::{Diagnostic, Label, Severity},
    source::{SourceFile, Span},
    Limits,
};
pub use constants::ConstantValue;
use std::collections::{BTreeMap, BTreeSet};
pub use types::{ModuleId, ScopeId, SymbolId, Type, TypeId, Types};

#[derive(Clone, Copy, Debug)]
pub struct SemanticOptions {
    pub target_pointer_bits: u8,
    pub limits: Limits,
    pub modules: usize,
    /// Total input bytes, including logical identities, before parsing.
    pub session_source_bytes: usize,
    /// Total retained syntax nodes across all modules.
    pub session_nodes: usize,
    pub work: usize,
}
impl Default for SemanticOptions {
    fn default() -> Self {
        Self {
            target_pointer_bits: 64,
            limits: Limits::default(),
            modules: 4096,
            session_source_bytes: 64 * 1024 * 1024,
            session_nodes: 1_000_000,
            work: 4_000_000,
        }
    }
}
pub struct ModuleInput<'a> {
    pub identity: &'a str,
    pub source: &'a SourceFile,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Constant,
    Record,
    Enum,
    Alias,
    Parameter,
    Local,
    Pattern,
}
#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub module: ModuleId,
    pub node: NodeId,
    pub span: Span,
    pub public: bool,
    pub mutable: bool,
    pub ty: TypeId,
    pub scope: ScopeId,
}
#[derive(Debug)]
pub struct Scope {
    pub parent: Option<ScopeId>,
    pub module: ModuleId,
    pub bindings: BTreeMap<String, SymbolId>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueCategory {
    Value,
    ReadOnlyPlace,
    WritablePlace,
}
#[derive(Debug)]
pub struct Module {
    pub identity: String,
    pub ast: Ast,
    pub imports: BTreeMap<String, ModuleId>,
    pub scope: ScopeId,
    pub node_types: BTreeMap<usize, TypeId>,
    pub resolutions: BTreeMap<usize, SymbolId>,
    pub value_categories: BTreeMap<usize, ValueCategory>,
}
#[derive(Debug)]
pub struct SemanticResult {
    pub modules: Vec<Module>,
    pub symbols: Vec<Symbol>,
    pub scopes: Vec<Scope>,
    pub types: Types,
    pub constants: BTreeMap<SymbolId, ConstantValue>,
    pub diagnostics: Vec<Diagnostic>,
    pub signatures: BTreeMap<SymbolId, Signature>,
    pub records: BTreeMap<SymbolId, Vec<Field>>,
    pub enums: BTreeMap<SymbolId, Vec<Variant>>,
}
impl SemanticResult {
    /// Name/type analysis only; does not certify the later move/loan safety stage.
    pub fn is_valid(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
    pub fn dump(&self) -> String {
        let mut s = String::new();
        for (id, t) in self.types.iter() {
            s.push_str(&format!("type {}: {t:?}\n", id.0));
        }
        for (i, v) in self.symbols.iter().enumerate() {
            s.push_str(&format!(
                "symbol {i}: {} {:?} type {} module {}\n",
                v.name, v.kind, v.ty.0, v.module.0
            ));
        }
        for (id, sig) in &self.signatures {
            s.push_str(&format!("function {}: {sig:?}\n", id.0));
        }
        for m in &self.modules {
            s.push_str(&format!("module {}\n", m.identity));
            for (n, t) in &m.node_types {
                s.push_str(&format!("  node {n}: type {}\n", t.0));
            }
            for (n, id) in &m.resolutions {
                s.push_str(&format!("  node {n}: symbol {}\n", id.0));
            }
            for (n, category) in &m.value_categories {
                s.push_str(&format!("  node {n}: {category:?}\n"));
            }
        }
        s
    }
}
/// Resolved function contract; `from` indexes the source reference parameter.
#[derive(Clone, Debug)]
pub struct Signature {
    pub parameters: Vec<TypeId>,
    pub result: TypeId,
    pub from: Option<usize>,
}
#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub ty: TypeId,
    pub public: bool,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct Variant {
    pub name: String,
    pub payload: Vec<TypeId>,
    pub span: Span,
}
struct Analyzer<'a> {
    sources: Vec<&'a SourceFile>,
    result: SemanticResult,
    options: SemanticOptions,
    states: Vec<u8>,
    constant_states: BTreeMap<SymbolId, u8>,
    work: usize,
    exhausted: bool,
}
/// Parse and analyze caller-supplied modules in deterministic identity order.
/// No manifest schema, standard library or filesystem search is invented here.
pub fn analyze(inputs: &[ModuleInput<'_>], options: SemanticOptions) -> SemanticResult {
    let mut a = Analyzer {
        sources: vec![],
        result: SemanticResult {
            modules: vec![],
            symbols: vec![],
            scopes: vec![],
            types: Types::default(),
            constants: BTreeMap::new(),
            diagnostics: vec![],
            signatures: BTreeMap::new(),
            records: BTreeMap::new(),
            enums: BTreeMap::new(),
        },
        options,
        states: vec![],
        constant_states: BTreeMap::new(),
        work: 0,
        exhausted: false,
    };
    // Reject the session before allocating the ordered index or any ASTs.
    // Caller-owned snapshots are outside this API's allocation boundary.
    let mut source_bytes = 0usize;
    for (index, input) in inputs.iter().enumerate() {
        let size = input.source.bytes().len().checked_add(input.identity.len());
        let total = size.and_then(|size| source_bytes.checked_add(size));
        if index >= options.modules || total.is_none_or(|n| n > options.session_source_bytes) {
            a.error(
                "R001",
                input.source.span(0, 0),
                "semantic session input budget exceeded",
            );
            return a.result;
        }
        source_bytes = total.unwrap_or(0);
    }
    let mut remaining_nodes = options.session_nodes;
    let mut ordered: Vec<_> = inputs.iter().collect();
    ordered.sort_by_key(|x| x.identity);
    let mut identities = BTreeMap::new();
    let mut folded = BTreeSet::new();
    let mut source_ids = BTreeSet::new();
    for input in ordered {
        let span = input.source.span(0, 0);
        if a.result.modules.len() >= options.modules {
            a.error("R001", span, "module limit exceeded");
            break;
        }
        if !valid_identity(input.identity)
            || identities.contains_key(input.identity)
            || !folded.insert(input.identity.to_ascii_lowercase())
            || !source_ids.insert(input.source.id)
        {
            a.error(
                "M001",
                span,
                "invalid, duplicate or case-colliding module identity/source snapshot",
            );
            continue;
        }
        if remaining_nodes == 0 {
            a.error("R001", span, "semantic session AST node budget exceeded");
            return a.result;
        }
        let mut limits = options.limits;
        limits.nodes = limits.nodes.min(remaining_nodes);
        let parsed = crate::parse_source(input.source, limits);
        remaining_nodes = remaining_nodes.saturating_sub(parsed.ast.nodes.len());
        let remaining = options
            .limits
            .diagnostics
            .clamp(1, 1000)
            .saturating_sub(a.result.diagnostics.len());
        a.result
            .diagnostics
            .extend(parsed.diagnostics.into_iter().take(remaining));
        let m = ModuleId(a.result.modules.len());
        let scope = ScopeId(a.result.scopes.len());
        a.result.scopes.push(Scope {
            parent: None,
            module: m,
            bindings: BTreeMap::new(),
        });
        a.result.modules.push(Module {
            identity: input.identity.into(),
            ast: parsed.ast,
            imports: BTreeMap::new(),
            scope,
            node_types: BTreeMap::new(),
            resolutions: BTreeMap::new(),
            value_categories: BTreeMap::new(),
        });
        a.sources.push(input.source);
        identities.insert(input.identity.to_string(), m);
    }
    if !matches!(options.target_pointer_bits, 32 | 64) {
        if let Some(s) = a.sources.first() {
            a.error(
                "T001",
                s.span(0, 0),
                "target pointer width must be explicitly 32 or 64",
            );
        }
        return a.result;
    }
    for i in 0..a.result.modules.len() {
        a.collect(ModuleId(i), &identities);
    }
    a.cycles();
    for i in 0..a.result.symbols.len() {
        a.resolve_symbol(SymbolId(i), 0);
    }
    a.check_layouts();
    for i in 0..a.result.symbols.len() {
        if a.result.symbols[i].kind == SymbolKind::Constant {
            a.check_constant(SymbolId(i), 0);
        }
    }
    for i in 0..a.result.symbols.len() {
        if a.result.symbols[i].kind == SymbolKind::Function {
            a.check_function(SymbolId(i));
        }
    }
    for i in 0..a.result.symbols.len() {
        if a.result.symbols[i].kind == SymbolKind::Function {
            a.check_return_provenance(SymbolId(i));
            a.check_result_obligations(SymbolId(i));
        }
    }
    a.validate_types();
    a.result
        .diagnostics
        .sort_by_key(|d| (d.primary.source, d.primary.start, d.code));
    a.result.diagnostics.dedup();
    a.result
        .diagnostics
        .truncate(options.limits.diagnostics.clamp(1, 1000));
    a.result
}
fn valid_identity(s: &str) -> bool {
    !s.is_empty()
        && s.split("::").all(|p| {
            !p.is_empty()
                && (p.as_bytes()[0].is_ascii_alphabetic() || p.as_bytes()[0] == b'_')
                && p != "_"
                && !p.starts_with("__")
                && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        })
}
impl Analyzer<'_> {
    fn error(&mut self, code: &'static str, span: Span, message: impl Into<String>) {
        if self.result.diagnostics.len() < self.options.limits.diagnostics.clamp(1, 1000) {
            let mut message = message.into();
            if message.len() > 2048 {
                let mut end = 2048;
                while !message.is_char_boundary(end) {
                    end -= 1;
                }
                message.truncate(end);
                message.push('…');
            }
            self.result.diagnostics.push(Diagnostic::error(
                code,
                span,
                message,
                "see the candidate language rule and related declaration",
            ));
        }
    }
    fn tick(&mut self, span: Span, depth: usize) -> bool {
        self.work = self.work.saturating_add(1);
        if self.work > self.options.work || depth > self.options.limits.nesting.min(256) {
            if !self.exhausted {
                self.error("R001", span, "semantic work or nesting limit exceeded");
                self.exhausted = true;
            }
            false
        } else {
            true
        }
    }
    fn node(&self, m: ModuleId, n: NodeId) -> NodeKind {
        self.result.modules[m.0].ast.nodes[n.0].kind.clone()
    }
    fn span(&self, m: ModuleId, n: NodeId) -> Span {
        self.result.modules[m.0].ast.nodes[n.0].span
    }
    fn spelling(&self, m: ModuleId, n: &Name) -> String {
        self.sources[m.0].slice(n.span).unwrap_or("").into()
    }
    fn path(&self, m: ModuleId, p: &[Name]) -> Vec<String> {
        p.iter().map(|n| self.spelling(m, n)).collect()
    }
    // Keep declaration attributes explicit at each insertion site.
    #[allow(clippy::too_many_arguments)]
    fn insert(
        &mut self,
        m: ModuleId,
        scope: ScopeId,
        node: NodeId,
        name: &Name,
        kind: SymbolKind,
        public: bool,
        mutable: bool,
    ) -> SymbolId {
        let text = self.spelling(m, name);
        let old = self.result.scopes[scope.0].bindings.get(&text).copied();
        if types::reserved(&text)
            || self.result.modules[m.0].imports.contains_key(&text)
            || old.is_some()
        {
            let diagnostic_count = self.result.diagnostics.len();
            self.error(
                "M001",
                name.span,
                format!("duplicate or reserved declaration `{text}`"),
            );
            if let Some(old) = old {
                if let Some(d) = self.result.diagnostics.get_mut(diagnostic_count) {
                    d.secondary.push(Label {
                        span: self.result.symbols[old.0].span,
                        message: "previous declaration".into(),
                    });
                }
            }
        }
        let id = SymbolId(self.result.symbols.len());
        self.result.symbols.push(Symbol {
            name: text.clone(),
            kind,
            module: m,
            node,
            span: name.span,
            public,
            mutable,
            ty: Types::ERROR,
            scope,
        });
        self.states.push(0);
        if old.is_none() {
            self.result.scopes[scope.0].bindings.insert(text, id);
        }
        id
    }
    fn collect(&mut self, m: ModuleId, identities: &BTreeMap<String, ModuleId>) {
        let Some(root) = self.result.modules[m.0].ast.root else {
            return;
        };
        let NodeKind::Program {
            module,
            imports,
            items,
        } = self.node(m, root)
        else {
            return;
        };
        if let Some(n) = module {
            if let NodeKind::Module { path } = self.node(m, n) {
                if self.path(m, &path).join("::") != self.result.modules[m.0].identity {
                    self.error(
                        "M001",
                        self.span(m, n),
                        "module declaration disagrees with supplied logical identity",
                    );
                }
            }
        }
        for n in imports {
            if let NodeKind::Import { path, alias } = self.node(m, n) {
                let parts = self.path(m, &path);
                let name = alias
                    .map(|a| self.spelling(m, &a))
                    .unwrap_or_else(|| parts.last().cloned().unwrap_or_default());
                if let Some(target) = identities.get(&parts.join("::")) {
                    if types::reserved(&name)
                        || self.result.modules[m.0]
                            .imports
                            .insert(name, *target)
                            .is_some()
                    {
                        self.error(
                            "M001",
                            self.span(m, n),
                            "duplicate or reserved import alias",
                        );
                    }
                } else {
                    self.error(
                        "M001",
                        self.span(m, n),
                        format!("module `{}` was not supplied", parts.join("::")),
                    );
                }
            }
        }
        for n in items {
            if let NodeKind::Item {
                public,
                declaration,
            } = self.node(m, n)
            {
                let (name, kind) = match self.node(m, declaration) {
                    NodeKind::Function { name, .. } => (name, SymbolKind::Function),
                    NodeKind::Constant { name, .. } => (name, SymbolKind::Constant),
                    NodeKind::Record { name, .. } => (name, SymbolKind::Record),
                    NodeKind::Enum { name, .. } => (name, SymbolKind::Enum),
                    NodeKind::Alias { name, .. } => (name, SymbolKind::Alias),
                    _ => continue,
                };
                self.insert(
                    m,
                    self.result.modules[m.0].scope,
                    declaration,
                    &name,
                    kind,
                    public,
                    false,
                );
            }
        }
    }
    fn cycles(&mut self) {
        let mut state = vec![0u8; self.result.modules.len()];
        for root in 0..state.len() {
            if state[root] != 0 {
                continue;
            }
            let mut stack = vec![(root, false)];
            while let Some((n, exit)) = stack.pop() {
                if exit {
                    state[n] = 2;
                    continue;
                }
                if state[n] == 2 {
                    continue;
                }
                if state[n] == 1 {
                    self.error("M001", self.sources[n].span(0, 0), "cyclic module imports");
                    continue;
                }
                state[n] = 1;
                stack.push((n, true));
                for target in self.result.modules[n].imports.values().rev() {
                    stack.push((target.0, false));
                }
            }
        }
    }
    fn lookup(
        &mut self,
        m: ModuleId,
        scope: ScopeId,
        parts: &[String],
        span: Span,
    ) -> Option<SymbolId> {
        let found = match parts {
            [name] => {
                let mut current = Some(scope);
                let mut found = None;
                while let Some(s) = current {
                    if let Some(id) = self.result.scopes[s.0].bindings.get(name) {
                        found = Some(*id);
                        break;
                    }
                    current = self.result.scopes[s.0].parent;
                }
                found
            }
            [alias, name] => self.result.modules[m.0]
                .imports
                .get(alias)
                .and_then(|m| {
                    self.result.scopes[self.result.modules[m.0].scope.0]
                        .bindings
                        .get(name)
                })
                .copied(),
            _ => None,
        };
        if let Some(id) = found {
            let s = &self.result.symbols[id.0];
            if s.module != m && !s.public {
                self.error("M001", span, format!("`{}` is private", parts.join("::")));
                None
            } else {
                Some(id)
            }
        } else {
            self.error(
                "M001",
                span,
                format!("cannot resolve `{}`", parts.join("::")),
            );
            None
        }
    }
    fn resolve_symbol(&mut self, id: SymbolId, depth: usize) -> TypeId {
        let m = self.result.symbols[id.0].module;
        let n = self.result.symbols[id.0].node;
        if !self.tick(self.span(m, n), depth) {
            return Types::ERROR;
        }
        if self.states[id.0] == 2 {
            return self.result.symbols[id.0].ty;
        }
        if self.states[id.0] == 1 {
            self.error("T001", self.span(m, n), "cyclic type alias or declaration");
            return Types::ERROR;
        }
        self.states[id.0] = 1;
        let scope = self.result.modules[m.0].scope;
        let ty = match self.node(m, n) {
            NodeKind::Alias { ty, .. } | NodeKind::Constant { ty, .. } => {
                self.resolve_type(m, scope, ty, depth + 1)
            }
            NodeKind::Record { fields, .. } => {
                let ty = self.result.types.intern(Type::Nominal(id));
                self.result.symbols[id.0].ty = ty;
                self.states[id.0] = 2;
                let mut names = BTreeSet::new();
                let mut fs = vec![];
                for f in fields {
                    if let NodeKind::Field {
                        name,
                        ty: t,
                        public,
                    } = self.node(m, f)
                    {
                        let name_text = self.spelling(m, &name);
                        if !names.insert(name_text.clone()) {
                            self.error("M001", name.span, "duplicate record field");
                        }
                        let ft = self.resolve_type(m, scope, t, depth + 1);
                        fs.push(Field {
                            name: name_text,
                            ty: ft,
                            public,
                            span: name.span,
                        });
                    }
                }
                self.result.records.insert(id, fs);
                ty
            }
            NodeKind::Enum { variants, .. } => {
                let ty = self.result.types.intern(Type::Nominal(id));
                self.result.symbols[id.0].ty = ty;
                self.states[id.0] = 2;
                let mut names = BTreeSet::new();
                let mut vs = vec![];
                for v in variants {
                    if let NodeKind::Variant { name, payload } = self.node(m, v) {
                        let name_text = self.spelling(m, &name);
                        if !names.insert(name_text.clone()) {
                            self.error("M001", name.span, "duplicate enum variant");
                        }
                        let payload = payload
                            .into_iter()
                            .map(|t| self.resolve_type(m, scope, t, depth + 1))
                            .collect();
                        vs.push(Variant {
                            name: name_text,
                            payload,
                            span: name.span,
                        });
                    }
                }
                if vs.is_empty() {
                    self.error(
                        "T001",
                        self.span(m, n),
                        "enum requires at least one variant",
                    );
                }
                self.result.enums.insert(id, vs);
                ty
            }
            NodeKind::Function {
                parameters,
                result,
                from,
                ..
            } => {
                let mut ps = vec![];
                let mut names = vec![];
                for p in parameters {
                    if let NodeKind::Parameter { name, ty, .. } = self.node(m, p) {
                        names.push(self.spelling(m, &name));
                        ps.push(self.resolve_type(m, scope, ty, depth + 1));
                    }
                }
                let rt = self.resolve_type(m, scope, result, depth + 1);
                let provenance = from
                    .as_ref()
                    .and_then(|f| names.iter().position(|n| n == &self.spelling(m, f)));
                match self.result.types.get(rt).cloned() {
                    Some(Type::Reference { mutable, .. }) => {
                        let compatible = provenance
                            .and_then(|i| self.result.types.get(ps[i]))
                            .is_some_and(
                                |t| matches!(t,Type::Reference{mutable:p,..} if !mutable||*p),
                            );
                        if !compatible {
                            self.error(
                                "B002",
                                self.span(m, n),
                                "reference return requires a compatible `from` reference parameter",
                            );
                        }
                    }
                    _ => {
                        if from.is_some() {
                            self.error(
                                "B002",
                                self.span(m, n),
                                "`from` is only valid on direct reference returns",
                            );
                        }
                    }
                }
                self.result.signatures.insert(
                    id,
                    Signature {
                        parameters: ps,
                        result: rt,
                        from: provenance,
                    },
                );
                rt
            }
            _ => Types::ERROR,
        };
        self.result.symbols[id.0].ty = ty;
        self.states[id.0] = 2;
        ty
    }
    fn resolve_type(&mut self, m: ModuleId, scope: ScopeId, n: NodeId, depth: usize) -> TypeId {
        if !self.tick(self.span(m, n), depth) {
            return Types::ERROR;
        }
        let ty = match self.node(m, n) {
            NodeKind::NamedType { path, arguments } => {
                let parts = self.path(m, &path);
                let args: Vec<_> = arguments
                    .into_iter()
                    .map(|n| self.resolve_type(m, scope, n, depth + 1))
                    .collect();
                if parts.len() == 1 {
                    if let Some(t) = self.result.types.primitive(&parts[0]) {
                        if !args.is_empty() {
                            self.error(
                                "T001",
                                self.span(m, n),
                                "primitive type takes no arguments",
                            );
                        }
                        t
                    } else if let Some(arity) = types::arity(&parts[0]) {
                        if args.len() != arity {
                            self.error("T001", self.span(m, n), "wrong built-in type arity");
                            Types::ERROR
                        } else {
                            self.result.types.intern(Type::Builtin {
                                name: parts[0].clone(),
                                arguments: args,
                            })
                        }
                    } else {
                        self.named_type(m, scope, n, &parts, &args, depth)
                    }
                } else {
                    self.named_type(m, scope, n, &parts, &args, depth)
                }
            }
            NodeKind::TupleType { elements } => {
                let es = elements
                    .into_iter()
                    .map(|n| self.resolve_type(m, scope, n, depth + 1))
                    .collect();
                self.result.types.intern(Type::Tuple(es))
            }
            NodeKind::ReferenceType { mutable, target } => {
                let t = self.resolve_type(m, scope, target, depth + 1);
                if matches!(self.result.types.get(t), Some(Type::Reference { .. })) {
                    self.error(
                        "B002",
                        self.span(m, n),
                        "reference-to-reference types are excluded",
                    );
                }
                self.result
                    .types
                    .intern(Type::Reference { mutable, target: t })
            }
            _ => Types::ERROR,
        };
        self.result.modules[m.0].node_types.insert(n.0, ty);
        ty
    }
    fn named_type(
        &mut self,
        m: ModuleId,
        scope: ScopeId,
        n: NodeId,
        parts: &[String],
        args: &[TypeId],
        depth: usize,
    ) -> TypeId {
        if !args.is_empty() {
            self.error("T001", self.span(m, n), "user types take no type arguments");
        }
        if let Some(id) = self.lookup(m, scope, parts, self.span(m, n)) {
            if matches!(
                self.result.symbols[id.0].kind,
                SymbolKind::Alias | SymbolKind::Record | SymbolKind::Enum
            ) {
                self.result.modules[m.0].resolutions.insert(n.0, id);
                self.resolve_symbol(id, depth + 1)
            } else {
                self.error("T001", self.span(m, n), "name does not denote a type");
                Types::ERROR
            }
        } else {
            Types::ERROR
        }
    }
    fn check_layouts(&mut self) {
        for i in 0..self.result.symbols.len() {
            if !matches!(
                self.result.symbols[i].kind,
                SymbolKind::Record | SymbolKind::Enum
            ) {
                continue;
            }
            let root = self.result.symbols[i].ty;
            let span = self.result.symbols[i].span;
            let mut state = BTreeMap::new();
            let mut stack = vec![(root, false)];
            while let Some((t, exit)) = stack.pop() {
                if !self.tick(span, 0) {
                    break;
                }
                if exit {
                    state.insert(t, 2);
                    continue;
                }
                match state.get(&t) {
                    Some(2) => continue,
                    Some(1) => {
                        self.error(
                            "T001",
                            span,
                            "recursive by-value layout; use owning indirection",
                        );
                        break;
                    }
                    _ => {}
                }
                state.insert(t, 1);
                stack.push((t, true));
                match self.result.types.get(t).cloned() {
                    Some(Type::Reference { .. }) => {
                        self.error("B002", span, "aggregate storage cannot contain references")
                    }
                    Some(Type::Nominal(s)) => {
                        if let Some(fs) = self.result.records.get(&s) {
                            stack.extend(fs.iter().map(|f| (f.ty, false)));
                        }
                        if let Some(vs) = self.result.enums.get(&s) {
                            for v in vs {
                                stack.extend(v.payload.iter().map(|t| (*t, false)));
                            }
                        }
                    }
                    Some(Type::Tuple(ts)) => stack.extend(ts.into_iter().map(|t| (t, false))),
                    Some(Type::Builtin { name, arguments })
                        if name == "Option" || name == "Result" =>
                    {
                        stack.extend(arguments.into_iter().map(|t| (t, false)))
                    }
                    _ => {}
                }
            }
        }
    }
}
mod check;
