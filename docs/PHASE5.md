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

- `cargo test --locked`: 276 tests passed (158 existing + 118 semantic).
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed.

No new dependencies. Existing platform CI automatically includes the semantic
tests. The earlier checkpoint `1cecb3f3ad2082f59b354d6441ea866599e18464` passed all
six push/PR checks on Linux, Windows and macOS (verified 2026-10-08). Later
changes require their own CI confirmation; these local results do not imply it.

## Remaining work and known limitations

- Implement bound Result obligations and expand control-flow integration tests.
- Complete ownership/loan-stage integration and resource review throughout
  recursive helpers. Direct return provenance is now checked, but there is no
  completed borrow checker.
- Finish a typed handoff that exposes all later-stage metadata and a documented
  inspection command without implying production `cretes check` semantics.
- Expand canonical semantic validation using explicit test-only contracts for
  conceptual standard-library APIs; the existing 14 examples remain syntax tests.
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
