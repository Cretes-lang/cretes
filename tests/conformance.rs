use cretes_frontend::{parse_source, source::SourceManager, Limits};
#[test]
fn fixture_000_empty() {
    let mut sm = SourceManager::default();
    let id = sm.add("empty", "".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_001_comments() {
    let mut sm = SourceManager::default();
    let id = sm.add("comments", "/* outer /* inner */ done */ // eof".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_002_unit() {
    let mut sm = SourceManager::default();
    let id = sm.add("unit", "fn unit() -> () { return; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_003_trailing_parameters() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "trailing-parameters",
        "fn pair(x: i64, y: bool,) -> (i64,bool,) { return (x,y,); }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_004_nested_types() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "nested-types",
        "type Values = Seq[Option[Result[i64, text]]];".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_005_borrow_return() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "borrow-return",
        "fn view(x: &mut i64) -> &mut i64 from x { return x; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_006_pub_fields() {
    let mut sm = SourceManager::default();
    let id = sm.add("pub-fields", "pub struct S { pub x: i64, }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_007_import_alias() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "import-alias",
        "module demo::main; import std::io as output; fn main() -> i32 { return 0; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_008_enum() {
    let mut sm = SourceManager::default();
    let id = sm.add("enum", "enum E { A, B(i64, text), }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_009_new_in_condition() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "new-in-condition",
        "fn sample() -> () { if new Flag { yes: true }.yes { return; } }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_010_else_chain() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "else-chain",
        "fn sample() -> () { if true { } else if false { } else { } }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_011_patterns() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "patterns",
        "fn sample() -> () { match value { E::A() => {} E::B(x,y) => {} } }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_012_wildcard() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "wildcard",
        "fn sample() -> () { match value { _ => {} } }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_013_unit_tuple() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "unit-tuple",
        "fn sample() -> () { let a = (); let b = (1,); let c = (1,2); }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_014_precedence() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "precedence",
        "fn sample() -> () { let x = 1 + 2 * 3 << 1; let y = true || false && true; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_015_bitwise() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "bitwise",
        "fn sample() -> () { let x = ~1 & 2 ^ 3 | 4; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_016_relation() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "relation",
        "fn sample() -> () { let a = 1 <= 2; let b = 3 >= 2; let c = 1 != 2; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_017_chains() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "chains",
        "fn sample() -> () { let x = call(1,2,).field[0]?; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_018_negative_boundary() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "negative-boundary",
        "fn sample() -> () { let x: i64 = -9223372036854775808; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_019_numeric_bases() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "numeric-bases",
        "fn sample() -> () { let a=0b10_01; let b=0o17; let c=0xAb_CD; let d=1_000; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_020_floats() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "floats",
        "fn sample() -> () { let a=1.25e-2; let b=1e+3; let c=1E3; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_021_literals() {
    let mut sm = SourceManager::default();
    let id=sm.add("literals","fn sample() -> () { let a=\"\u{00e9}\\u{1F600}\\n\"; let b='\u{754c}'; let c=b\"A\\xFF\"; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_022_borrow_deref() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "borrow-deref",
        "fn sample() -> () { var x=1; let r=&mut x; *r=2; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_023_loops() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "loops",
        "fn sample() -> () { while true { break; } for x in [1,2] { continue; } loop { break; } }"
            .as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_024_nested_blocks() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "nested-blocks",
        "fn sample() -> () { let x=1; { let x=x+1; discard(x,\"shadow\"); } }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_025_constant() {
    let mut sm = SourceManager::default();
    let id = sm.add("constant", "const C: i64 = 1+2; type I = i64;".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_026_crlf() {
    let mut sm = SourceManager::default();
    let id = sm.add("crlf", "fn f() -> () {\r\nreturn;\r\n}".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_027_string_comment() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "string-comment",
        "fn sample() -> () { let s=\"/* not a comment */\"; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_028_escaped_control() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "escaped-control",
        "fn sample() -> () { let s=\"\\u{202e}\"; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_029_keyword_prefix() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "keyword-prefix",
        "fn sample() -> () { let format=1; let variable=2; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_030_float_dot_member() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "float-dot-member",
        "fn sample() -> () { let x=1.foo; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_031_utf8_comment() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "utf8-comment",
        "// \u{0395}\u{03bb}\u{03bb}\u{03b7}\u{03bd}\u{03b9}\u{03ba}\u{03ac}\nfn f() -> () {}"
            .as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_032_missing_semicolon() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "missing-semicolon",
        "fn sample() -> () { let x=1 }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_033_missing_brace() {
    let mut sm = SourceManager::default();
    let id = sm.add("missing-brace", "fn f() -> () {".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_034_missing_return_type() {
    let mut sm = SourceManager::default();
    let id = sm.add("missing-return-type", "fn f() {}".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_035_keyword_name() {
    let mut sm = SourceManager::default();
    let id = sm.add("keyword-name", "fn sample() -> () { let if=1; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_036_chained_comparison() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "chained-comparison",
        "fn sample() -> () { let x=1<2<3; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_037_chained_equality() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "chained-equality",
        "fn sample() -> () { let x=1==2==3; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_038_assignment_expression() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "assignment-expression",
        "fn sample() -> () { let x=(a=1); }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_039_double_assignment() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "double-assignment",
        "fn sample() -> () { a=b=c; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_040_unbraced_if() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "unbraced-if",
        "fn sample() -> () { if true return; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_041_unbraced_while() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "unbraced-while",
        "fn sample() -> () { while true break; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_042_unsupported_async() {
    let mut sm = SourceManager::default();
    let id = sm.add("unsupported-async", "async fn f() -> () {}".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_043_unsupported_await() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "unsupported-await",
        "fn sample() -> () { await job(); }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_044_unsupported_unsafe() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "unsupported-unsafe",
        "fn sample() -> () { unsafe { raw(); } }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_045_unsupported_ffi() {
    let mut sm = SourceManager::default();
    let id = sm.add("unsupported-ffi", "extern \"C\" fn f() -> ();".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_046_generic_function() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "generic-function",
        "fn id[T](x:T) -> T { return x; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_047_attribute() {
    let mut sm = SourceManager::default();
    let id = sm.add("attribute", "@test fn f() -> () {}".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_048_macro() {
    let mut sm = SourceManager::default();
    let id = sm.add("macro", "fn sample() -> () { print!(\"hi\"); }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_049_bad_record() {
    let mut sm = SourceManager::default();
    let id = sm.add("bad-record", "struct X { x: i64 }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_050_record_no_new() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "record-no-new",
        "fn sample() -> () { let p=Point {x:1}; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_051_named_args() {
    let mut sm = SourceManager::default();
    let id = sm.add("named-args", "fn sample() -> () { call(x:1); }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_052_empty_statement() {
    let mut sm = SourceManager::default();
    let id = sm.add("empty-statement", "fn sample() -> () { ; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_053_uninitialized() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "uninitialized",
        "fn sample() -> () { var x: i64; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_054_top_level_let() {
    let mut sm = SourceManager::default();
    let id = sm.add("top-level-let", "let x=1;".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_055_import_after_item() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "import-after-item",
        "fn f() -> () {} import std::io;".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_056_bad_function() {
    let mut sm = SourceManager::default();
    let id = sm.add("bad-function", "fn () -> () {}".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_057_missing_type() {
    let mut sm = SourceManager::default();
    let id = sm.add("missing-type", "fn f(x:) -> () {}".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_058_bad_type() {
    let mut sm = SourceManager::default();
    let id = sm.add("bad-type", "type X = Seq[i64;".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_059_empty_enum() {
    let mut sm = SourceManager::default();
    let id = sm.add("empty-enum", "enum E {}".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_060_nested_function() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "nested-function",
        "fn sample() -> () { fn inner() -> () {} }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_061_range() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "range",
        "fn sample() -> () { for n in 0..10 {} }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_062_trailing_dot() {
    let mut sm = SourceManager::default();
    let id = sm.add("trailing-dot", "fn sample() -> () { let n=1.; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_063_bad_base() {
    let mut sm = SourceManager::default();
    let id = sm.add("bad-base", "fn sample() -> () { let x=0b102; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_064_missing_base() {
    let mut sm = SourceManager::default();
    let id = sm.add("missing-base", "fn sample() -> () { let x=0x; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_065_bad_separator() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "bad-separator",
        "fn sample() -> () { let x=1__0; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_066_suffix() {
    let mut sm = SourceManager::default();
    let id = sm.add("suffix", "fn sample() -> () { let x=12i64; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_067_exponent() {
    let mut sm = SourceManager::default();
    let id = sm.add("exponent", "fn sample() -> () { let x=1e+; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_068_bad_escape() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "bad-escape",
        "fn sample() -> () { let x=\"\\q\"; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_069_surrogate() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "surrogate",
        "fn sample() -> () { let x=\"\\u{D800}\"; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_070_large_scalar() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "large-scalar",
        "fn sample() -> () { let x=\"\\u{110000}\"; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_071_long_char() {
    let mut sm = SourceManager::default();
    let id = sm.add("long-char", "fn sample() -> () { let x='ab'; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_072_empty_char() {
    let mut sm = SourceManager::default();
    let id = sm.add("empty-char", "fn sample() -> () { let x=''; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_073_byte_unicode() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "byte-unicode",
        "fn sample() -> () { let x=b\"\u{00e9}\"; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_074_bad_byte_escape() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "bad-byte-escape",
        "fn sample() -> () { let x=b\"\\x0\"; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_075_raw_newline() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "raw-newline",
        "fn sample() -> () { let x=\"a\nb\"; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_076_unicode_name() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "unicode-name",
        "fn sample() -> () { let x=caf\u{00e9}; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_077_bidi() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "bidi",
        "fn sample() -> () { let x=\"\u{202e}\"; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_078_bom() {
    let mut sm = SourceManager::default();
    let id = sm.add("bom", "fn sample() -> () { let x=\u{feff}; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_079_bare_cr() {
    let mut sm = SourceManager::default();
    let id = sm.add("bare-cr", "fn sample() -> () { let x=\r; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_080_internal_name() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "internal-name",
        "fn sample() -> () { let x=__secret; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_081_unclosed_comment() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "unclosed-comment",
        "fn sample() -> () { let x=/* nested /* x */; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_082_unclosed_string() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "unclosed-string",
        "fn sample() -> () { let x=\"hello; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_083_zero_width() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "zero-width",
        "fn sample() -> () { let x=a\u{200b}b; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_084_nbsp() {
    let mut sm = SourceManager::default();
    let id = sm.add("nbsp", "fn sample() -> () { let x=a\u{00a0}b; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(!r.is_valid(), "{:?}", r.diagnostics);
    assert!(r.diagnostics.iter().any(|d| d.code.starts_with("L")));
}
#[test]
fn fixture_085_immutable_write() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "immutable-write",
        "fn sample() -> () { let x=1; x=2; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_086_moved_owner() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "moved-owner",
        "fn sample() -> () { let x:Seq[i64]=[1]; let y=x; discard(x,\"invalid use\"); }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_087_escaping_local() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "escaping-local",
        "fn bad() -> &i64 { let x=1; return &x; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_088_conflicting_borrow() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "conflicting-borrow",
        "fn sample() -> () { var x=1; let r=&x; let m=&mut x; discard(*r,\"still live\"); }"
            .as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_089_unused_result() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "unused-result",
        "fn sample() -> () { let result=Result::Err(1); }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_090_non_exhaustive() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "non-exhaustive",
        "fn sample() -> () { match true { false => {} } }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_091_outside_break() {
    let mut sm = SourceManager::default();
    let id = sm.add("outside-break", "fn sample() -> () { break; }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_092_bad_arity() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "bad-arity",
        "fn sample() -> () { let x:Option[i64,text]=Option::None(); }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_093_option_propagation() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "option-propagation",
        "fn sample() -> () { let x:Option[i64]=Option::None(); let y=x?; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_094_constant_io() {
    let mut sm = SourceManager::default();
    let id = sm.add("constant-io", "const X: i64 = io::read();".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_095_numeric_conversion() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "numeric-conversion",
        "fn sample() -> () { let x:i8=1; let y:i64=x; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_096_borrowed_field() {
    let mut sm = SourceManager::default();
    let id = sm.add("borrowed-field", "struct Bad { x: &i64, }".as_bytes());
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_097_bad_provenance() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "bad-provenance",
        "fn bad(x:i64) -> i64 from x { return x; }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_098_unreachable_arm() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "unreachable-arm",
        "fn sample() -> () { match true { _ => {} true => {} } }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_099_result_discard_reason() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "result-discard-reason",
        "fn sample() -> () { discard(Result::Err(1), \"\"); }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn fixture_100_non_place_borrow() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "non-place-borrow",
        "fn sample() -> () { let r=&(1+2); }".as_bytes(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_01() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "01-hello-world.cretes",
        include_bytes!("../examples/01-hello-world.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_02() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "02-variables.cretes",
        include_bytes!("../examples/02-variables.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_03() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "03-functions.cretes",
        include_bytes!("../examples/03-functions.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_04() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "04-conditionals.cretes",
        include_bytes!("../examples/04-conditionals.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_05() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "05-loops.cretes",
        include_bytes!("../examples/05-loops.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_06() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "06-collections.cretes",
        include_bytes!("../examples/06-collections.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_07() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "07-user-types.cretes",
        include_bytes!("../examples/07-user-types.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_08() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "08-errors.cretes",
        include_bytes!("../examples/08-errors.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_09() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "09-modules.cretes",
        include_bytes!("../examples/09-modules.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_10() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "10-borrowing.cretes",
        include_bytes!("../examples/10-borrowing.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_11() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "11-automation.cretes",
        include_bytes!("../examples/11-automation.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_12() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "12-packet-validation.cretes",
        include_bytes!("../examples/12-packet-validation.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_13() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "13-numeric-preprocessing.cretes",
        include_bytes!("../examples/13-numeric-preprocessing.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn example_14() {
    let mut sm = SourceManager::default();
    let id = sm.add(
        "14-defensive-bytes.cretes",
        include_bytes!("../examples/14-defensive-bytes.cretes").as_slice(),
    );
    let r = parse_source(sm.get(id).unwrap(), Limits::default());
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
