# Phase 5 provisional implementation checkpoint

Status: **in progress; not complete or accepted**. Recorded 2026-10-09.

This branch implements against Phase 3 candidate spec revision
`fe6d0d48518d0419d59fb680673150c92b6a084c`. Phase 2/3 acceptance remains
pending. It does not update RFC status or begin Phase 6.

## Current implementation

`compiler/semantic` exposes `analyze(&[ModuleInput], SemanticOptions)`.
The caller supplies a logical module identity and an existing source snapshot.
No filesystem discovery, manifest schema, standard library or implicit unknown
symbol acceptance is introduced. Modules are sorted before collection.

- Stable session module/symbol/scope IDs, lexical scopes and two-pass declarations.
- Interned types, explicit target pointer width (32/64), nominal records/enums,
  transparent aliases, recursion/layout checks, visibility and public interfaces.
- Exact type compatibility, contextual numeric literals, calls, assignments,
  control structures, Result propagation, Option/Result constructors and patterns.
- Recursive pattern-matrix usefulness/exhaustiveness checking.
- Reachable exit sets for branches, matches and loops, distinguishing return,
  break, continue, fallthrough and divergence. Literal boolean conditions are
  recognized without skipping type checks in unreachable syntax.
- Direct-reference return provenance follows aliases, reassignments, projections,
  calls with `from`, borrowed patterns, iterator bindings, branch joins and loop
  back edges. State-copy work uses the shared semantic fuel budget. This checks
  return contracts; it is not the later full owner/loan validity analysis.
- `analyze_entry` validates an explicitly selected executable module's `main`
  contract. Library analysis remains entry-free; absent inputs return an API error.
- Restricted, fuel-bounded constant evaluation with integer overflow, shift,
  division and dependency-cycle diagnostics. Constant syntax is checked even in
  short-circuited operands.
- Source-linked diagnostics using the candidate M001/T001/T002/B002/C001/R001
  conventions. Shared Phase 4 source, spans, AST and diagnostic infrastructure.
- Map/Set built-in key restrictions and aggregate reference-storage restrictions.
- Iterative place/type/layout traversal avoids recursive stack growth and repeated
  expansion of shared type graphs; duplicate-declaration labels respect diagnostic limits.
  Type mismatch formatting also bounds output and expansion work. Enum constructor
  lookup respects local shadowing, type qualifiers and exact path lengths.

The result retains ASTs, symbols/scopes, interned types, expression type tables,
resolved-symbol tables, public resolved function contracts (including `from`),
record/enum shapes, value categories, constants and diagnostics. `is_valid()` currently means
no diagnostics from the implemented name/type checks; **it does not certify
move/loan safety or full language validity**. Rust APIs and dumps remain experimental.

## Verified local checkpoint

Rust 1.85.1, Linux, 2026-10-09:

- `cargo test --locked`: 313 tests passed (150 semantic + 12 CLI + 151 other frontend).
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed.

No new dependencies. Existing platform CI automatically includes the semantic
tests. The earlier checkpoint `1cecb3f3ad2082f59b354d6441ea866599e18464` passed all
six push/PR checks on Linux, Windows and macOS (verified 2026-10-08). Later
changes require their own CI confirmation; these local results do not imply it.

## Remaining work and known limitations

- Complete projection-aware Result obligations and collection iteration flow.
- Complete ownership/loan-stage integration and resource review throughout
  recursive helpers. Direct return provenance is now checked, but there is no
  completed borrow checker.
- Finish a typed handoff that exposes all later-stage safety metadata. The
  experimental inspection command is implemented; it is not `cretes check`.
- Expand adversarial semantic cases beyond the 14 canonical examples now covered
  with explicit test-only contracts for conceptual standard-library APIs.
- Complete semantic traceability, fuzz/adversarial review, measured performance
  baseline, cross-platform CI and full integration review.
- Reconcile any accepted specification changes before formal closure.

No completed Phase 5 gate, production safety claim, runtime, MIR, backend or
native code generation is claimed by this checkpoint.

## 2026-10-09 focused review

Added 21 regression tests covering entry contracts, enum qualifier resolution,
shared-type diagnostic expansion, and direct-reference provenance. Tests include
wrong owners/local storage, branch reassignment, unreachable edges, loop back
edges, zero-iteration paths, borrowed match payloads and iterator references.
The provenance pass uses finite origin sets with union at joins and a monotone
loop fixed point. It rejects a reachable return unless its only origin is the
specified source parameter. Full lifetime and move validity remain a separate
unimplemented safety boundary; success must not be advertised as memory safety.

The PR was marked ready for review by the maintainer after the previous
checkpoint. That UI status does not satisfy the three unchecked completion
gates. New commits require fresh review and their own cross-platform CI evidence.

## Further integration checkpoint — 2026-10-09

Local suite: **313 passing tests** (150 semantic, 12 CLI, 151 other frontend).
Formatting, Clippy with warnings denied and release build passed locally for
this revision; earlier cross-platform checks do not cover these edits.

The first whole-owner Result dataflow pass diagnoses unused bindings/parameters,
overwrite, scope exit, return/propagation exits, branch joins, loop exits,
wildcard dropping and transfers through whole aggregates. This pass is not yet
fully conformant: it collapses aggregate payload obligations to one owner bit.
Partial field updates, mixed empty/nonempty nested payloads and collection
iteration therefore need more precise projection tracking. These are genuine
completion blockers, not new language restrictions. Do not treat a conservative
T003 rejection in these cases as a normative language rule.

`cretes-front analyze --target-bits 64 --module app path.cretes --entry app`
provides experimental semantic inspection. Repeat `--module identity path` for
an explicit module map; omit `--entry` for library analysis. JSON diagnostics
are optional. Exit codes are 0 (implemented checks pass), 1 (source diagnostics),
2 (usage/I/O failure). Exit 0 does not certify full validity or move/loan safety.
No implicit library, package manifest or filesystem search is introduced.

All 14 unchanged canonical examples now pass the implemented semantic checks.
Nine are self-contained. Five use explicit test-only library contracts in
`tests/semantic.rs`; the collection signatures are concrete for those examples,
not generic standard-library implementations. Divergent contract bodies exist
only to supply test signatures. They do not implement library behavior.

### Measured baseline

`cargo bench --bench semantic --locked`, Rust 1.85.1, release, Linux,
2026-10-09. One source, 46,280 bytes, 500 distinct functions, seven samples.
End-to-end `analyze` includes parsing, type checking, constants and dataflow.

| Target width | Minimum | Median | Maximum |
| --- | ---: | ---: | ---: |
| 32 | 6,689 us | 8,985 us | 18,162 us |
| 64 | 5,931 us | 6,673 us | 7,293 us |

These are local observations, not portable performance promises. Memory use was
not measured. The fixture is reproducible in `benchmarks/semantic.rs`.

### Review findings still open

- Replace whole-owner Result bits with projection-aware obligations, including
  collection iteration and early loop exits. Keep positive conformance tests for
  empty nested payloads so conservative false rejections cannot hide regressions.
- Complete cumulative session memory budgeting across module inputs and audit
  pattern-matrix/state-copy amplification. Per-file/fuel limits are not a full
  memory-exhaustion defense. Do not claim a completed security review.
- Finish the documented handoff to the separate ownership/loan stage and verify
  that no consumer mistakes these partial checks for a full safety certificate.
- Complete the final requirements traceability/diff review and obtain checks on
  the eventual final revision, then reconcile accepted Phase 2/3 decisions.

The three Phase 5 completion gates remain unchecked.
