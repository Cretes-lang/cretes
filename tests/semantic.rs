use cretes_frontend::{
    semantic::{analyze, ModuleInput, SemanticOptions, SemanticResult, Type},
    source::SourceManager,
};
fn modules(sources: &[(&str, &str)]) -> SemanticResult {
    let mut sm = SourceManager::default();
    let ids: Vec<_> = sources
        .iter()
        .map(|(n, s)| sm.add(*n, s.as_bytes()))
        .collect();
    let inputs: Vec<_> = sources
        .iter()
        .zip(ids)
        .map(|((n, _), id)| ModuleInput {
            identity: n,
            source: sm.get(id).unwrap(),
        })
        .collect();
    analyze(&inputs, SemanticOptions::default())
}
fn good(s: &str) {
    let r = modules(&[("test", s)]);
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
fn bad(s: &str, code: &str) {
    let r = modules(&[("test", s)]);
    assert!(
        r.diagnostics.iter().any(|d| d.code == code),
        "{s}\n{:?}",
        r.diagnostics
    );
}
#[test]
fn annotated_function() {
    good("fn add(x: i32, y: i32) -> i32 { return x + y; }");
}
#[test]
fn forward_functions() {
    good("fn first() -> i64 { return second(); } fn second() -> i64 { return 2; }");
}
#[test]
fn outer_initializer() {
    good("fn f() -> i64 { let x = 3; { let x = x + 1; discard(x, \"test\"); } return x; }");
}
#[test]
fn duplicate_parameter() {
    bad("fn f(x:i64,x:i64)->(){return;}", "M001");
}
#[test]
fn parameter_body_scope() {
    bad("fn f(x:i64)->(){let x=1;}", "M001");
}
#[test]
fn undefined_name() {
    bad("fn f()->i64{return absent;}", "M001");
}
#[test]
fn exact_numeric_types() {
    bad("fn f(x:i32,y:i64)->i64{return x+y;}", "T001");
}
#[test]
fn contextual_literals() {
    good("fn f(x:i8)->i8{return 1+x;}");
}
#[test]
fn integer_minimum() {
    good("fn f()->i64{return -9223372036854775808;}");
}
#[test]
fn integer_overflow() {
    bad("fn f()->i8{return 128;}", "T001");
}
#[test]
fn unsigned_negation() {
    bad("fn f()->u8{return -1;}", "T001");
}
#[test]
fn float_overflow() {
    bad("fn f()->f32{return 1e100;}", "T001");
}
#[test]
fn immutable_assignment() {
    bad("fn f()->(){let x=1; x=2;}", "T001");
}
#[test]
fn mutable_assignment() {
    good("fn f()->i32{var x:i32=1;x=2;return x;}");
}
#[test]
fn mutable_reference() {
    good("fn f(x:&mut i64)->(){*x=3;}");
}
#[test]
fn readonly_reference() {
    bad("fn f(var x:&i64)->(){*x=3;}", "T001");
}
#[test]
fn borrowed_temporary() {
    bad("fn f()->(){let x=&1;}", "B002");
}
#[test]
fn truthiness_rejected() {
    bad("fn f()->(){if 1 {return;}}", "T001");
}
#[test]
fn calls_check_arity() {
    bad(
        "fn f(x:i32)->i32{return x;} fn g()->i32{return f();}",
        "T001",
    );
}
#[test]
fn calls_check_types() {
    bad(
        "fn f(x:i32)->i32{return x;} fn g()->i32{return f(true);}",
        "T001",
    );
}
#[test]
fn missing_return() {
    bad("fn f(x:bool)->i64{if x{return 1;}}", "T001");
}
#[test]
fn all_paths_return() {
    good("fn f(x:bool)->i64{if x{return 1;}else{return 2;}}");
}
#[test]
fn infinite_loop_diverges() {
    good("fn f()->i64{loop{}}");
}
#[test]
fn break_context() {
    bad("fn f()->(){break;}", "T001");
}
#[test]
fn continue_context() {
    bad("fn f()->(){continue;}", "T001");
}
#[test]
fn records() {
    good("struct P {x:i64,} fn f()->i64{let p=new P{x:2};return p.x;}");
}
#[test]
fn record_missing_field() {
    bad("struct P {x:i64,} fn f()->P{return new P{};}", "T001");
}
#[test]
fn unknown_field() {
    bad("struct P {} fn f(p:P)->i64{return p.x;}", "T001");
}
#[test]
fn aliases() {
    good("type Number=i64; fn f(x:Number)->i64{return x;}");
}
#[test]
fn alias_cycle() {
    bad("type A=B; type B=A;", "T001");
}
#[test]
fn recursive_layout() {
    bad("struct P {p:P,}", "T001");
}
#[test]
fn boxed_recursion() {
    good("struct P {p:Box[P],}");
}
#[test]
fn enum_match() {
    good("enum E {A,B(i64),} fn f(e:E)->i64{match e {E::A()=>{return 0;} E::B(x)=>{return x;}}}");
}
#[test]
fn missing_variant() {
    bad(
        "enum E {A,B,} fn f(e:E)->(){match e {E::A()=>{return;}}}",
        "T002",
    );
}
#[test]
fn option_context() {
    good("fn f()->Option[i64]{return Option::Some(1);}");
}
#[test]
fn none_needs_context() {
    bad("fn f()->(){let n=Option::None();}", "T001");
}
#[test]
fn result_propagation() {
    good("fn f()->Result[i64,bool]{return Result::Ok(1);} fn g()->Result[i64,bool]{let x=f()?;return Result::Ok(x);}");
}
#[test]
fn option_cannot_propagate() {
    bad(
        "fn f(x:Option[i64])->Result[i64,bool]{let y=x?;return Result::Ok(y);}",
        "T001",
    );
}
#[test]
fn empty_sequence_context() {
    good("fn f()->Seq[i32]{return [];}");
}
#[test]
fn discarded_expression() {
    bad("fn f()->(){42;}", "T001");
}
#[test]
fn reason_required() {
    bad("fn f()->(){discard(1,\"\");}", "T001");
}
#[test]
fn valid_modules() {
    let r = modules(&[
        ("p::a", "module p::a; pub fn f()->i32{return 1;}"),
        (
            "p::b",
            "module p::b; import p::a; fn g()->i32{return a::f();}",
        ),
    ]);
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn private_function() {
    let r = modules(&[
        ("p::a", "fn f()->i32{return 1;}"),
        ("p::b", "import p::a; fn g()->i32{return a::f();}"),
    ]);
    assert!(r.diagnostics.iter().any(|d| d.code == "M001"));
}
#[test]
fn import_cycles() {
    let r = modules(&[("p::a", "import p::b;"), ("p::b", "import p::a;")]);
    assert!(r.diagnostics.iter().any(|d| d.code == "M001"));
}
#[test]
fn missing_module() {
    bad("import absent;", "M001");
}
#[test]
fn explicit_identity() {
    bad("module other;", "M001");
}
#[test]
fn canonical_type_ids() {
    let r = modules(&[("test", "fn f(x:i64,y:i64)->i64{return x+y;}")]);
    let ids: Vec<_> = r
        .types
        .iter()
        .filter(|(_, t)| {
            matches!(
                t,
                Type::Integer {
                    signed: true,
                    bits: 64
                }
            )
        })
        .collect();
    assert_eq!(ids.len(), 1);
}
#[test]
fn deterministic_output() {
    let src = [
        ("z", "fn z()->i64{return 1;}"),
        ("a", "fn a()->bool{return true;}"),
    ];
    let x = modules(&src);
    let y = modules(&[src[1], src[0]]);
    assert_eq!(x.dump(), y.dump());
}
#[test]
fn constant_values() {
    let r = modules(&[("test", "const n:i8=2+3*4; fn f()->i8{return n;}")]);
    assert!(r.is_valid(), "{:?}", r.diagnostics);
    assert!(r
        .constants
        .values()
        .any(|v| matches!(v, cretes_frontend::semantic::ConstantValue::Integer(14))));
}
#[test]
fn constant_cycle() {
    bad("const a:i64=b; const b:i64=a;", "C001");
}
#[test]
fn constant_overflow() {
    bad("const a:i8=127+1;", "C001");
}
#[test]
fn constant_division() {
    bad("const a:i64=1/0;", "C001");
}
#[test]
fn constant_min_remainder() {
    bad("const a:i8=-128%-1;", "C001");
}
#[test]
fn constant_shift() {
    bad("const a:u8=1<<8;", "C001");
}
#[test]
fn constant_min() {
    good("const a:i64=-9223372036854775808;");
}
#[test]
fn constant_short_circuit() {
    good("const a:bool=false&&(1/0==0);");
}
#[test]
fn constant_call_rejected() {
    bad("fn f()->i64{return 1;} const a:i64=f();", "C001");
}
#[test]
fn nested_exhaustive_patterns() {
    good("enum E {V(bool),} fn f(e:E)->i64{match e {E::V(true)=>{return 1;}E::V(false)=>{return 0;}}}");
}
#[test]
fn tuple_exhaustiveness() {
    good("fn f(x:(bool,bool))->i64{match x {(true,_)=>{return 1;} (false,true)=>{return 2;} (false,false)=>{return 3;}}}");
}
#[test]
fn nested_nonexhaustive() {
    bad(
        "enum E {V(bool),} fn f(e:E)->(){match e {E::V(true)=>{return;}}}",
        "T002",
    );
}
#[test]
fn duplicate_boolean_pattern() {
    bad(
        "fn f(x:bool)->(){match x {true=>{} true=>{} false=>{}}}",
        "T002",
    );
}
#[test]
fn duplicate_literal_spelling() {
    bad("fn f(x:i64)->(){match x {0x01=>{} 1=>{} _=>{}}}", "T002");
}
#[test]
fn wrong_enum_pattern() {
    bad(
        "enum A{X,} enum B{X,} fn f(x:A)->(){match x{B::X()=>{}}}",
        "T001",
    );
}

#[test]
fn constant_underscore_character() {
    let r = modules(&[("test", "const c:char='_';")]);
    assert!(r.is_valid(), "{:?}", r.diagnostics);
    assert!(r
        .constants
        .values()
        .any(|v| matches!(v, cretes_frontend::semantic::ConstantValue::Char('_'))));
}
#[test]
fn unevaluated_constant_call_rejected() {
    bad(
        "fn f()->bool{return true;} const b:bool=false&&f();",
        "C001",
    );
}
#[test]
fn underscore_module_identity() {
    let r = modules(&[("_internal", "module _internal; fn f()->(){}")]);
    assert!(r.is_valid(), "{:?}", r.diagnostics);
}
#[test]
fn malformed_match_arity_does_not_crash() {
    bad(
        "enum E{V(bool),} fn f(e:E)->(){match e{E::V(true,false)=>{} E::V(_)=>{}}}",
        "T001",
    );
}

#[test]
fn nested_aggregate_reference() {
    bad("fn f(x:Box[Seq[&i64]])->(){}", "B002");
}
#[test]
fn tuple_reference() {
    bad("fn f(x:(&i64,))->(){}", "B002");
}
#[test]
fn float_map_key() {
    bad("fn f(x:Map[f64,i32])->(){}", "T001");
}
#[test]
fn nominal_set_key() {
    bad("struct P{} fn f(x:Set[P])->(){}", "T001");
}
#[test]
fn tuple_hash_key() {
    good("fn f(x:Map[(text,u8),i32])->(){}");
}
#[test]
fn private_return_type() {
    bad("struct P{} pub fn f()->P{return new P{};}", "M001");
}
#[test]
fn private_parameter_type() {
    bad("struct P{} pub fn f(p:Option[P])->(){}", "M001");
}
#[test]
fn public_enum_private_payload() {
    bad("struct P{} pub enum E{V(P),}", "M001");
}
#[test]
fn public_record_private_representation() {
    good("struct P{} pub struct S{p:P,}");
}
#[test]
fn public_field_private_type() {
    bad("struct P{} pub struct S{pub p:P,}", "M001");
}
#[test]
fn type_name_is_not_value() {
    bad("struct P{} fn f()->P{return P;}", "T001");
}
#[test]
fn invalid_target_width() {
    let mut sm = SourceManager::default();
    let id = sm.add("test", "const x:usize=1;".as_bytes());
    let r = analyze(
        &[ModuleInput {
            identity: "test",
            source: sm.get(id).unwrap(),
        }],
        SemanticOptions {
            target_pointer_bits: 255,
            ..SemanticOptions::default()
        },
    );
    assert!(r.diagnostics.iter().any(|d| d.code == "T001"));
}

#[test]
fn shared_type_graph_does_not_expand_exponentially() {
    let mut s = String::from("struct T0{a:i64,b:i64,}");
    for i in 1..40 {
        s.push_str(&format!("struct T{i}{{a:T{},b:T{},}}", i - 1, i - 1));
    }
    s.push_str("fn equal(a:T39,b:T39)->bool{return a==b;}");
    good(&s);
}
#[test]
fn copyable_option_equality() {
    good("fn f(a:Option[i64],b:Option[i64])->bool{return a==b;}");
}
#[test]
fn noncopyable_option_equality() {
    bad(
        "fn f(a:Option[text],b:Option[text])->bool{return a==b;}",
        "T001",
    );
}
#[test]
fn diagnostic_labels_are_bounded() {
    let mut sm = SourceManager::default();
    let s = format!("fn f()->(){{{}}}", "let x=1;".repeat(1000));
    let id = sm.add("test", s.as_bytes());
    let mut options = SemanticOptions::default();
    options.limits.diagnostics = 1;
    let r = analyze(
        &[ModuleInput {
            identity: "test",
            source: sm.get(id).unwrap(),
        }],
        options,
    );
    assert_eq!(r.diagnostics.len(), 1);
    assert!(r.diagnostics[0].secondary.len() <= 1);
}
#[test]
fn semantic_fuel_limit() {
    let mut sm = SourceManager::default();
    let id = sm.add("test", "fn f()->i64{return 1;}".as_bytes());
    let r = analyze(
        &[ModuleInput {
            identity: "test",
            source: sm.get(id).unwrap(),
        }],
        SemanticOptions {
            work: 1,
            ..SemanticOptions::default()
        },
    );
    assert!(r.diagnostics.iter().any(|d| d.code == "R001"));
}
#[test]
fn malformed_semantic_inputs_terminate() {
    for source in [
        "fn",
        "fn f(x:)->{",
        "enum E{V(bool),} fn f(x:E)->(){match x{E::V()=>{} E::V(true,false)=>{} _=>{}}}",
        "fn f()->(){let a=((((;}",
    ] {
        let r = modules(&[("test", source)]);
        assert!(!r.is_valid());
    }
}

#[test]
fn mixed_loop_exits() {
    good("fn f(x:bool)->i64{loop{if x{return 1;}else{break;}}return 2;}");
}
#[test]
fn unreachable_break_does_not_end_loop() {
    good("fn f()->i64{loop{continue;break;}}");
}
#[test]
fn literal_true_while_diverges() {
    good("fn f()->i64{while true {}}");
}
#[test]
fn false_while_still_needs_return() {
    bad("fn f()->i64{while false{return 1;}}", "T001");
}
#[test]
fn conditional_break_requires_return() {
    bad(
        "fn f(x:bool)->i64{loop{if x{break;}else{return 1;}}}",
        "T001",
    );
}
#[test]
fn nested_loop_break_is_local() {
    good("fn f()->i64{loop{loop{break;}}}");
}
#[test]
fn match_breaks_leave_loop() {
    bad(
        "fn f(x:bool)->i64{loop{match x{true=>{break;}false=>{return 1;}}}}",
        "T001",
    );
}
#[test]
fn branch_return_continue_diverges_or_returns() {
    good("fn f(x:bool)->i64{loop{if x{return 1;}else{continue;}}}");
}
#[test]
fn for_may_be_empty() {
    bad("fn f(x:Seq[i64])->i64{for y in x{return y;}}", "T001");
}
#[test]
fn literal_true_branch_returns() {
    good("fn f()->i64{if true{return 1;}}");
}

#[test]
fn semantic_handoff_preserves_contracts() {
    let r = modules(&[(
        "test",
        "pub struct P{pub x:i64,} pub enum E{V(i64),} fn f(p:&i64)->&i64 from p{return p;}",
    )]);
    assert!(r.is_valid(), "{:?}", r.diagnostics);
    let sig = r.signatures.values().next().unwrap();
    assert_eq!(sig.from, Some(0));
    assert_eq!(sig.parameters.len(), 1);
    assert_eq!(r.records.values().next().unwrap()[0].name, "x");
    assert_eq!(r.enums.values().next().unwrap()[0].payload.len(), 1);
}
#[test]
fn value_categories_distinguish_mutability() {
    use cretes_frontend::semantic::ValueCategory;
    let r = modules(&[("test", "fn f()->i64{let x=1;var y=2;y=x+1;return y;}")]);
    assert!(r.is_valid(), "{:?}", r.diagnostics);
    let c = &r.modules[0].value_categories;
    assert!(c.values().any(|x| *x == ValueCategory::Value));
    assert!(c.values().any(|x| *x == ValueCategory::ReadOnlyPlace));
    assert!(c.values().any(|x| *x == ValueCategory::WritablePlace));
}

#[test]
fn enum_constructor_respects_local_shadowing() {
    bad(
        "enum E { A, } fn f()->E { let E=1; return E::A(); }",
        "M001",
    );
}
#[test]
fn enum_value_is_not_a_type_qualifier() {
    bad(
        "enum E { A, } fn f()->E { let value=E::A(); return value::A(); }",
        "T001",
    );
}
#[test]
fn enum_constructor_rejects_extra_path_components() {
    let r = modules(&[
        ("a", "pub enum E { A, }"),
        ("b", "import a; fn f()->a::E{return a::E::extra::A();}"),
    ]);
    assert!(!r.is_valid());
}
#[test]
fn diagnostic_type_dag_expansion_is_bounded() {
    let mut source = "type A0 = (i64,i64);".to_string();
    for i in 1..24 {
        source.push_str(&format!("type A{i} = (A{},A{});", i - 1, i - 1));
    }
    source.push_str("fn f(x:A23)->bool{return x;}");
    let r = modules(&[("test", &source)]);
    assert!(!r.is_valid());
    assert!(r.diagnostics.iter().all(|d| d.message.len() < 1200));
}
fn entry(source: &str, selected: &str) -> SemanticResult {
    let mut sm = SourceManager::default();
    let id = sm.add("app.cretes", source.as_bytes());
    cretes_frontend::semantic::analyze_entry(
        &[ModuleInput {
            identity: "app",
            source: sm.get(id).unwrap(),
        }],
        SemanticOptions::default(),
        selected,
    )
    .unwrap()
}
#[test]
fn entry_accepts_i32_and_result() {
    assert!(entry("fn main()->i32{return 0;}", "app").is_valid());
    assert!(entry("fn main()->Result[i32,text]{return Result::Ok(0);}", "app").is_valid());
}
#[test]
fn entry_rejects_wrong_contracts() {
    for source in [
        "",
        "fn main(x:i32)->i32{return x;}",
        "fn main()->i64{return 0;}",
        "fn main()->Result[i64,text]{return Result::Ok(0);}",
        "const main:i32=0;",
    ] {
        assert!(!entry(source, "app").is_valid(), "{source}");
    }
}
#[test]
fn entry_requires_explicit_existing_module() {
    assert!(!entry("fn main()->i32{return 0;}", "missing").is_valid());
}
#[test]
fn library_does_not_require_main() {
    good("pub fn answer()->i64{return 42;}");
}

#[test]
fn entry_rejects_absent_sources() {
    assert!(
        cretes_frontend::semantic::analyze_entry(&[], SemanticOptions::default(), "app").is_err()
    );
}

#[test]
fn reference_return_direct_and_reborrow() {
    good("fn f(x:&i64)->&i64 from x{return x;}");
    good("fn f(x:&i64)->&i64 from x{return &*x;}");
    good("fn f(x:&mut i64)->&i64 from x{return &*x;}");
}
#[test]
fn reference_return_rejects_local_storage() {
    bad("fn f(x:&i64)->&i64 from x{let y=0; return &y;}", "B002");
}
#[test]
fn reference_return_rejects_other_parameter() {
    bad("fn f(x:&i64,y:&i64)->&i64 from x{return y;}", "B002");
}
#[test]
fn reference_return_tracks_aliases_and_reassignment() {
    good("fn f(x:&i64,y:&i64)->&i64 from x{var r=y;r=x;let s=r;return s;}");
    bad(
        "fn f(x:&i64,y:&i64)->&i64 from x{var r=x;r=y;return r;}",
        "B002",
    );
}
#[test]
fn reference_return_checks_branch_join() {
    bad(
        "fn f(x:&i64,y:&i64,b:bool)->&i64 from x{var r=x;if b{r=y;}return r;}",
        "B002",
    );
    good("fn f(x:&i64,y:&i64,b:bool)->&i64 from x{var r=y;if b{r=x;}else{r=x;}return r;}");
}
#[test]
fn reference_return_unreachable_branch_has_no_origin_effect() {
    good("fn f(x:&i64,y:&i64)->&i64 from x{var r=x;if false{r=y;}return r;}");
    good("fn f(x:&i64,y:&i64)->&i64 from x{return x;return y;}");
}
#[test]
fn reference_return_tracks_call_contract() {
    good("fn id(x:&i64)->&i64 from x{return x;} fn f(x:&i64)->&i64 from x{return id(x);}");
    bad(
        "fn id(x:&i64)->&i64 from x{return x;} fn f(x:&i64,y:&i64)->&i64 from x{return id(y);}",
        "B002",
    );
}
#[test]
fn reference_return_tracks_projection() {
    good("struct R{n:i64,} fn f(x:&R)->&i64 from x{return &(*x).n;}");
    good("fn f(x:&Seq[i64])->&i64 from x{return &(*x)[0];}");
}
#[test]
fn reference_return_checks_loop_backedge() {
    bad(
        "fn f(x:&i64,y:&i64,b:bool)->&i64 from x{var r=x;loop{if b{return r;}r=y;}}",
        "B002",
    );
    good("fn f(x:&i64,y:&i64)->&i64 from x{var r=y;loop{r=x;break;}return r;}");
}
#[test]
fn reference_return_checks_zero_iteration_path() {
    bad(
        "fn f(x:&i64,y:&i64,b:bool)->&i64 from x{var r=y;while b{r=x;}return r;}",
        "B002",
    );
}
#[test]
fn reference_return_tracks_borrowed_match_payload() {
    good("fn f(x:&Option[i64],fallback:&i64)->&i64 from x{match x{Option::Some(v)=>{return v;}Option::None()=>{loop{}}}} ");
}
#[test]
fn reference_return_tracks_iterator_reference() {
    good("fn f(x:&Seq[i64])->&i64 from x{for v in x{return v;}loop{}}");
}
