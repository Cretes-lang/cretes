//! A flat arena prevents recursive destruction of long operator chains.
use crate::{source::Span, token::Kind};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeId(pub usize);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name {
    pub span: Span,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Program {
        module: Option<NodeId>,
        imports: Vec<NodeId>,
        items: Vec<NodeId>,
    },
    Module {
        path: Vec<Name>,
    },
    Import {
        path: Vec<Name>,
        alias: Option<Name>,
    },
    Item {
        public: bool,
        declaration: NodeId,
    },
    Function {
        name: Name,
        parameters: Vec<NodeId>,
        result: NodeId,
        from: Option<Name>,
        body: NodeId,
    },
    Parameter {
        mutable: bool,
        name: Name,
        ty: NodeId,
    },
    Constant {
        name: Name,
        ty: NodeId,
        value: NodeId,
    },
    Alias {
        name: Name,
        ty: NodeId,
    },
    Record {
        name: Name,
        fields: Vec<NodeId>,
    },
    Field {
        public: bool,
        name: Name,
        ty: NodeId,
    },
    Enum {
        name: Name,
        variants: Vec<NodeId>,
    },
    Variant {
        name: Name,
        payload: Vec<NodeId>,
    },
    NamedType {
        path: Vec<Name>,
        arguments: Vec<NodeId>,
    },
    TupleType {
        elements: Vec<NodeId>,
    },
    ReferenceType {
        mutable: bool,
        target: NodeId,
    },
    Block {
        statements: Vec<NodeId>,
    },
    Binding {
        mutable: bool,
        name: Name,
        ty: Option<NodeId>,
        value: NodeId,
    },
    Return {
        value: Option<NodeId>,
    },
    If {
        condition: NodeId,
        then_block: NodeId,
        otherwise: Option<NodeId>,
    },
    While {
        condition: NodeId,
        body: NodeId,
    },
    For {
        name: Name,
        iterable: NodeId,
        body: NodeId,
    },
    Loop {
        body: NodeId,
    },
    Break,
    Continue,
    Match {
        subject: NodeId,
        arms: Vec<NodeId>,
    },
    Arm {
        pattern: NodeId,
        body: NodeId,
    },
    ExpressionStatement {
        value: NodeId,
    },
    Assignment {
        place: NodeId,
        value: NodeId,
    },
    Literal {
        kind: Kind,
    },
    Path {
        components: Vec<Name>,
    },
    Group {
        value: NodeId,
    },
    Tuple {
        elements: Vec<NodeId>,
    },
    Sequence {
        elements: Vec<NodeId>,
    },
    Construct {
        path: Vec<Name>,
        fields: Vec<NodeId>,
    },
    FieldValue {
        name: Name,
        value: NodeId,
    },
    Unary {
        operator: Kind,
        mutable: bool,
        operand: NodeId,
    },
    Binary {
        operator: Kind,
        left: NodeId,
        right: NodeId,
    },
    Call {
        callee: NodeId,
        arguments: Vec<NodeId>,
    },
    Member {
        receiver: NodeId,
        name: Name,
    },
    Index {
        receiver: NodeId,
        index: NodeId,
    },
    Propagate {
        value: NodeId,
    },
    WildcardPattern,
    BindingPattern {
        name: Name,
    },
    ConstructorPattern {
        path: Vec<Name>,
        fields: Vec<NodeId>,
    },
    TuplePattern {
        fields: Vec<NodeId>,
    },
    LiteralPattern {
        kind: Kind,
    },
    Error,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub span: Span,
    pub kind: NodeKind,
}
#[derive(Debug, Default)]
pub struct Ast {
    pub nodes: Vec<Node>,
    pub root: Option<NodeId>,
}
impl Ast {
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)
    }
    /// Deterministic arena order; references are local node IDs, not pointers.
    pub fn dump(&self) -> String {
        let mut out = String::new();
        for (i, node) in self.nodes.iter().enumerate() {
            out.push_str(&format!(
                "{i} @{}:{}..{} {:?}\n",
                node.span.source.0, node.span.start, node.span.end, node.kind
            ));
        }
        out
    }
}
impl Ast {
    /// Documentation comments adjacent to an item/field, without a blank line.
    pub fn documentation(
        &self,
        id: NodeId,
        source: &crate::source::SourceFile,
        trivia: &[crate::token::Trivia],
    ) -> Vec<Span> {
        use crate::token::TriviaKind;
        let Some(node) = self.get(id) else {
            return vec![];
        };
        if !matches!(node.kind, NodeKind::Item { .. } | NodeKind::Field { .. }) {
            return vec![];
        }
        let mut end = node.span.start;
        let mut docs = vec![];
        for t in trivia
            .iter()
            .rev()
            .filter(|t| t.span.end <= node.span.start)
        {
            if t.span.end != end {
                break;
            }
            match t.kind {
                TriviaKind::Whitespace => {
                    if source
                        .slice(t.span)
                        .unwrap_or("")
                        .bytes()
                        .filter(|b| *b == b'\n')
                        .count()
                        > 1
                    {
                        break;
                    }
                }
                TriviaKind::DocComment => docs.push(t.span),
                _ => break,
            }
            end = t.span.start;
        }
        docs.reverse();
        docs
    }
}
