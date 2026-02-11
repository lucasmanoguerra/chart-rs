# chart-rs

`chart-rs` is a Rust-native charting library designed for desktop applications built with GTK4/Relm4.

The project targets behavioral parity with TradingView Lightweight Charts v5.1 using a staged
"core + extensions" strategy while keeping a Rust-idiomatic API.

## Status

Early bootstrap phase. The architecture, quality gates, and contributor workflow are in place.
Core rendering and interaction parity work is ongoing.

## Design Goals

- Modular architecture with strict responsibility boundaries
- Rust-idiomatic and safe public API
- Reusable engine for GTK4/Relm4 desktop applications
- Deterministic rendering and interaction behavior
- Strong test coverage for every feature

## Core Stack

- `cairo-rs`
- `pango`
- `pangocairo`

## Repository Layout

- `src/core` domain models, scales, and deterministic math
- `src/render` rendering contracts and backend glue
- `src/interaction` event/state interaction layer
- `src/api` public crate interface
- `src/platform_gtk` GTK4 adapter (feature-gated)
- `src/extensions` optional advanced capabilities
- `tests` integration and property tests
- `benches` criterion benchmarks
- `docs` architecture and parity tracking docs

## Getting Started

```bash
cargo test
cargo test --all-features
cargo bench --bench core_math_bench
```

## Quality Gates

Pull requests are expected to pass:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`

## Versioning

This project follows SemVer and Keep a Changelog.

See:
- `CHANGELOG.md`
- `CONTRIBUTING.md`
- `AGENTS.md`
