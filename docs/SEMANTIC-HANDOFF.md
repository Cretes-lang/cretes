# Semantic handoff and verification contract

Reviewed 2026-10-13. This describes the implemented provisional Phase 5 boundary,
not a completed safety checker or a stable serialized format. Language authority:
Phase 3 candidate `fe6d0d48518d0419d59fb680673150c92b6a084c`; architecture authority:
accepted RFC 0001, especially ARCH-PIPELINE-001, ARCH-TYPE-001 and ARCH-MEM-001.

## Caller contract

Call `analyze(&[ModuleInput], SemanticOptions)` for a library/module set, or
`analyze_entry` for an explicitly selected executable module. Supply immutable
source snapshots from one source-manager session, unique valid module identities,
and an explicit 32/64-bit target width. The analyzer never discovers dependencies,
executes source, supplies a standard library, or generates a program artifact.

Keep the source manager alive for later source rendering. The result owns ASTs and
semantic tables, but source bytes remain caller-owned. Keep every table together:
IDs are valid only within one result/session, not persistent hashes or cross-build
identities. A node is identified by `(ModuleId, NodeId)`, not NodeId alone.
Module identities sort deterministically; SourceId order need not match ModuleId.
Do not mutate public tables and then assume analyzer invariants still hold.

`SemanticResult.target_pointer_bits` retains the target used by range checks.
`Type::Usize` remains an abstract target-sized type; consumers must use that target
metadata instead of the host pointer width. Types are interned within the session.
Nominal types link to SymbolId; aliases resolve transparently. Source spans remain
original byte offsets and SourceIds, not module indices or display columns.

## Available data

| Data | Representation | Consumer responsibility |
| --- | --- | --- |
| Syntax/source mapping | `modules[].ast`, node/name spans | Keep original snapshots; visit the AST for evaluation order and control flow |
| Logical module map | `Module.identity`, `imports`, `scope` | Use the explicit map; do not invent filesystem/package resolution |
| Binding resolution | `Module.resolutions`, `symbols`, `scopes` | Qualify NodeId by its module; respect symbol visibility and mutable flag |
| Types | `node_types`, `types`, `Symbol.ty` | Reject error results; do not assume every punctuation/declaration node has a type-table entry |
| Places | `value_categories` | Value/read-only/writable is a type-level classification, not a loan-validity proof |
| Function contracts | `signatures` | `from` is a parameter index; resolve its reference type and later verify full lifetime/loan validity |
| Aggregate contracts | `records`, `enums` | Preserve field visibility, variant payloads and source spans |
| Constant values | `constants` | Only the restricted checked constant subset is evaluated |
| Diagnostics | `diagnostics` | Error results may contain partial tables; never lower them as valid programs |

This boundary is typed AST plus side tables. It is not HIR, MIR, a CFG, an ownership
certificate or executable code. The separate lowering/safety stages must construct
and verify their own representations before any backend work.

## What success means

`is_valid()` means no error diagnostics from the checks currently implemented.
It is not a whole-language or memory-safety assertion. Inspection output begins
with the implemented-check result, `move/loan safety: not analyzed`, and target
pointer width, including when source diagnostics are produced. CLI exit zero has
the same limited meaning. The experimental dump is deterministic debugging output,
not a stable protocol. JSON diagnostics remain diagnostics, not a safety certificate.

Current checks include resolution/privacy, type compatibility/local inference,
return paths, restricted constants, match coverage, direct return provenance and
Result-obligation analysis. The separate safety stage must still establish:

- definite initialization, moves/use-after-move, and reinitialization;
- live shared/exclusive loans, reborrow ancestry and reference lifetime bounds;
- invalidation by mutation/resizing, indirect writes and interprocedural effects;
- cleanup/drop states for ordinary exits, propagation and nested control flow;
- partial-move prohibition and safe ownership transfer at every projection.

Return provenance checks a `from` origin relation; it does not discharge those
loan obligations. A writable place does not imply it is currently safe to write.
No later consumer may skip safety analysis because `is_valid()` returns true.

## Result precision and outstanding integration cases

The pass tracks actual tuple/record/constructor/known-sequence payloads and joins
possible obligations at branches. Owned iteration initializes each element's
obligation; break/return/propagation check possible unvisited obligations. Earlier
argument/aggregate temporaries remain live during subsequent evaluation. While
conditions are reevaluated at abstract back edges.

Unknown call/parameter payloads, dynamic indices and iteration cardinalities are
conservative. Recursive shapes widen to a may-own value at 64 projection levels;
this bounds abstract growth but may lose knowledge of deeply empty payloads.
Reference mutation and full ownership effects remain integration work. These are
implementation limitations, not additions to the normative language. Do not use
a conservative T003 rejection to rewrite the language specification.

## Phase 1 → architecture → language → implementation → tests

Tests below are in `tests/semantic.rs` unless stated otherwise. This matrix records
Phase 5 contributions, not satisfaction of every runtime/release requirement.

| Requirements | Architecture | Language | Implementation | Representative evidence |
| --- | --- | --- | --- | --- |
| CORE-007–009 | ARCH-MODULE-001 / ARCH-PIPELINE-001 | LANG-MODULE-001 | `mod.rs::collect`, scope/name resolution | `valid_modules`, `private_function`, `import_cycles`, `missing_module` |
| CORE-011, CORE-020 | ARCH-TYPE-001 | LANG-DECL-001 / LANG-MUT-001 | `check.rs::statement`, `expression`, place categories | `outer_initializer`, `immutable_assignment`, `mutable_assignment`, `value_categories_distinguish_mutability` |
| CORE-012 | ARCH-TYPE-001 | LANG-RECORD-001 / LANG-SUM-001 | `resolve_symbol`, `check_layouts`, record/enum tables | `records`, `aliases`, `recursive_layout`, `boxed_recursion` |
| CORE-014–015 | ARCH-TYPE-001 | LANG-FUNC-001 / LANG-IF-001 / LANG-LOOP-001 | function contracts and reachable exit analysis | `calls_check_arity`, `all_paths_return`, `mixed_loop_exits`, `for_may_be_empty` |
| CORE-018, CORE-021 | ARCH-TYPE-001 | LANG-TYPE-001 / LANG-LITERAL-001 | interned types, contextual literals, target width | `exact_numeric_types`, `integer_overflow`, `invalid_target_width`, `semantic_handoff_keeps_target_and_session_identity` |
| CORE-022–024 | ARCH-ERROR-001 | LANG-ERROR-001 / LANG-PATTERN-001 | `obligations.rs`, expression propagation, `patterns.rs` | `result_propagation`, `result_shape_preserves_empty_tuple_component`, `result_owned_iteration_handles_each_element`, `result_temporaries_survive_until_enclosing_expression_completes` |
| SAFE-005 | ARCH-MEM-001 | LANG-PARAM-001 | `provenance.rs::check_return_provenance` | `reference_return_tracks_call_contract`, `reference_return_checks_branch_join`; full loan validity deferred |
| SAFE-015 | ARCH-TYPE-001 | LANG-TYPE-001 / LANG-PATTERN-001 | type compatibility, public type validation, match usefulness | `public_enum_private_payload`, `nested_exhaustive_patterns`, `wrong_enum_pattern` |
| SAFE-008–011 | ARCH-TYPE-001 | LANG-OP-001 / LANG-DECL-001 | `constants.rs` restricted arithmetic | `constant_overflow`, `constant_division`, `constant_shift`; runtime checks deferred |
| CORE-025 | ARCH-COMPILER-001 | LANG-MODULE-001 | deterministic module ordering and semantic dump | `deterministic_output`, `semantic_handoff_keeps_target_and_session_identity`, CLI `semantic_cli_dump_is_deterministic` |
| DX-005–008 | ARCH-DIAGNOSTIC-001 / ARCH-SECURITY-001 | LANG-DIAG-001 | source diagnostics, work/input/node budgets | `malformed_semantic_inputs_terminate`, `semantic_fuel_limit`, `session_input_budget_rejects_before_parsing`, `pattern_matrix_width_is_charged_before_expansion` |
| PLAT-006, PLAT-009 | ARCH-PLATFORM-001 | LANG-SOURCE-001 | target-independent source spans / explicit target width | source-ID handoff test; Linux/Windows/macOS CI |

## Security review disposition

Closed focused findings: cumulative source/identity and retained-AST budgets;
pattern-subtree duplication and uncharged matrix expansion; wildcard boolean
product enumeration; missing temporary Result tracking; unbounded recursive
Result-shape growth. Regression tests preserve each finding. Rust dependencies
remain empty and crate code forbids unsafe Rust.

Open: broader state-copy/working-memory amplification review, indirect mutation
and loan-stage integration, and final end-to-end completion review. Structural
budgets are not an exact process-memory guarantee. No independent external audit,
fuzz campaign coverage claim or full safety certification is implied.

Fuzz/adversarial entry point: source bytes → `SourceManager::add` → `analyze`, with
small explicit `SemanticOptions` budgets and a caller-enforced wall-clock/process
memory limit. Preserve crashing or hanging inputs as regression fixtures. Existing
malformed-input and budget tests are useful smoke tests, not a substitute for a
sustained fuzz campaign. No new fuzz dependency is required for this boundary.

## Reproducible validation

Run the pinned Rust 1.85.1 commands from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
cargo bench --bench semantic --locked
```

2026-10-13 local Linux: 331 tests pass (168 semantic, 12 CLI, 151 other frontend),
format/Clippy/release build pass. The handoff regression covers both target widths,
reversed source/module ordering, import mapping, source slices, type/resolution IDs
and returned-reference signatures. CLI tests check successful and rejected dumps.

Measured existing release benchmark: 500 functions, 46,280 source bytes, seven
samples per target. 32-bit min/median/max: 4,362/6,189/10,018 microseconds; 64-bit:
4,834/6,127/11,536 microseconds. This includes parsing and implemented semantic
passes; it excludes program execution and is not a cross-language performance claim.
Peak memory was not measured. Final-head remote CI must be recorded in the PR.
