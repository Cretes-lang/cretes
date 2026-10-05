#![forbid(unsafe_code)]
//! Provisional Cretes frontend; syntax only, against the Phase 3 draft.
pub mod ast;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod source;
pub mod token;
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub source_bytes: usize,
    pub tokens: usize,
    pub nodes: usize,
    pub diagnostics: usize,
    pub nesting: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            source_bytes: 16 * 1024 * 1024,
            tokens: 1_000_000,
            nodes: 1_000_000,
            diagnostics: 100,
            nesting: 512,
        }
    }
}
#[derive(Debug)]
pub struct FrontendResult {
    pub ast: ast::Ast,
    pub tokens: Vec<token::Token>,
    pub trivia: Vec<token::Trivia>,
    pub diagnostics: Vec<diagnostic::Diagnostic>,
}
impl FrontendResult {
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
pub fn parse_source(source: &source::SourceFile, limits: Limits) -> FrontendResult {
    let limits = limits.normalized();
    let lex = lexer::lex(source, limits);
    let parsed = if source.text().is_ok() {
        parser::parse(source, &lex.tokens, limits)
    } else {
        parser::ParseResult {
            ast: ast::Ast::default(),
            diagnostics: vec![],
        }
    };
    let mut diagnostics = lex.diagnostics;
    diagnostics.extend(parsed.diagnostics);
    diagnostics.sort_by_key(|d| (d.primary.source, d.primary.start, d.code));
    diagnostics.truncate(limits.diagnostics.max(1));
    FrontendResult {
        ast: parsed.ast,
        tokens: lex.tokens,
        trivia: lex.trivia,
        diagnostics,
    }
}

impl Limits {
    pub(crate) fn normalized(mut self) -> Self {
        self.diagnostics = self.diagnostics.clamp(1, 1000);
        self.nesting = self.nesting.clamp(1, 512);
        self
    }
}
