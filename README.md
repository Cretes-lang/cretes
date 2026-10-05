# Cretes

The Cretes Programming Language — automation, networking, AI/ML applications and cybersecurity.

This repository contains the **provisional Phase 4 syntax frontend** for `.cretes` sources. It implements the published Phase 3 candidate, not an accepted or stable language release. Phase 2/3 RFC acceptance remains a separate governance gate. There is no program execution, semantic analyzer, type checker, IR, backend or runtime.

## Build and inspect

Install the pinned Rust 1.85.1 toolchain (see `rust-toolchain.toml`). No third-party Rust dependencies are needed.

```sh
cargo build --locked
cargo test --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo run --bin cretes-front -- lex examples/01-hello-world.cretes
cargo run --bin cretes-front -- parse examples/01-hello-world.cretes
cargo run --bin cretes-front -- parse examples/08-errors.cretes --json-diagnostics
cargo bench --bench baseline
```

`cretes-front` is an experimental developer utility. `lex` prints tokens; `parse` prints the arena AST. Diagnostics go to stderr. Exit codes: 0 for successful syntax processing, 1 for source diagnostics, 2 for usage/I/O failures. Syntax success does not establish semantic validity. Debug output and Rust APIs are not stable compatibility promises.

## Implementation and evidence

- [Frontend architecture, API, diagnostics and limits](docs/FRONTEND.md)
- [Grammar coverage and requirements traceability](docs/CONFORMANCE.md)
- [Source → tokens → AST demonstrations](docs/DEMONSTRATIONS.md)
- [Security review and performance measurements](docs/REVIEW.md)
- [Phase 4 implementation status and handoff](docs/PHASE4.md)

Canonical source: [Phase 3 candidate](https://github.com/Cretes-lang/spec/tree/fe6d0d48518d0419d59fb680673150c92b6a084c/docs/language). The 14 examples are byte-for-byte copies. Domain examples validate syntax only; their APIs are not implemented.

## Contributing

Follow the organization [contribution rules](https://github.com/Cretes-lang/.github/blob/main/CONTRIBUTING.md), [governance](https://github.com/Cretes-lang/.github/blob/main/GOVERNANCE.md), [engineering standards](https://github.com/Cretes-lang/.github/blob/main/ENGINEERING.md), [code of conduct](https://github.com/Cretes-lang/.github/blob/main/CODE_OF_CONDUCT.md) and [security reporting policy](https://github.com/Cretes-lang/.github/blob/main/SECURITY.md). Open an issue, use a feature branch and submit a tested PR. Changes to language design require the RFC process; do not adapt the specification to parser bugs.

License: [Apache-2.0](LICENSE). Initial maintainer: @krishanth7.
