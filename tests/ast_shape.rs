use cretes_frontend::{
    ast::{Ast, NodeId, NodeKind as N},
    parse_source,
    source::SourceManager,
    token::Kind as K,
    Limits,
};
fn parse(text: &str) -> Ast {
    let mut sources = SourceManager::default();
    let id = sources.add("shape.cretes", text.as_bytes());
    let result = parse_source(sources.get(id).unwrap(), Limits::default());
    assert!(result.is_valid(), "{:?}", result.diagnostics);
    result.ast
}
fn expression(text: &str) -> (Ast, NodeId) {
    let ast = parse(&format!("fn f() -> i64 {{ return {text}; }}"));
    let value = ast
        .nodes
        .iter()
        .find_map(|n| {
            if let N::Return { value } = n.kind {
                value
            } else {
                None
            }
        })
        .unwrap();
    (ast, value)
}
fn binary(ast: &Ast, id: NodeId, op: K) -> (NodeId, NodeId) {
    match ast.get(id).unwrap().kind {
        N::Binary {
            operator,
            left,
            right,
        } => {
            assert_eq!(operator, op);
            (left, right)
        }
        ref other => panic!("expected binary, got {other:?}"),
    }
}
#[test]
fn all_binary_precedence_boundaries() {
    let ops = [
        ("||", K::OrOr),
        ("&&", K::AndAnd),
        ("|", K::Pipe),
        ("^", K::Caret),
        ("&", K::Amp),
        ("==", K::EqEq),
        ("<", K::Lt),
        ("<<", K::Shl),
        ("+", K::Plus),
        ("*", K::Star),
    ];
    for pair in ops.windows(2) {
        let (a, ka) = pair[0];
        let (b, kb) = pair[1];
        let (ast, root) = expression(&format!("a {a} b {b} c"));
        let (_, right) = binary(&ast, root, ka);
        binary(&ast, right, kb);
        let (ast, root) = expression(&format!("a {b} b {a} c"));
        let (left, _) = binary(&ast, root, ka);
        binary(&ast, left, kb);
    }
}
#[test]
fn left_associative_operators() {
    for (op, kind) in [
        ("-", K::Minus),
        ("/", K::Slash),
        ("%", K::Percent),
        (">>", K::Shr),
        ("^", K::Caret),
    ] {
        let (ast, root) = expression(&format!("a {op} b {op} c"));
        let (left, _) = binary(&ast, root, kind);
        binary(&ast, left, kind);
    }
}
#[test]
fn grouping_overrides_precedence() {
    let (ast, root) = expression("(a + b) * c");
    let (left, _) = binary(&ast, root, K::Star);
    match ast.get(left).unwrap().kind {
        N::Group { value } => {
            binary(&ast, value, K::Plus);
        }
        _ => panic!("missing group"),
    }
}
#[test]
fn unary_and_postfix_structure() {
    let (ast, root) = expression("-f(x).value[0]?");
    let N::Unary {
        operator: K::Minus,
        operand,
        ..
    } = ast.get(root).unwrap().kind
    else {
        panic!("unary")
    };
    let N::Propagate { value } = ast.get(operand).unwrap().kind else {
        panic!("propagate")
    };
    let N::Index { receiver, .. } = ast.get(value).unwrap().kind else {
        panic!("index")
    };
    let N::Member { receiver, .. } = ast.get(receiver).unwrap().kind else {
        panic!("member")
    };
    assert!(matches!(&ast.get(receiver).unwrap().kind,N::Call{arguments,..} if arguments.len()==1));
}
#[test]
fn function_types_and_visibility() {
    let ast = parse("pub fn view(var x: &mut i64) -> &i64 from x { return &*x; }");
    let N::Program { items, .. } = &ast.get(ast.root.unwrap()).unwrap().kind else {
        panic!("program")
    };
    let N::Item {
        public: true,
        declaration,
    } = ast.get(items[0]).unwrap().kind
    else {
        panic!("visibility")
    };
    let N::Function {
        parameters,
        result,
        from: Some(_),
        body,
        ..
    } = &ast.get(declaration).unwrap().kind
    else {
        panic!("function")
    };
    assert_eq!(parameters.len(), 1);
    let N::Parameter {
        mutable: true, ty, ..
    } = ast.get(parameters[0]).unwrap().kind
    else {
        panic!("parameter")
    };
    assert!(matches!(
        ast.get(ty).unwrap().kind,
        N::ReferenceType { mutable: true, .. }
    ));
    assert!(matches!(
        ast.get(*result).unwrap().kind,
        N::ReferenceType { mutable: false, .. }
    ));
    assert!(matches!(&ast.get(*body).unwrap().kind,N::Block{statements} if statements.len()==1));
}
#[test]
fn user_types_and_patterns() {
    let ast = parse(include_str!("../examples/07-user-types.cretes"));
    assert!(ast
        .nodes
        .iter()
        .any(|n| matches!(&n.kind,N::Record{fields,..} if fields.len()==2)));
    assert!(ast
        .nodes
        .iter()
        .any(|n| matches!(&n.kind,N::Enum{variants,..} if variants.len()==2)));
    assert!(ast
        .nodes
        .iter()
        .any(|n| matches!(&n.kind,N::Construct{fields,..} if fields.len()==2)));
    assert!(ast
        .nodes
        .iter()
        .any(|n| matches!(&n.kind,N::Match{arms,..} if arms.len()==2)));
    assert!(ast
        .nodes
        .iter()
        .any(|n| matches!(&n.kind,N::ConstructorPattern{fields,..} if fields.len()==1)));
}
#[test]
fn modules_and_import_alias() {
    let ast = parse("module a::b; import c::d as e; fn f() -> () {}");
    let N::Program {
        module: Some(module),
        imports,
        items,
    } = &ast.get(ast.root.unwrap()).unwrap().kind
    else {
        panic!("program")
    };
    assert_eq!(items.len(), 1);
    assert_eq!(imports.len(), 1);
    assert!(matches!(&ast.get(*module).unwrap().kind,N::Module{path} if path.len()==2));
    assert!(
        matches!(&ast.get(imports[0]).unwrap().kind,N::Import{path,alias:Some(_)} if path.len()==2)
    );
}
#[test]
fn deterministic_dump() {
    let a = parse("fn f() -> () {}");
    let b = parse("fn f() -> () {}");
    assert_eq!(a.dump(), b.dump());
    assert!(!a.dump().contains("0x"));
}
