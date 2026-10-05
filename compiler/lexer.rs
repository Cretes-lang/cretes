//! Single forward scan; values retain their original spellings in source.
use crate::{
    diagnostic::Diagnostic,
    source::SourceFile,
    token::{Kind, Token, Trivia, TriviaKind, KEYWORDS, PUNCTUATION},
    Limits,
};
#[derive(Debug)]
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub trivia: Vec<Trivia>,
    pub diagnostics: Vec<Diagnostic>,
}
pub fn lex(source: &SourceFile, limits: Limits) -> LexResult {
    let limits = limits.normalized();
    let mut out = LexResult {
        tokens: vec![],
        trivia: vec![],
        diagnostics: vec![],
    };
    let eof = Token {
        kind: Kind::Eof,
        span: source.span(source.bytes().len(), source.bytes().len()),
    };
    if source.bytes().len() > limits.source_bytes {
        out.diagnostics.push(Diagnostic::error(
            "R001",
            source.span(0, 0),
            "source byte limit exceeded",
            "split the source or raise the configured source limit",
        ));
        out.tokens.push(eof);
        return out;
    }
    let text = match source.text() {
        Ok(t) => t,
        Err(e) => {
            out.diagnostics.push(Diagnostic::error(
                "L001",
                source.span(
                    e.valid_up_to(),
                    (e.valid_up_to() + e.error_len().unwrap_or(1)).min(source.bytes().len()),
                ),
                "invalid UTF-8",
                "save the original source as strict UTF-8",
            ));
            out.tokens.push(eof);
            return out;
        }
    };
    let mut l = Lexer {
        source,
        text,
        pos: 0,
        out,
        limits,
    };
    // This pass deliberately also checks comments and literals.
    for (i, c) in text.char_indices() {
        let n = c as u32;
        if (n < 32 && !matches!(c, '\t' | '\n' | '\r'))
            || (127..=159).contains(&n)
            || matches!(
                n,
                0x61c | 0x200e | 0x200f | 0x2060 | 0xfeff | 0x2028 | 0x2029
            )
            || (0x200b..=0x200d).contains(&n)
            || (0x202a..=0x202e).contains(&n)
            || (0x2066..=0x2069).contains(&n)
            || (c == '\r' && text.as_bytes().get(i + 1) != Some(&b'\n'))
        {
            l.error(
                "L001",
                i,
                i + c.len_utf8(),
                "prohibited raw source scalar or bare CR",
                "remove the raw control; use a permitted literal escape for intentional data",
            );
        }
    }
    while l.pos < text.len() {
        if l.out.tokens.len() + l.out.trivia.len() >= limits.tokens {
            l.error(
                "R001",
                l.pos,
                l.pos,
                "token/trivia limit exceeded",
                "split the source or raise the configured token limit",
            );
            break;
        }
        let start = l.pos;
        let c = l.ch();
        if matches!(c, ' ' | '\t' | '\r' | '\n') {
            l.bump();
            while l.pos < text.len() && matches!(l.ch(), ' ' | '\t' | '\r' | '\n') {
                l.bump();
            }
            l.trivia(TriviaKind::Whitespace, start);
            continue;
        }
        if l.rest().starts_with("//") {
            let doc = l.rest().starts_with("///");
            while l.pos < text.len() && !matches!(l.ch(), '\r' | '\n') {
                l.bump();
            }
            l.trivia(
                if doc {
                    TriviaKind::DocComment
                } else {
                    TriviaKind::LineComment
                },
                start,
            );
            continue;
        }
        if l.rest().starts_with("/*") {
            l.pos += 2;
            let mut depth = 1usize;
            while l.pos < text.len() && depth > 0 {
                if l.rest().starts_with("/*") {
                    depth += 1;
                    l.pos += 2;
                } else if l.rest().starts_with("*/") {
                    depth -= 1;
                    l.pos += 2;
                } else {
                    l.bump();
                }
            }
            if depth != 0 {
                l.error(
                    "L005",
                    start,
                    start + 2,
                    "unterminated block comment",
                    "close each opening /* with */",
                );
            }
            l.trivia(TriviaKind::BlockComment, start);
            continue;
        }
        let byte = l.rest().starts_with("b\"");
        let kind = if byte || matches!(c, '"' | '\'') {
            l.literal(byte)
        } else if c.is_ascii_digit() {
            l.number()
        } else if c.is_ascii_alphabetic() || c == '_' {
            l.bump();
            while l.pos < text.len() && (l.ch().is_ascii_alphanumeric() || l.ch() == '_') {
                l.bump();
            }
            let word = &text[start..l.pos];
            if word.starts_with("__") {
                l.error(
                    "L002",
                    start,
                    l.pos,
                    "reserved internal identifier",
                    "choose a name that does not start with two underscores",
                );
                Kind::Invalid
            } else if word == "_" {
                Kind::Wildcard
            } else {
                KEYWORDS
                    .iter()
                    .find(|(s, _)| *s == word)
                    .map(|(_, k)| *k)
                    .unwrap_or(Kind::Ident)
            }
        } else if let Some((s, k)) = PUNCTUATION.iter().find(|(s, _)| l.rest().starts_with(s)) {
            l.pos += s.len();
            *k
        } else {
            l.bump();
            l.error(
                "L002",
                start,
                l.pos,
                "unrecognized source character",
                "identifiers use ASCII letters, digits and underscore; check punctuation",
            );
            Kind::Invalid
        };
        l.out.tokens.push(Token {
            kind,
            span: source.span(start, l.pos),
        });
    }
    l.out.tokens.push(eof);
    l.out
}
struct Lexer<'a> {
    source: &'a SourceFile,
    text: &'a str,
    pos: usize,
    out: LexResult,
    limits: Limits,
}
impl Lexer<'_> {
    fn rest(&self) -> &str {
        &self.text[self.pos..]
    }
    fn ch(&self) -> char {
        self.rest().chars().next().unwrap_or('\0')
    }
    fn bump(&mut self) {
        self.pos += self.ch().len_utf8();
    }
    fn trivia(&mut self, kind: TriviaKind, start: usize) {
        self.out.trivia.push(Trivia {
            kind,
            span: self.source.span(start, self.pos),
        });
    }
    fn error(&mut self, code: &'static str, start: usize, end: usize, message: &str, help: &str) {
        if self.out.diagnostics.len() < self.limits.diagnostics {
            self.out.diagnostics.push(Diagnostic::error(
                code,
                self.source.span(start, end),
                message,
                help,
            ));
        }
    }
    fn digits(&mut self, base: u32) -> bool {
        let mut any = false;
        let mut last = false;
        let mut valid = true;
        while self.pos < self.text.len() {
            let c = self.ch();
            if c.is_ascii() && c.is_digit(base) {
                any = true;
                last = true;
                self.bump();
            } else if c == '_' {
                if !last {
                    valid = false;
                }
                last = false;
                self.bump();
            } else {
                break;
            }
        }
        any && last && valid
    }
    fn number(&mut self) -> Kind {
        let start = self.pos;
        let mut float = false;
        let base = if self.rest().starts_with("0b") {
            2
        } else if self.rest().starts_with("0o") {
            8
        } else if self.rest().starts_with("0x") {
            16
        } else {
            10
        };
        if base != 10 {
            self.pos += 2;
        }
        let mut valid = self.digits(base);
        if base == 10 {
            if self.ch() == '.'
                && self
                    .text
                    .as_bytes()
                    .get(self.pos + 1)
                    .is_some_and(u8::is_ascii_digit)
            {
                float = true;
                self.pos += 1;
                valid = self.digits(10) && valid;
            }
            if matches!(self.ch(), 'e' | 'E') {
                float = true;
                self.pos += 1;
                if matches!(self.ch(), '+' | '-') {
                    self.pos += 1;
                }
                valid = self.digits(10) && valid;
            }
        }
        if self.pos < self.text.len() && (self.ch().is_alphanumeric() || self.ch() == '_') {
            valid = false;
            while self.pos < self.text.len() && (self.ch().is_alphanumeric() || self.ch() == '_') {
                self.bump();
            }
        }
        if !valid {
            self.error(
                "L003",
                start,
                self.pos,
                "malformed numeric literal",
                "check radix digits, separators and exponent digits; suffixes are not supported",
            );
            Kind::Invalid
        } else if float {
            Kind::Float
        } else {
            Kind::Int
        }
    }
    fn literal(&mut self, byte: bool) -> Kind {
        let start = self.pos;
        if byte {
            self.pos += 1;
        }
        let quote = self.ch();
        self.bump();
        let mut count = 0;
        let mut valid = true;
        let mut closed = false;
        while self.pos < self.text.len() {
            let c = self.ch();
            if c == quote {
                self.bump();
                closed = true;
                break;
            }
            if matches!(c, '\n' | '\r') {
                valid = false;
                break;
            }
            if c == '\t' {
                valid = false;
            }
            self.bump();
            if c == '\\' {
                if self.pos == self.text.len() {
                    valid = false;
                    break;
                }
                let esc = self.ch();
                self.bump();
                match esc {
                    'n' | 'r' | 't' | '0' | '\\' | '"' | '\'' => {}
                    'x' if byte => {
                        for _ in 0..2 {
                            if self.pos < self.text.len() && self.ch().is_ascii_hexdigit() {
                                self.bump();
                            } else {
                                valid = false;
                                break;
                            }
                        }
                    }
                    'u' if !byte => {
                        if self.ch() != '{' {
                            valid = false;
                        } else {
                            self.bump();
                            let mut value = 0u32;
                            let mut digits = 0;
                            while self.pos < self.text.len() && self.ch().is_ascii_hexdigit() {
                                if digits < 6 {
                                    value = value * 16 + self.ch().to_digit(16).unwrap_or(0);
                                }
                                digits += 1;
                                self.bump();
                            }
                            if self.ch() == '}' {
                                self.bump();
                            } else {
                                valid = false;
                            }
                            if !(1..=6).contains(&digits) || char::from_u32(value).is_none() {
                                valid = false;
                            }
                        }
                    }
                    _ => valid = false,
                }
            } else if byte && !(' '..='~').contains(&c) {
                valid = false;
            }
            count += 1;
        }
        if !closed || quote == '\'' && count != 1 {
            valid = false;
        }
        if !valid {
            self.error(
                "L004",
                start,
                self.pos,
                "invalid or unterminated literal",
                "close the delimiter; use supported escapes and exactly one scalar for a character",
            );
            Kind::Invalid
        } else if byte {
            Kind::Bytes
        } else if quote == '\'' {
            Kind::Char
        } else {
            Kind::String
        }
    }
}
