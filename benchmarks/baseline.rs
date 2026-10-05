use cretes_frontend::{lexer, parse_source, parser, source::SourceManager, Limits};
use std::{hint::black_box, time::Instant};
fn main() {
    let text = "fn f(x: i64) -> i64 { return x + 2 * 3; }\n".repeat(10000);
    let mut sm = SourceManager::default();
    let id = sm.add("baseline.cretes", text.as_bytes());
    let src = sm.get(id).unwrap();
    let t = Instant::now();
    let lex = lexer::lex(src, Limits::default());
    let lex_time = t.elapsed();
    assert!(lex.diagnostics.is_empty());
    let t = Instant::now();
    let syntax = parser::parse(src, &lex.tokens, Limits::default());
    let parser_time = t.elapsed();
    assert!(syntax.diagnostics.is_empty());
    let t = Instant::now();
    let parsed = parse_source(src, Limits::default());
    let parse_time = t.elapsed();
    assert!(parsed.is_valid());
    let t = Instant::now();
    for i in (0..text.len()).step_by(31) {
        black_box(src.location(i));
    }
    println!(
        "bytes={} tokens={} nodes={} lex_us={} parser_us={} frontend_us={} mapping_us={}",
        text.len(),
        lex.tokens.len(),
        parsed.ast.nodes.len(),
        lex_time.as_micros(),
        parser_time.as_micros(),
        parse_time.as_micros(),
        t.elapsed().as_micros()
    );
}
