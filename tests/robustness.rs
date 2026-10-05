use cretes_frontend::{
    ast::NodeKind,
    lexer, parse_source,
    source::{SourceId, SourceManager, Span},
    token::{Kind, KEYWORDS, PUNCTUATION},
    Limits,
};
fn check(bytes: &[u8]) -> cretes_frontend::FrontendResult {
    let mut sm = SourceManager::default();
    let id = sm.add("test", bytes);
    parse_source(sm.get(id).unwrap(), Limits::default())
}
#[test]
fn every_keyword_and_prefix() {
    for (word, kind) in KEYWORDS {
        let mut sm = SourceManager::default();
        let text = format!("{word} {word}Name");
        let id = sm.add("k", text.as_bytes());
        let r = lexer::lex(sm.get(id).unwrap(), Limits::default());
        assert_eq!(r.tokens[0].kind, *kind);
        assert_eq!(r.tokens[1].kind, Kind::Ident);
        assert!(r.diagnostics.is_empty());
    }
}
#[test]
fn every_operator_and_longest_match() {
    for (word, kind) in PUNCTUATION {
        let mut sm = SourceManager::default();
        let id = sm.add("op", word.as_bytes());
        let r = lexer::lex(sm.get(id).unwrap(), Limits::default());
        assert_eq!(r.tokens[0].kind, *kind);
        assert_eq!(r.tokens[0].span.end, word.len());
        assert_eq!(r.tokens.len(), 2);
    }
}
#[test]
fn invalid_utf8_variants() {
    for bytes in [
        &[0xff][..],
        &[0xc0, 0xaf],
        &[0xed, 0xa0, 0x80],
        &[0xf4, 0x90, 0x80, 0x80],
        &[0xe2, 0x82],
    ] {
        let r = check(bytes);
        assert_eq!(r.diagnostics[0].code, "L001");
        assert_eq!(r.tokens.last().unwrap().kind, Kind::Eof);
    }
}
#[test]
fn unicode_and_crlf_positions() {
    let mut sm = SourceManager::default();
    let id = sm.add("a", "//😀\r\nlet".as_bytes());
    let s = sm.get(id).unwrap();
    assert_eq!(s.location(6), Some((1, 4)));
    assert_eq!(s.location(8), Some((2, 1)));
    assert_eq!(s.location(3), None);
    assert_eq!(s.location(12), None);
    assert_eq!(s.slice(Span::new(SourceId(99), 0, 1)), None);
    assert_eq!(s.slice(s.span(4, 3)), None);
    let r = lexer::lex(s, Limits::default());
    assert_eq!(r.tokens[0].span.start, 8);
}
#[test]
fn stable_session_ids() {
    let mut sm = SourceManager::default();
    let a = sm.add("same", b"a".as_slice());
    let b = sm.add("same", b"b".as_slice());
    assert_ne!(a, b);
    assert_eq!(sm.get(a).unwrap().bytes(), b"a");
}
#[test]
fn preserves_trivia_and_doc_attachment() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "docs",
        b"/// hi\n/// there\nfn f()->(){}\n/// detached\n\nfn g()->(){}".as_slice(),
    );
    let s = sm.get(id).unwrap();
    let r = parse_source(s, Limits::default());
    assert!(r.is_valid());
    let items = r
        .ast
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| matches!(n.kind, NodeKind::Item { .. }))
        .map(|(i, _)| cretes_frontend::ast::NodeId(i))
        .collect::<Vec<_>>();
    assert_eq!(r.ast.documentation(items[0], s, &r.trivia).len(), 2);
    assert!(r.ast.documentation(items[1], s, &r.trivia).is_empty());
}
#[test]
fn nested_comments_10000() {
    let text = format!("{}{}", "/*".repeat(10000), "*/".repeat(10000));
    assert!(check(text.as_bytes()).is_valid());
}
#[test]
fn minimum_supported_parenthesis_nesting() {
    let text = format!(
        "fn f()->i64{{return {}1{};}}",
        "(".repeat(128),
        ")".repeat(128)
    );
    assert!(check(text.as_bytes()).is_valid());
}
#[test]
fn minimum_supported_block_nesting() {
    let text = format!("fn f()->(){{{}{} }}", "{".repeat(128), "}".repeat(128));
    assert!(check(text.as_bytes()).is_valid());
}
#[test]
fn excessive_nesting_is_diagnostic() {
    let text = format!(
        "fn f()->i64{{return {}1{};}}",
        "(".repeat(4000),
        ")".repeat(4000)
    );
    let r = check(text.as_bytes());
    assert!(r.diagnostics.iter().any(|d| d.code == "R001"));
}
#[test]
fn long_operator_chain_has_flat_ownership() {
    let text = format!("fn f()->i64{{return 0{};}}", "+1".repeat(20000));
    let r = check(text.as_bytes());
    assert!(r.is_valid());
    assert!(r.ast.nodes.len() > 20000);
}
#[test]
fn minimum_source_capacity() {
    let text = format!("//{}", "x".repeat(1024 * 1024));
    assert!(check(text.as_bytes()).is_valid());
}
#[test]
fn long_literal_and_identifier() {
    let text = format!(
        "fn {}()->text{{return \"{}\";}}",
        "a".repeat(100000),
        "é".repeat(100000)
    );
    assert!(check(text.as_bytes()).is_valid());
}
#[test]
fn diagnostic_amplification_is_bounded() {
    let r = check("@ ".repeat(10000).as_bytes());
    assert!(!r.is_valid());
    assert!(r.diagnostics.len() <= 100);
}
#[test]
fn deterministic_arbitrary_bytes() {
    let mut state = 7u64;
    for n in 0..400 {
        let mut bytes = vec![];
        for _ in 0..n % 128 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            bytes.push((state >> 32) as u8);
        }
        let a = check(&bytes);
        let b = check(&bytes);
        assert_eq!(a.diagnostics, b.diagnostics);
        assert_eq!(a.ast.dump(), b.ast.dump());
    }
}
#[test]
fn random_ascii_terminates() {
    let alphabet = b"fn let var (){}[]:;=+-*/'\"abc012_?&\n ";
    let mut state = 9u32;
    for _ in 0..1000 {
        let mut bytes = vec![];
        for _ in 0..128 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            bytes.push(alphabet[state as usize % alphabet.len()]);
        }
        let r = check(&bytes);
        assert!(r.diagnostics.len() <= 100);
    }
}
#[test]
fn recovery_keeps_later_function() {
    let r = check(b"fn bad()->(){let =1; let x=; } fn good()->(){return;}");
    assert!(!r.is_valid());
    assert!(r
        .ast
        .nodes
        .iter()
        .any(|n| matches!(n.kind, NodeKind::Error)));
    assert_eq!(
        r.ast
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, NodeKind::Function { .. }))
            .count(),
        2
    );
}
#[test]
fn malformed_delimiter_has_opening_label() {
    let r = check(b"fn f(x:i64 -> () {}");
    assert!(r.diagnostics.iter().any(|d| !d.secondary.is_empty()));
}
#[test]
fn source_controls_rejected_inside_comments() {
    for c in ['\u{202e}', '\u{200b}', '\u{feff}', '\u{7f}', '\u{85}'] {
        let text = format!("// {c}");
        assert!(check(text.as_bytes())
            .diagnostics
            .iter()
            .any(|d| d.code == "L001"));
    }
}
#[test]
fn raw_cr_is_lexical_error() {
    assert!(check(b"// x\ry")
        .diagnostics
        .iter()
        .any(|d| d.code == "L001"));
}
#[test]
fn numeric_error_categories() {
    for s in ["0x", "0b102", "1_", "1__2", "1e+", "12foo", "0o8"] {
        let mut sm = SourceManager::default();
        let id = sm.add("n", s.as_bytes());
        assert_eq!(
            lexer::lex(sm.get(id).unwrap(), Limits::default()).diagnostics[0].code,
            "L003"
        );
    }
}
#[test]
fn literal_error_categories() {
    for s in [
        "\"\\q\"",
        "'ab'",
        "''",
        "b\"é\"",
        "\"\\u{d800}\"",
        "\"\\u{110000}\"",
        "\"abc",
    ] {
        assert!(check(s.as_bytes())
            .diagnostics
            .iter()
            .any(|d| d.code == "L004"));
    }
}
#[test]
fn explicit_resource_limits() {
    let mut sm = SourceManager::default();
    let id = sm.add("x", b"fn f()->(){}".as_slice());
    for limits in [
        Limits {
            source_bytes: 2,
            ..Default::default()
        },
        Limits {
            tokens: 2,
            ..Default::default()
        },
        Limits {
            nodes: 1,
            ..Default::default()
        },
    ] {
        let r = parse_source(sm.get(id).unwrap(), limits);
        assert!(r.diagnostics.iter().any(|d| d.code == "R001"));
    }
}
#[test]
fn zero_diagnostic_budget_cannot_hide_errors() {
    let mut sm = SourceManager::default();
    let id = sm.add("x", b"@".as_slice());
    assert!(!parse_source(
        sm.get(id).unwrap(),
        Limits {
            diagnostics: 0,
            ..Default::default()
        }
    )
    .is_valid());
}
#[test]
fn json_and_terminal_escape_controls() {
    let mut sm = SourceManager::default();
    let id = sm.add("bad\x1b[31m", b"@".as_slice());
    let s = sm.get(id).unwrap();
    let r = parse_source(s, Limits::default());
    let d = &r.diagnostics[0];
    assert!(!d.render(s).contains('\x1b'));
    assert!(d.json().contains("\"schema_version\":1"));
    assert!(d.json().contains("utf8-byte"));
}
#[test]
fn tokens_and_nodes_are_valid_ranges() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "x",
        include_bytes!("../examples/10-borrowing.cretes").as_slice(),
    );
    let s = sm.get(id).unwrap();
    let r = parse_source(s, Limits::default());
    for t in &r.tokens {
        assert!(s.slice(t.span).is_some());
    }
    for n in &r.ast.nodes {
        assert!(s.slice(n.span).is_some());
    }
}
#[test]
fn malformed_public_token_stream() {
    let mut sm = SourceManager::default();
    let id = sm.add("x", b"".as_slice());
    let s = sm.get(id).unwrap();
    assert!(!cretes_frontend::parser::parse(s, &[], Limits::default())
        .diagnostics
        .is_empty());
}

#[test]
fn public_parser_respects_input_budget() {
    let mut sm = SourceManager::default();
    let id = sm.add("limit.cretes", b"fn f() -> () {}".as_slice());
    let source = sm.get(id).unwrap();
    let lexed = cretes_frontend::lexer::lex(source, Limits::default());
    let r = cretes_frontend::parser::parse(
        source,
        &lexed.tokens,
        Limits {
            tokens: 1,
            ..Limits::default()
        },
    );
    assert!(r.ast.root.is_none());
    assert_eq!(r.diagnostics[0].code, "R001");
}
