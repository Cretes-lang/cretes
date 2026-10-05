//! Keyword-dispatched recursive descent with one precedence table.
use crate::{
    ast::{Ast, Name, Node, NodeId, NodeKind as N},
    diagnostic::{Diagnostic, Label},
    source::{SourceFile, Span},
    token::{Kind as K, Token, TokenStream, KEYWORDS},
    Limits,
};
#[derive(Debug)]
pub struct ParseResult {
    pub ast: Ast,
    pub diagnostics: Vec<Diagnostic>,
}
#[derive(Clone, Copy)]
struct Failure;
type P<T> = Result<T, Failure>;
pub fn parse(source: &SourceFile, tokens: &[Token], limits: Limits) -> ParseResult {
    let limits = limits.normalized();
    if source.bytes().len() > limits.source_bytes || tokens.len().saturating_sub(1) > limits.tokens
    {
        return ParseResult {
            ast: Ast::default(),
            diagnostics: vec![Diagnostic::error(
                "R001",
                source.span(0, 0),
                "parser input budget exceeded",
                "reduce input or raise the configured source/token limit",
            )],
        };
    }
    let Some(stream) = TokenStream::new(tokens) else {
        return ParseResult {
            ast: Ast::default(),
            diagnostics: vec![Diagnostic::error(
                "P001",
                source.span(0, 0),
                "token stream requires EOF",
                "use the lexer-produced token stream",
            )],
        };
    };
    // Public callers may supply tokens: reject ranges that cannot belong to this snapshot.
    if tokens[..tokens.len() - 1].iter().any(|t| t.kind == K::Eof)
        || tokens.windows(2).any(|w| w[0].span.end > w[1].span.start)
        || tokens.iter().any(|t| {
            t.span.source != source.id
                || t.span.start > t.span.end
                || t.span.end > source.bytes().len()
                || source.slice(t.span).is_none()
        })
    {
        return ParseResult {
            ast: Ast::default(),
            diagnostics: vec![Diagnostic::error(
                "P001",
                source.span(0, 0),
                "invalid token source range",
                "use tokens from this exact source snapshot",
            )],
        };
    }
    let mut p = Parser {
        source,
        stream,
        ast: Ast::default(),
        diagnostics: vec![],
        limits,
        depth: 0,
        last_end: 0,
        halt: false,
    };
    p.program();
    ParseResult {
        ast: p.ast,
        diagnostics: p.diagnostics,
    }
}
struct Parser<'a> {
    source: &'a SourceFile,
    stream: TokenStream<'a>,
    ast: Ast,
    diagnostics: Vec<Diagnostic>,
    limits: Limits,
    depth: usize,
    last_end: usize,
    halt: bool,
}
impl Parser<'_> {
    fn token(&self) -> Token {
        self.stream.current()
    }
    fn at(&self, k: K) -> bool {
        self.token().kind == k
    }
    fn bump(&mut self) -> Token {
        let t = self.stream.advance();
        self.last_end = t.span.end;
        t
    }
    fn take(&mut self, k: K) -> bool {
        if self.at(k) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn error(&mut self, code: &'static str, message: impl Into<String>, help: &str) {
        if self.diagnostics.len() < self.limits.diagnostics {
            self.diagnostics
                .push(Diagnostic::error(code, self.token().span, message, help));
        } else {
            self.halt = true;
        }
    }
    fn expect(&mut self, k: K) -> P<Token> {
        if self.at(k) {
            Ok(self.bump())
        } else {
            self.error(
                "P001",
                format!("expected {k:?}, found {:?}", self.token().kind),
                "insert the expected token or correct the preceding construct",
            );
            Err(Failure)
        }
    }
    fn close(&mut self, k: K, opening: Span) -> P<Token> {
        match self.expect(k) {
            Ok(t) => Ok(t),
            Err(e) => {
                if let Some(d) = self.diagnostics.last_mut() {
                    d.secondary.push(Label {
                        span: opening,
                        message: "opening delimiter".into(),
                    });
                }
                Err(e)
            }
        }
    }
    fn name(&mut self) -> P<Name> {
        if self.at(K::Ident) {
            Ok(Name {
                span: self.bump().span,
            })
        } else {
            let code = if KEYWORDS.iter().any(|(_, k)| *k == self.token().kind) {
                "P002"
            } else {
                "P001"
            };
            self.error(
                code,
                "expected identifier",
                "use an ASCII identifier that is not a keyword or wildcard",
            );
            Err(Failure)
        }
    }
    fn path(&mut self) -> P<Vec<Name>> {
        let mut names = vec![self.name()?];
        while self.take(K::ColonColon) {
            names.push(self.name()?);
        }
        Ok(names)
    }
    fn add(&mut self, start: usize, kind: N) -> P<NodeId> {
        if self.ast.nodes.len() >= self.limits.nodes {
            self.error(
                "R001",
                "AST node limit exceeded",
                "split the source or raise the configured node limit",
            );
            self.halt = true;
            return Err(Failure);
        }
        let id = NodeId(self.ast.nodes.len());
        self.ast.nodes.push(Node {
            span: self.source.span(start, self.last_end.max(start)),
            kind,
        });
        Ok(id)
    }
    fn start(&self, id: NodeId) -> usize {
        self.ast.nodes[id.0].span.start
    }
    fn nested<T>(&mut self, f: impl FnOnce(&mut Self) -> P<T>) -> P<T> {
        if self.depth >= self.limits.nesting {
            self.error(
                "R001",
                "syntax nesting limit exceeded",
                "reduce nesting or raise the supported nesting limit",
            );
            self.halt = true;
            return Err(Failure);
        }
        self.depth += 1;
        let r = f(self);
        self.depth -= 1;
        r
    }
    fn item_start(&self) -> bool {
        matches!(
            self.token().kind,
            K::Pub | K::Fn | K::Const | K::Struct | K::Enum | K::Type
        )
    }
    fn recover(&mut self, start: usize, position: usize, top: bool) -> Option<NodeId> {
        // A boundary may be left for the caller only after some progress.
        if self.stream.position() == position && !self.at(K::Eof) && !self.halt {
            self.bump();
        }
        while !self.at(K::Eof) && !self.halt {
            if self.take(K::Semi) {
                break;
            }
            if self.at(K::RBrace) {
                if top {
                    self.bump();
                }
                break;
            }
            if self.item_start()
                || (!top
                    && matches!(
                        self.token().kind,
                        K::Let
                            | K::Var
                            | K::Return
                            | K::If
                            | K::While
                            | K::For
                            | K::Loop
                            | K::Match
                    ))
            {
                break;
            }
            self.bump();
        }
        self.add(start, N::Error).ok()
    }
    fn program(&mut self) {
        let mut module = None;
        let mut imports = vec![];
        let mut items = vec![];
        if self.at(K::Module) {
            let start = self.token().span.start;
            let pos = self.stream.position();
            module = match self.module() {
                Ok(n) => Some(n),
                Err(_) => self.recover(start, pos, true),
            };
        }
        while self.at(K::Import) && !self.halt {
            let start = self.token().span.start;
            let pos = self.stream.position();
            match self.import() {
                Ok(n) => imports.push(n),
                Err(_) => {
                    if let Some(n) = self.recover(start, pos, true) {
                        imports.push(n);
                    }
                }
            }
        }
        while !self.at(K::Eof) && !self.halt {
            let start = self.token().span.start;
            let pos = self.stream.position();
            match self.item() {
                Ok(n) => items.push(n),
                Err(_) => {
                    if let Some(n) = self.recover(start, pos, true) {
                        items.push(n);
                    }
                }
            }
        }
        if !self.halt {
            self.last_end = self.source.bytes().len();
            self.ast.root = self
                .add(
                    0,
                    N::Program {
                        module,
                        imports,
                        items,
                    },
                )
                .ok();
        }
    }
    fn module(&mut self) -> P<NodeId> {
        let s = self.bump().span.start;
        let path = self.path()?;
        self.expect(K::Semi)?;
        self.add(s, N::Module { path })
    }
    fn import(&mut self) -> P<NodeId> {
        let s = self.bump().span.start;
        let path = self.path()?;
        let alias = if self.take(K::As) {
            Some(self.name()?)
        } else {
            None
        };
        self.expect(K::Semi)?;
        self.add(s, N::Import { path, alias })
    }
    fn item(&mut self) -> P<NodeId> {
        let s = self.token().span.start;
        let public = self.take(K::Pub);
        let declaration = match self.token().kind {
            K::Fn => self.function()?,
            K::Const => self.constant()?,
            K::Struct => self.record()?,
            K::Enum => self.enumeration()?,
            K::Type => self.alias()?,
            _ => {
                self.error(
                    "P001",
                    "expected a module-level declaration",
                    "use fn, const, struct, enum or type; imports must precede items",
                );
                return Err(Failure);
            }
        };
        self.add(
            s,
            N::Item {
                public,
                declaration,
            },
        )
    }
    fn function(&mut self) -> P<NodeId> {
        let s = self.bump().span.start;
        let name = self.name()?;
        let open = self.expect(K::LParen)?.span;
        let parameters = self.list(K::RParen, true, Self::parameter)?;
        self.close(K::RParen, open)?;
        self.expect(K::Arrow)?;
        let result = self.ty()?;
        let from = if self.take(K::From) {
            Some(self.name()?)
        } else {
            None
        };
        let body = self.block()?;
        self.add(
            s,
            N::Function {
                name,
                parameters,
                result,
                from,
                body,
            },
        )
    }
    fn parameter(&mut self) -> P<NodeId> {
        let s = self.token().span.start;
        let mutable = self.take(K::Var);
        let name = self.name()?;
        self.expect(K::Colon)?;
        let ty = self.ty()?;
        self.add(s, N::Parameter { mutable, name, ty })
    }
    fn list(&mut self, end: K, empty: bool, f: fn(&mut Self) -> P<NodeId>) -> P<Vec<NodeId>> {
        let mut out = vec![];
        if empty && self.at(end) {
            return Ok(out);
        }
        out.push(f(self)?);
        while self.take(K::Comma) {
            if self.at(end) {
                break;
            }
            out.push(f(self)?);
        }
        Ok(out)
    }
    fn ty(&mut self) -> P<NodeId> {
        self.nested(Self::ty_inner)
    }
    fn ty_inner(&mut self) -> P<NodeId> {
        let s = self.token().span.start;
        if self.take(K::Amp) {
            let mutable = self.take(K::Mut);
            let target = self.ty()?;
            self.add(s, N::ReferenceType { mutable, target })
        } else if self.at(K::LParen) {
            let open = self.bump().span;
            let mut elements = vec![];
            if !self.at(K::RParen) {
                elements.push(self.ty()?);
                self.expect(K::Comma)?;
                if !self.at(K::RParen) {
                    elements.extend(self.list(K::RParen, false, Self::ty)?);
                }
            }
            self.close(K::RParen, open)?;
            self.add(s, N::TupleType { elements })
        } else {
            let path = self.path()?;
            let arguments = if self.at(K::LBracket) {
                let open = self.bump().span;
                let a = self.list(K::RBracket, false, Self::ty)?;
                self.close(K::RBracket, open)?;
                a
            } else {
                vec![]
            };
            self.add(s, N::NamedType { path, arguments })
        }
    }
    fn constant(&mut self) -> P<NodeId> {
        let s = self.bump().span.start;
        let name = self.name()?;
        self.expect(K::Colon)?;
        let ty = self.ty()?;
        self.expect(K::Eq)?;
        let value = self.expression()?;
        self.expect(K::Semi)?;
        self.add(s, N::Constant { name, ty, value })
    }
    fn alias(&mut self) -> P<NodeId> {
        let s = self.bump().span.start;
        let name = self.name()?;
        self.expect(K::Eq)?;
        let ty = self.ty()?;
        self.expect(K::Semi)?;
        self.add(s, N::Alias { name, ty })
    }
    fn record(&mut self) -> P<NodeId> {
        let s = self.bump().span.start;
        let name = self.name()?;
        let open = self.expect(K::LBrace)?.span;
        let mut fields = vec![];
        while !self.at(K::RBrace) && !self.at(K::Eof) {
            let a = self.token().span.start;
            let public = self.take(K::Pub);
            let name = self.name()?;
            self.expect(K::Colon)?;
            let ty = self.ty()?;
            self.expect(K::Comma)?;
            fields.push(self.add(a, N::Field { public, name, ty })?);
        }
        self.close(K::RBrace, open)?;
        self.add(s, N::Record { name, fields })
    }
    fn enumeration(&mut self) -> P<NodeId> {
        let s = self.bump().span.start;
        let name = self.name()?;
        let open = self.expect(K::LBrace)?.span;
        let variants = self.list(K::RBrace, false, Self::variant)?;
        self.close(K::RBrace, open)?;
        self.add(s, N::Enum { name, variants })
    }
    fn variant(&mut self) -> P<NodeId> {
        let s = self.token().span.start;
        let name = self.name()?;
        let payload = if self.at(K::LParen) {
            let open = self.bump().span;
            let p = self.list(K::RParen, false, Self::ty)?;
            self.close(K::RParen, open)?;
            p
        } else {
            vec![]
        };
        self.add(s, N::Variant { name, payload })
    }
    fn block(&mut self) -> P<NodeId> {
        self.nested(Self::block_inner)
    }
    fn block_inner(&mut self) -> P<NodeId> {
        let open = self.expect(K::LBrace)?.span;
        let mut statements = vec![];
        while !self.at(K::RBrace) && !self.at(K::Eof) && !self.halt {
            let s = self.token().span.start;
            let pos = self.stream.position();
            match self.statement() {
                Ok(n) => statements.push(n),
                Err(_) => {
                    if let Some(n) = self.recover(s, pos, false) {
                        statements.push(n);
                    }
                }
            }
        }
        self.close(K::RBrace, open)?;
        self.add(open.start, N::Block { statements })
    }
    fn statement(&mut self) -> P<NodeId> {
        let s = self.token().span.start;
        match self.token().kind {
            K::Let | K::Var => {
                let mutable = self.bump().kind == K::Var;
                let name = self.name()?;
                let ty = if self.take(K::Colon) {
                    Some(self.ty()?)
                } else {
                    None
                };
                self.expect(K::Eq)?;
                let value = self.expression()?;
                self.expect(K::Semi)?;
                self.add(
                    s,
                    N::Binding {
                        mutable,
                        name,
                        ty,
                        value,
                    },
                )
            }
            K::Const => self.constant(),
            K::LBrace => self.block(),
            K::If => self.if_stmt(),
            K::Return => {
                self.bump();
                let value = if self.at(K::Semi) {
                    None
                } else {
                    Some(self.expression()?)
                };
                self.expect(K::Semi)?;
                self.add(s, N::Return { value })
            }
            K::Break | K::Continue => {
                let k = self.bump().kind;
                self.expect(K::Semi)?;
                self.add(s, if k == K::Break { N::Break } else { N::Continue })
            }
            K::While => {
                self.bump();
                let condition = self.expression()?;
                let body = self.block()?;
                self.add(s, N::While { condition, body })
            }
            K::For => {
                self.bump();
                let name = self.name()?;
                self.expect(K::In)?;
                let iterable = self.expression()?;
                let body = self.block()?;
                self.add(
                    s,
                    N::For {
                        name,
                        iterable,
                        body,
                    },
                )
            }
            K::Loop => {
                self.bump();
                let body = self.block()?;
                self.add(s, N::Loop { body })
            }
            K::Match => {
                self.bump();
                let subject = self.expression()?;
                let open = self.expect(K::LBrace)?.span;
                let mut arms = vec![];
                while !self.at(K::RBrace) && !self.at(K::Eof) {
                    let a = self.token().span.start;
                    let pattern = self.pattern()?;
                    self.expect(K::FatArrow)?;
                    let body = self.block()?;
                    arms.push(self.add(a, N::Arm { pattern, body })?);
                }
                self.close(K::RBrace, open)?;
                self.add(s, N::Match { subject, arms })
            }
            _ => {
                let value = self.expression()?;
                let kind = if self.take(K::Eq) {
                    let rhs = self.expression()?;
                    N::Assignment {
                        place: value,
                        value: rhs,
                    }
                } else {
                    N::ExpressionStatement { value }
                };
                self.expect(K::Semi)?;
                self.add(s, kind)
            }
        }
    }
    fn if_stmt(&mut self) -> P<NodeId> {
        self.nested(Self::if_inner)
    }
    fn if_inner(&mut self) -> P<NodeId> {
        let s = self.expect(K::If)?.span.start;
        let condition = self.expression()?;
        let then_block = self.block()?;
        let otherwise = if self.take(K::Else) {
            Some(if self.at(K::If) {
                self.if_stmt()?
            } else {
                self.block()?
            })
        } else {
            None
        };
        self.add(
            s,
            N::If {
                condition,
                then_block,
                otherwise,
            },
        )
    }
    fn pattern(&mut self) -> P<NodeId> {
        self.nested(Self::pattern_inner)
    }
    fn pattern_inner(&mut self) -> P<NodeId> {
        let s = self.token().span.start;
        if self.take(K::Wildcard) {
            self.add(s, N::WildcardPattern)
        } else if literal(self.token().kind) {
            let kind = self.bump().kind;
            self.add(s, N::LiteralPattern { kind })
        } else if self.at(K::LParen) {
            let open = self.bump().span;
            let mut fields = vec![self.pattern()?];
            self.expect(K::Comma)?;
            if !self.at(K::RParen) {
                fields.extend(self.list(K::RParen, false, Self::pattern)?);
            }
            self.close(K::RParen, open)?;
            self.add(s, N::TuplePattern { fields })
        } else {
            let path = self.path()?;
            if self.at(K::LParen) {
                let open = self.bump().span;
                let fields = self.list(K::RParen, true, Self::pattern)?;
                self.close(K::RParen, open)?;
                self.add(s, N::ConstructorPattern { path, fields })
            } else if path.len() == 1 {
                self.add(
                    s,
                    N::BindingPattern {
                        name: path[0].clone(),
                    },
                )
            } else {
                self.error(
                    "P001",
                    "constructor pattern requires parentheses",
                    "write qualified variant patterns with parentheses, including nullary variants",
                );
                Err(Failure)
            }
        }
    }
    fn expression(&mut self) -> P<NodeId> {
        self.binary(1)
    }
    fn binary(&mut self, min: u8) -> P<NodeId> {
        self.nested(|p| p.binary_inner(min))
    }
    fn binary_inner(&mut self, min: u8) -> P<NodeId> {
        let mut left = self.unary()?;
        let mut seen_nonassoc = 0u16;
        while let Some((prec, nonassoc)) = precedence(self.token().kind) {
            if prec < min {
                break;
            }
            if nonassoc && seen_nonassoc & (1 << prec) != 0 {
                self.error(
                    "P003",
                    "chained comparison or equality",
                    "write separate comparisons joined with &&, or explicitly parenthesize",
                );
                return Err(Failure);
            }
            if nonassoc {
                seen_nonassoc |= 1 << prec;
            }
            let operator = self.bump().kind;
            let right = self.binary(prec + 1)?;
            left = self.add(
                self.start(left),
                N::Binary {
                    operator,
                    left,
                    right,
                },
            )?;
        }
        Ok(left)
    }
    fn unary(&mut self) -> P<NodeId> {
        self.nested(Self::unary_inner)
    }
    fn unary_inner(&mut self) -> P<NodeId> {
        if matches!(
            self.token().kind,
            K::Minus | K::Bang | K::Tilde | K::Star | K::Amp
        ) {
            let t = self.bump();
            let mutable = t.kind == K::Amp && self.take(K::Mut);
            let operand = self.unary()?;
            self.add(
                t.span.start,
                N::Unary {
                    operator: t.kind,
                    mutable,
                    operand,
                },
            )
        } else {
            self.postfix()
        }
    }
    fn postfix(&mut self) -> P<NodeId> {
        let mut value = self.primary()?;
        loop {
            let s = self.start(value);
            value = match self.token().kind {
                K::LParen => {
                    let open = self.bump().span;
                    let arguments = self.list(K::RParen, true, Self::expression)?;
                    self.close(K::RParen, open)?;
                    self.add(
                        s,
                        N::Call {
                            callee: value,
                            arguments,
                        },
                    )?
                }
                K::LBracket => {
                    let open = self.bump().span;
                    let index = self.expression()?;
                    self.close(K::RBracket, open)?;
                    self.add(
                        s,
                        N::Index {
                            receiver: value,
                            index,
                        },
                    )?
                }
                K::Dot => {
                    self.bump();
                    let name = self.name()?;
                    self.add(
                        s,
                        N::Member {
                            receiver: value,
                            name,
                        },
                    )?
                }
                K::Question => {
                    self.bump();
                    self.add(s, N::Propagate { value })?
                }
                _ => break,
            };
        }
        Ok(value)
    }
    fn primary(&mut self) -> P<NodeId> {
        let s = self.token().span.start;
        let kind = self.token().kind;
        if literal(kind) {
            self.bump();
            return self.add(s, N::Literal { kind });
        }
        match kind {
            K::Ident => {
                let components = self.path()?;
                self.add(s, N::Path { components })
            }
            K::LParen => {
                let open = self.bump().span;
                if self.take(K::RParen) {
                    return self.add(s, N::Tuple { elements: vec![] });
                }
                let value = self.expression()?;
                if self.take(K::Comma) {
                    let mut elements = vec![value];
                    if !self.at(K::RParen) {
                        elements.extend(self.list(K::RParen, false, Self::expression)?);
                    }
                    self.close(K::RParen, open)?;
                    self.add(s, N::Tuple { elements })
                } else {
                    self.close(K::RParen, open)?;
                    self.add(s, N::Group { value })
                }
            }
            K::LBracket => {
                let open = self.bump().span;
                let elements = self.list(K::RBracket, true, Self::expression)?;
                self.close(K::RBracket, open)?;
                self.add(s, N::Sequence { elements })
            }
            K::New => {
                self.bump();
                let path = self.path()?;
                let open = self.expect(K::LBrace)?.span;
                let fields = self.list(K::RBrace, true, Self::field_value)?;
                self.close(K::RBrace, open)?;
                self.add(s, N::Construct { path, fields })
            }
            _ => {
                self.error(
                    "P001",
                    format!("expected expression, found {kind:?}"),
                    "supply a literal, name, unary operand or parenthesized expression",
                );
                Err(Failure)
            }
        }
    }
    fn field_value(&mut self) -> P<NodeId> {
        let s = self.token().span.start;
        let name = self.name()?;
        self.expect(K::Colon)?;
        let value = self.expression()?;
        self.add(s, N::FieldValue { name, value })
    }
}
fn literal(k: K) -> bool {
    matches!(
        k,
        K::Int | K::Float | K::String | K::Char | K::Bytes | K::True | K::False
    )
}
/// Precedence matches SPEC §3.10; comparisons/equality are non-associative.
pub fn precedence(k: K) -> Option<(u8, bool)> {
    Some(match k {
        K::OrOr => (1, false),
        K::AndAnd => (2, false),
        K::Pipe => (3, false),
        K::Caret => (4, false),
        K::Amp => (5, false),
        K::EqEq | K::NotEq => (6, true),
        K::Lt | K::Le | K::Gt | K::Ge => (7, true),
        K::Shl | K::Shr => (8, false),
        K::Plus | K::Minus => (9, false),
        K::Star | K::Slash | K::Percent => (10, false),
        _ => return None,
    })
}
