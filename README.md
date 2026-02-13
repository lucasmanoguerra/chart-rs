# chart-rs

`chart-rs` is a Rust-native charting library designed for desktop applications built with GTK4/Relm4.

The project targets behavioral parity with TradingView Lightweight Charts v5.1 using a staged
"core + extensions" strategy while keeping a Rust-idiomatic API.

## Status

Advanced alpha phase. The tracked parity checklist currently reports completed
Core/Render/Extensions blocks for the documented v5.1 scope, with deterministic
tests and CI gates in place. Ongoing work is focused on parity hardening
(visual fixture validation, default/boundary tuning, and UX-level refinements).

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

## Developer Docs

- `docs/developer-guide.md`
- `docs/architecture.md`
- `docs/parity-v5.1-checklist.md`

## Getting Started

```bash
cargo test-core
cargo test-visual
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

## Automation Script

To automate the full feature-to-release alpha flow (checks, PRs, merge, release PR, prerelease):

```bash
scripts/ship_alpha.sh \
  --feature-branch feat/r0xx-your-feature \
  --feature-commit "feat(render): add R-0xx your feature" \
  --feature-pr-title "feat(render): add R-0xx your feature"
```

The script retries transient `gh` API connectivity errors and waits for GitHub checks automatically.
