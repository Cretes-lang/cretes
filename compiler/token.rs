use crate::source::Span;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Ident,
    Int,
    Float,
    String,
    Char,
    Bytes,
    Wildcard,
    Invalid,
    Eof,
    As,
    Break,
    Const,
    Continue,
    Else,
    Enum,
    False,
    Fn,
    For,
    From,
    If,
    Import,
    In,
    Let,
    Loop,
    Match,
    Module,
    Mut,
    New,
    Pub,
    Return,
    Struct,
    True,
    Type,
    Var,
    While,
    ColonColon,
    Arrow,
    FatArrow,
    EqEq,
    NotEq,
    Le,
    Ge,
    Shl,
    Shr,
    AndAnd,
    OrOr,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,
    Pipe,
    Caret,
    Tilde,
    Bang,
    Eq,
    Lt,
    Gt,
    Question,
    Dot,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Colon,
}
pub const KEYWORDS: &[(&str, Kind)] = &[
    ("as", Kind::As),
    ("break", Kind::Break),
    ("const", Kind::Const),
    ("continue", Kind::Continue),
    ("else", Kind::Else),
    ("enum", Kind::Enum),
    ("false", Kind::False),
    ("fn", Kind::Fn),
    ("for", Kind::For),
    ("from", Kind::From),
    ("if", Kind::If),
    ("import", Kind::Import),
    ("in", Kind::In),
    ("let", Kind::Let),
    ("loop", Kind::Loop),
    ("match", Kind::Match),
    ("module", Kind::Module),
    ("mut", Kind::Mut),
    ("new", Kind::New),
    ("pub", Kind::Pub),
    ("return", Kind::Return),
    ("struct", Kind::Struct),
    ("true", Kind::True),
    ("type", Kind::Type),
    ("var", Kind::Var),
    ("while", Kind::While),
];
pub const PUNCTUATION: &[(&str, Kind)] = &[
    ("::", Kind::ColonColon),
    ("->", Kind::Arrow),
    ("=>", Kind::FatArrow),
    ("==", Kind::EqEq),
    ("!=", Kind::NotEq),
    ("<=", Kind::Le),
    (">=", Kind::Ge),
    ("<<", Kind::Shl),
    (">>", Kind::Shr),
    ("&&", Kind::AndAnd),
    ("||", Kind::OrOr),
    ("+", Kind::Plus),
    ("-", Kind::Minus),
    ("*", Kind::Star),
    ("/", Kind::Slash),
    ("%", Kind::Percent),
    ("&", Kind::Amp),
    ("|", Kind::Pipe),
    ("^", Kind::Caret),
    ("~", Kind::Tilde),
    ("!", Kind::Bang),
    ("=", Kind::Eq),
    ("<", Kind::Lt),
    (">", Kind::Gt),
    ("?", Kind::Question),
    (".", Kind::Dot),
    ("(", Kind::LParen),
    (")", Kind::RParen),
    ("[", Kind::LBracket),
    ("]", Kind::RBracket),
    ("{", Kind::LBrace),
    ("}", Kind::RBrace),
    (",", Kind::Comma),
    (";", Kind::Semi),
    (":", Kind::Colon),
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind,
    pub span: Span,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriviaKind {
    Whitespace,
    LineComment,
    DocComment,
    BlockComment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub span: Span,
}
/// Input is always terminated with EOF, even after a resource limit.
pub struct TokenStream<'a> {
    tokens: &'a [Token],
    position: usize,
}
impl<'a> TokenStream<'a> {
    pub fn new(tokens: &'a [Token]) -> Option<Self> {
        (tokens.last()?.kind == Kind::Eof).then_some(Self {
            tokens,
            position: 0,
        })
    }
    pub fn current(&self) -> Token {
        self.peek(0)
    }
    pub fn peek(&self, n: usize) -> Token {
        self.tokens[self.position.saturating_add(n).min(self.tokens.len() - 1)]
    }
    pub fn advance(&mut self) -> Token {
        let t = self.current();
        if t.kind != Kind::Eof {
            self.position += 1;
        }
        t
    }
    pub fn position(&self) -> usize {
        self.position
    }
}
