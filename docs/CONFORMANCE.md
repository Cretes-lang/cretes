# Grammar conformance and traceability

Baseline: spec `fe6d0d48518d0419d59fb680673150c92b6a084c`, docs/language/cretes.ebnf and SPEC.md. All 45 syntactic productions have implementation and test links below. This is a review of the published candidate, not formal language acceptance or a proof of full semantic correctness.

Functions listed below are in `compiler/parser.rs`; tests are in `tests/conformance.rs`, `tests/ast_shape.rs` or `tests/robustness.rs`. Reusable list parsing accounts for trailing commas and empty/nonempty distinctions.

| Production | Parser component | Representative tests |
|---|---|---|
| `program` | `program` | `fixture_000_empty; modules_and_import_alias` |
| `module_decl` | `module` | `modules_and_import_alias` |
| `import_decl` | `import` | `fixture_007_import_alias; modules_and_import_alias` |
| `path` | `path` | `modules_and_import_alias; fixture_011_patterns` |
| `item` | `item` | `function_types_and_visibility; user_types_and_patterns` |
| `function` | `function` | `fixture_003_trailing_parameters; function_types_and_visibility` |
| `parameters` | `list / parameter` | `fixture_003_trailing_parameters` |
| `parameter` | `parameter` | `function_types_and_visibility` |
| `type` | `ty / ty_inner` | `fixture_004_nested_types; function_types_and_visibility` |
| `types` | `list / ty` | `fixture_004_nested_types; fixture_008_enum` |
| `const_decl` | `constant` | `fixture_025_constant` |
| `alias_decl` | `alias` | `example_07` |
| `record_decl` | `record` | `user_types_and_patterns` |
| `field_decl` | `record` | `fixture_006_pub_fields; user_types_and_patterns` |
| `enum_decl` | `enumeration` | `fixture_008_enum; user_types_and_patterns` |
| `variant` | `variant` | `fixture_008_enum` |
| `block` | `block / block_inner` | `fixture_024_nested_blocks; minimum_supported_block_nesting` |
| `statement` | `statement` | `fixture_023_loops; recovery_keeps_later_function` |
| `binding` | `statement` | `example_02` |
| `return_stmt` | `statement` | `function_types_and_visibility; example_03` |
| `if_stmt` | `if_stmt / if_inner` | `fixture_010_else_chain; example_04` |
| `while_stmt` | `statement` | `fixture_023_loops; example_05` |
| `for_stmt` | `statement` | `example_05` |
| `loop_stmt` | `statement` | `fixture_023_loops` |
| `match_stmt` | `statement` | `user_types_and_patterns; example_08` |
| `arm` | `statement` | `fixture_011_patterns; example_08` |
| `pattern` | `pattern / pattern_inner` | `fixture_011_patterns; fixture_012_wildcard` |
| `patterns` | `list / pattern` | `fixture_011_patterns` |
| `expression` | `expression / binary` | `all_binary_precedence_boundaries` |
| `logic_or` | `binary / binary_inner / precedence` | `all_binary_precedence_boundaries; left_associative_operators; fixture_036_chained_comparison; fixture_037_chained_equality` |
| `logic_and` | `binary / binary_inner / precedence` | `all_binary_precedence_boundaries; left_associative_operators; fixture_036_chained_comparison; fixture_037_chained_equality` |
| `bit_or` | `binary / binary_inner / precedence` | `all_binary_precedence_boundaries; left_associative_operators; fixture_036_chained_comparison; fixture_037_chained_equality` |
| `bit_xor` | `binary / binary_inner / precedence` | `all_binary_precedence_boundaries; left_associative_operators; fixture_036_chained_comparison; fixture_037_chained_equality` |
| `bit_and` | `binary / binary_inner / precedence` | `all_binary_precedence_boundaries; left_associative_operators; fixture_036_chained_comparison; fixture_037_chained_equality` |
| `equality` | `binary / binary_inner / precedence` | `all_binary_precedence_boundaries; left_associative_operators; fixture_036_chained_comparison; fixture_037_chained_equality` |
| `comparison` | `binary / binary_inner / precedence` | `all_binary_precedence_boundaries; left_associative_operators; fixture_036_chained_comparison; fixture_037_chained_equality` |
| `shift` | `binary / binary_inner / precedence` | `all_binary_precedence_boundaries; left_associative_operators; fixture_036_chained_comparison; fixture_037_chained_equality` |
| `sum` | `binary / binary_inner / precedence` | `all_binary_precedence_boundaries; left_associative_operators; fixture_036_chained_comparison; fixture_037_chained_equality` |
| `product` | `binary / binary_inner / precedence` | `all_binary_precedence_boundaries; left_associative_operators; fixture_036_chained_comparison; fixture_037_chained_equality` |
| `unary` | `unary / unary_inner` | `unary_and_postfix_structure; fixture_022_borrow_deref` |
| `postfix` | `postfix` | `unary_and_postfix_structure; fixture_017_chains` |
| `arguments` | `list / expression` | `example_03; fixture_013_unit_tuple` |
| `primary` | `primary` | `grouping_overrides_precedence; fixture_013_unit_tuple; example_06` |
| `field_values` | `list / field_value` | `user_types_and_patterns` |
| `literal` | `literal / primary` | `fixture_019_numeric_bases; fixture_020_floats; fixture_021_literals` |

## Lexical coverage

`compiler/lexer.rs` and `compiler/token.rs` implement lexical.ebnf: whitespace, all 26 keywords, all 34 punctuation/operator spellings, ASCII identifiers and wildcard, nested comments and documentation comments, integer bases/separators, decimal/exponent floats, strings, character scalars and byte strings. `every_keyword_and_prefix` and `every_operator_and_longest_match` cover the entire fixed inventories. Conformance fixtures 019–022 and 064–084 cover literals and lexical errors; robustness tests add UTF-8, raw controls, CRLF, longest inputs and malformed streams.

Tests embed original fixture source bytes, including actual prohibited controls. The 101 fixtures comprise 32 syntax-valid, 31 syntax-invalid, 22 lexical-invalid and 16 semantic-negative cases. All semantic-negative fixtures must parse: this prevents accidental implementation of Phase 5. The 14 examples are unmodified copies with names preserved.

## Phase 1 → Phase 2 → Phase 3 → Phase 4

IDs are inherited from the spec repository's requirements, architecture and language traceability documents. This table records frontend contribution only, never satisfaction of a whole semantic/runtime requirement.

| Phase 1 | Phase 2 | Phase 3 | Implementation → evidence |
|---|---|---|---|
| CORE-001–003 | ARCH-PIPELINE-001 | LANG-SOURCE-001 | SourceManager/Span, UTF validation → UTF-8 and CRLF robustness tests |
| CORE-004–005 | ARCH-PIPELINE-001 | LANG-IDENT-001 | lexer identifier/source-control handling → keyword/prefix, bidi and reserved-name fixtures |
| CORE-006 | ARCH-TYPE-001 | LANG-LITERAL-001 | distinct literal token kinds → literal tests; representation semantics deferred |
| CORE-007–008 | ARCH-MODULE-001 | LANG-MODULE-001 | module/import/item AST → modules_and_import_alias; resolution/privacy deferred |
| CORE-011, CORE-020 | ARCH-TYPE-001 | LANG-MUT-001, LANG-DECL-001 | binding/reference/assignment syntax → examples 02/10; mutability/type inference deferred |
| CORE-012 | ARCH-TYPE-001 | LANG-RECORD-001 | record/enum/alias AST → user_types_and_patterns |
| CORE-013 | ARCH-V0.1-ARCHITECTURE-001 | LANG-EXPR-001 | explicit precedence AST → all_binary_precedence_boundaries; runtime evaluation deferred |
| CORE-014 | ARCH-TYPE-001 | LANG-LOOP-001 | statement/control flow parser → example_04/example_05; control-flow validation deferred |
| CORE-015, CORE-027 | ARCH-TYPE-001 | LANG-FUNC-001 | function signatures/body/provenance → function_types_and_visibility; entrypoint validation deferred |
| CORE-018, CORE-033 | ARCH-TYPE-001 | LANG-TYPE-001 | type arguments, tuples, references → nested_types and collections; type definitions/semantics deferred |
| CORE-022–024 | ARCH-ERROR-001 | LANG-ERROR-001 | propagation/calls/match → example_08; Result semantics deferred |
| CORE-025 | ARCH-COMPILER-001 | LANG-MODULE-001 | deterministic arena/diagnostics → deterministic_dump and CLI tests |
| CORE-031 | ARCH-COMPAT-001 | LANG-BASELINE-001 | 45-production mapping + 101-fixture suite + 14 canonical examples |
| DX-005–008 | ARCH-DIAGNOSTIC-001 | LANG-DIAG-001 | structured codes/spans, JSON, recovery, limits → CLI and robustness suites |
| PLAT-006, PLAT-009 | ARCH-PLATFORM-001 | LANG-SOURCE-001 | original bytes, OS paths, LF/CRLF fixtures → frontend CI matrix; full platform support not implied |

Generic declarations, async/concurrency and user FFI/unsafe constructs are explicitly post-v0.1. Built-in type argument syntax remains applicable. No syntax was changed to fit the implementation. Future changes must update the normative source through its RFC/change process, then this mapping and tests.
