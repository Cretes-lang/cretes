# Frontend review and measured baseline

Review date: 2026-10-05. AI-assisted implementation/self-review, not an independent security audit. Candidate specification pinned in FRONTEND.md.

## Correctness and security review

- Reviewed all 45 grammar productions against parser dispatch/list/precedence behavior; all have mapped tests. Checked empty/nonempty lists, mandatory commas, assignment statements, comparison non-associativity and unsupported future syntax.
- Source offsets preserve original UTF-8 bytes, including CRLF. Boundary-checked slicing and scalar-column mapping prevent invalid Unicode slicing. Invalid encoding is retained for diagnostics. UTF validation is cached to avoid repeated whole-file validation during token range checks.
- Lexing advances on invalid characters. Nested block comments use an iterative depth counter. Depth cannot exceed the number of source bytes. Numeric escape accumulation is bounded to six hex digits before scalar validation. Literals retain spellings and avoid integer value overflow during lexing.
- Parser recursion is bounded; long binary chains have flat ownership. Recovery either consumes input, returns to an enclosing boundary or halts. Node and diagnostic limits stop amplification. Public token input is range-validated and now also enforces source/token budgets; a regression test covers that review finding.
- SourceManager filesystem loading originally used an unrestricted read. It now rejects files beyond the default source budget after reading at most budget+1 bytes. The CLI uses the same bounded-read principle. In-memory callers must bound allocations before adding snapshots.
- No unsafe Rust, third-party Rust dependencies, process execution by parsed programs, network operations, module imports or native library loading occur in the frontend. The crate forbids unsafe code.
- Human diagnostics escape terminal controls, bound displayed strings and omit source snippets. Secondary opening labels are rendered. JSON escapes control characters and uses explicit byte coordinates.
- Tests cover malformed UTF-8, prohibited controls in comments, bare CR, long names/literals, 10,000 nested comments, 128 supported nested parentheses/blocks, 4,000 excessive parentheses, 20,000 operators, 400 deterministic arbitrary-byte inputs and 1,000 random ASCII cases. This is not exhaustive fuzzing.
- CI uses read-only contents permission, no stored checkout credentials, a pinned checkout revision, no secrets, and ordinary pull_request rather than privileged pull_request_target.

## Known limitations and accepted engineering debt

1. The Phase 2/3 design remains provisional until its governance acceptance gates close. This implementation does not accept RFCs.
2. Host allocation failure can still terminate the process. Budgets limit common abuse, but this is not a proof against all denial-of-service behavior. Use host resource isolation for hostile inputs.
3. Scalar column lookup is linear within a line; repeatedly locating positions across a huge single line is potentially quadratic. Optional documentation association scans trivia. Neither is on the parser hot path.
4. Human diagnostics currently show escaped locations/labels/help without source excerpts. Literal values are validated lexically but remain source-backed; typed conversion and semantic checks belong to later work.
5. Node IDs are arena-local and Source IDs session-local, not stable incremental identities across edits. No incremental parser, LSP adapter or sustained fuzz campaign is claimed.
6. Debug AST/token output and the Rust API are experimental. Version 0.1.0 in Cargo is an unpublished development package, not a language release.

## Reproducible performance baseline

Command: `cargo bench --bench baseline`, Rust 1.85.1, optimized build, Linux x86_64 hosted workspace. Fixture: 10,000 repeated function declarations, 420,000 bytes, 180,001 tokens including EOF, 120,001 AST nodes. Duplicate names are intentionally legal syntax; no semantic checking occurs.

Five warm runs of the built benchmark executable, microseconds:

| Run | Lexer | Parser on tokens | Combined frontend | Source mapping |
|---|---:|---:|---:|---:|
| 1 | 13468 | 9037 | 20964 | 479 |
| 2 | 20021 | 9271 | 21128 | 517 |
| 3 | 14771 | 8145 | 21779 | 499 |
| 4 | 11717 | 7747 | 19361 | 480 |
| 5 | 12690 | 7988 | 21909 | 521 |
| Median | 13468 | 8145 | 21128 | 499 |

Mapping performs one query every 31 bytes. Lexer time includes trivia preservation and source-control validation; source snapshot construction is outside timed sections. Combined frontend includes lexing, parsing and diagnostic combination. Benchmarks retain lexical and parsed results simultaneously.

A separate wait4/rusage measurement of the same executable recorded **53,392 KiB peak RSS** and exit 0. This is whole-process peak memory (including simultaneously retained results), not isolated parser memory. Hosted machine scheduling varies; these are regression reference measurements, not product performance claims or cross-language comparisons.

## Validation evidence

Local Linux commands passed: `cargo test --locked` (158 integration tests: 115 conformance/example, 28 robustness, 8 AST shape, 7 CLI), `cargo fmt --all -- --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo build --release --locked`. There are no separate unit/doctest cases; tests exercise the public library and CLI. No test count is inferred from fixture lines.

Cross-platform workflow runs the same commands on ubuntu-latest, windows-latest and macos-latest. Actual remote results must be recorded in the PR before merge; workflow configuration alone is not validation. Full runtime/platform support is outside Phase 4.
