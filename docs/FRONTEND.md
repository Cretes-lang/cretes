# Frontend implementation

## Baseline and scope

The baseline is the Phase 3 candidate in spec commit `fe6d0d48518d0419d59fb680673150c92b6a084c` (content commit `5a50ab3dd9fceb3918e4dabeea750bf92f17eb1b`). Phase 2 ARCH-COMPILER-001, ARCH-PIPELINE-001 and ARCH-DIAGNOSTIC-001 guide this provisional implementation. Their RFC acceptance is not implied. No specification files or RFC statuses are changed here.

A single dependency-free Rust 2021 library crate keeps modules separate without introducing empty workspace crates. The companion `cretes-front` binary is only a development inspector. Rust 1.85.1 is pinned for reproducible bootstrap; upgrades need review and CI.

## Ownership and API

`SourceManager` owns immutable source snapshots with session-local, append-only `SourceId`s. In-memory `add` accepts arbitrary bytes; `load` reads a physical path with the default 16 MiB limit. Invalid UTF-8 remains representable and produces L001 when lexed. Valid UTF-8 is checked once and cached. A line-start index supports location lookup without changing CRLF or original source bytes.

```rust
use cretes_frontend::{parse_source, source::SourceManager, Limits};
let mut sources = SourceManager::default();
let id = sources.add("memory.cretes", b"fn main() -> i32 { return 0; }".as_slice());
let result = parse_source(sources.get(id).unwrap(), Limits::default());
assert!(result.is_valid());
assert!(result.ast.root.is_some());
```

IDs belong to one manager/session; keep the manager alive alongside results. They are not globally unique identities across managers or edits. AST `NodeId`s index one arena, remain stable during that parse and are not persistent across reparses. All nodes use half-open original UTF-8 byte spans. `SourceFile::slice` validates boundaries and source IDs; `location` returns one-based line/Unicode-scalar column, not display width or UTF-16 positions.

`lexer::lex` returns tokens, preserved trivia and diagnostics. `parser::parse` validates a supplied token stream, then returns AST and diagnostics. `parse_source` combines both, sorts diagnostics by source/offset/code and applies the budget. Callers must inspect diagnostics; a recovered tree can contain `Error` nodes. Resource aborts can omit the root.

## Lexer and tokens

Fixed keywords and punctuation have explicit `Kind` variants and centralized inventories. Identifiers and literals retain source spans instead of copied strings. Exact spellings, including digit separators and escapes, are available through the source snapshot. Literal values are not numerically evaluated or type checked.

The scanner performs a bounded source-control pass and a forward lexical scan. It uses longest-match punctuation, ASCII identifiers, case-sensitive keywords, base-prefixed/decimal/exponent numbers, validated separators, strings, Unicode scalar characters and byte strings. Whitespace, line comments, documentation comments and nested block comments are preserved as trivia. Prohibited raw controls and bidi/invisible scalars are rejected even inside comments. No Unicode identifier normalization is introduced.

`TokenStream` has current/peek/advance/position operations; EOF is stable under lookahead and consumption. Lexing always returns a final EOF, including on malformed input or resource failure.

## Parser and AST

Keyword-dispatched recursive descent handles declarations, types, statements and patterns. Precedence climbing uses one operator table; unary and postfix parsing supply the two highest tiers. Comparisons/equality are non-associative unless explicitly grouped. Assignment is a statement, not an expression.

The AST is a flat, strongly typed arena with separate variants for declarations, statements, expressions, types and patterns. Edges are node IDs, so destroying or dumping a long binary chain does not recursively traverse ownership. Names and major components retain spans. Parentheses are retained as `Group` where they explain explicit association. Public item wrappers preserve visibility. Documentation trivia can be associated with item/field nodes through `Ast::documentation`, stopping at blank lines or other comments.

Functions preserve parameter mutability, return syntax and `from` provenance names. Type syntax includes paths, square-bracket arguments, tuples and references. It does not establish whether names, type constructors or arities are legal. User-defined generic declarations, async/concurrency, FFI and unsafe syntax are deferred by the candidate and are not implemented. Built-in generic-shaped type syntax is parsed without semantic validation.

Recovery synchronizes at semicolons, braces and known declaration/statement starts, always advancing or returning to an enclosing boundary. Failed constructs can yield explicit `Error` nodes while later declarations remain available. Missing closing delimiters retain a secondary opening label.

## Diagnostics registry

Codes follow the Phase 3 candidate. They describe syntax failures, not semantic errors.

| Code | Category |
|---|---|
| L001 | Invalid encoding or prohibited raw source scalar/bare CR |
| L002 | Invalid source character or reserved internal identifier |
| L003 | Malformed numeric literal |
| L004 | Invalid/unterminated string, character or byte literal |
| L005 | Unterminated block comment |
| P001 | Expected syntax/token or malformed token stream |
| P002 | Keyword used where an identifier is required |
| P003 | Non-associative comparison/equality chain |
| R001 | Source/token/node/nesting resource budget exceeded |

Diagnostics contain severity, primary span, secondary labels, notes and help. Human output escapes controls and limits displayed messages; it reports locations and opening labels without printing source snippets. JSON-lines output has schema_version 1, UTF-8 byte offsets, half-open ranges, source IDs and structured labels/notes. The caller owns the ID-to-path mapping. No unsafe automatic fix is offered.

## Limits and complexity

Defaults: 16 MiB source, 1,000,000 tokens plus trivia, 1,000,000 AST nodes, 100 diagnostics, 512 internal recursive parser entries. Diagnostic configuration is clamped to 1–1000, recursion to 1–512. These are engineering limits, not language grammar rules. Tests support at least 1 MiB sources, 128 nested parentheses/blocks, long identifiers/literals and 20,000-operator chains. More complicated nesting consumes more internal entries.

File loading and CLI reads are bounded. In-memory `add` deliberately accepts caller-owned snapshots; hosts must bound untrusted input before allocating/storing it. Memory allocation failure is not converted into a recoverable diagnostic; use process/container memory limits for hostile workloads. Avoid configuring enormous budgets.

Lexing and parsing are approximately linear in input/token count with fixed operator/keyword inventories. Line lookup uses binary search plus scalar counting within the selected line; repeated mapping across a single enormous line can be expensive. Optional documentation association scans trivia; do not query every node repeatedly on large files. Neither operation is on the main parser path.

## Testing and extension

Run the README commands. `tests/conformance.rs` embeds the 101 Phase 3 fixtures, including 16 semantic-negative programs that must parse successfully, and parses all 14 original examples. `ast_shape.rs` checks precedence and important tree structures. `robustness.rs` covers lexical inventories, UTF-8, nesting, recovery, spans, budgets and deterministic random input. `cli.rs` checks golden tokens, JSON fields, exit codes, deterministic AST output and bounded loading.

Fuzz entry points are `SourceManager::add` + `parse_source` for arbitrary bytes, and `lexer::lex` for lexical isolation. A future cargo-fuzz harness can call these without filesystem or runtime behavior. Current deterministic stress tests are preparation, not a claim of exhaustive fuzzing.

Phase 5 may consume the source snapshots, arena AST, node IDs and diagnostics. Name resolution, module loading, scopes, type checking, borrow rules, generic validation and symbol tables remain unimplemented.
