# AGENTS.md

## Mission
This repository builds a Rust-native desktop charting engine intended for reuse in GTK4/Relm4 applications.

Primary technical stack:
- `cairo-rs`
- `pango`
- `pangocairo`

Product goal:
- Deliver a feature-complete behavioral replica of TradingView Lightweight Charts v5.1 through a phased roadmap.
- Preserve Rust idioms and architecture quality while matching interaction, rendering, and logic outcomes.

## Product Strategy: Core + Extensions
Parity with Lightweight Charts v5.1 is delivered in two tracks.

1. Core (mandatory)
- Time scale and price scale behavior
- Core series primitives (line, area, candlestick)
- Autoscale and viewport fitting
- Crosshair basic behavior
- Deterministic rendering behavior for supported scenarios

2. Extensions (optional, feature-gated)
- Advanced markers and overlays
- Plugin-like/custom behaviors
- Non-essential advanced interaction patterns

Each parity item must include:
- Reference behavior from v5.1
- Explicit acceptance criteria
- Linked tests that prove expected behavior

## Engineering Principles
Follow UNIX philosophy and Rust-first design.

- Single responsibility per module.
- Explicit interfaces between modules.
- Composition over tight coupling.
- Side effects isolated in adapter/platform layers.
- No hidden panics in public API paths.
- Fallible operations return explicit error types.
- Keep mutable shared state minimal and intentional.

## Architecture Contract
Use clean dependency direction and strict layer boundaries.

Layers:
- `core`: domain models, scale math, series state, viewport math
- `render`: rendering pipeline abstractions and cairo/pango implementation
- `interaction`: event-to-intent state machine (pan, zoom, crosshair, etc.)
- `api`: public Rust-idiomatic facade consumed by host applications
- `platform_gtk`: GTK4/Relm4 integration adapter (widget lifecycle and event bridging)
- `extensions`: optional capabilities behind Cargo features

Dependency direction:
- `api` may depend on `core`, `render`, `interaction`
- `platform_gtk` may depend on `api`
- `extensions` may depend on `core`, `render`, `interaction`, `api`
- `core` must not depend on `platform_gtk`
- `render` and `interaction` must not depend directly on GTK-specific code

Forbidden coupling:
- Mixing rendering + interaction + domain logic in one module
- Leaking platform concerns into core domain logic

## Public API Policy
Public API must be Rust-idiomatic.

- Do not mirror JavaScript API shape 1:1 as the primary interface.
- Prioritize safety, type clarity, and explicit semantics.
- Keep semver discipline for public API changes.
- Any externally visible behavior change requires documentation updates.

Preferred public surface categories:
- Engine lifecycle (`ChartEngine`-style)
- Series handles and typed data updates
- Viewport/scale control API
- Interaction controller API
- Renderer abstraction with cairo-backed implementation

## Testing Policy (Mandatory)
Every feature must ship with a complete test set.

Required coverage categories:
- Unit tests for local invariants and logic
- Integration tests for cross-module behavior
- Property tests using `proptest` for invariant robustness
- Interaction scenario tests for event sequences
- Rendering regression tests with deterministic fixtures and tolerance policy

If a change affects performance-critical paths:
- Add or update `criterion` benchmarks.

Rule:
- No feature merges without tests that verify its behavior.

## CI Quality Gates
PR-blocking quality gates are strict except benchmarks.

Required on every PR:
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- Property tests must run in normal CI test flow

Benchmarks:
- `criterion` benchmarks are required for performance-sensitive areas.
- Benchmarks run in scheduled/nightly/release workflows (non-blocking for standard PRs).

## Definition of Done (Per Change)
A change is done only if all are true:
- Module responsibility remains narrow and clear.
- Architecture boundaries remain respected.
- Required tests are added/updated and passing.
- Public docs are updated for visible behavior changes.
- If parity-related, mapped checklist item includes evidence.
- CI gates pass without warnings or formatting drift.

## Repository Conventions
- Prefer library-first design for reuse.
- Keep binaries/examples as thin wrappers around library APIs.
- Use clear naming for responsibilities:
  - `*_core`
  - `*_renderer`
  - `*_controller`
  - `*_adapter`
- Keep Cargo features explicit, documented, and minimal.

## Parity Tracking Requirements
Maintain a parity checklist for Lightweight Charts v5.1.

Each checklist entry should record:
- Feature name and scope
- Status (`not started`, `in progress`, `done`, `blocked`)
- Source references to expected behavior
- Test IDs / files proving parity
- Known deviations and rationale

## Required Test Scenarios
At minimum, ensure coverage for:
- Scale invariants (monotonic mapping, numeric stability)
- Price/time conversion round-trip behavior within tolerance
- Data stream updates (append/replace/history updates)
- Pan/zoom anchoring and boundary behavior
- Crosshair snapping behavior
- Rendering determinism for fixed fixtures
- Sparse/missing/extreme dataset robustness with `proptest`
- Performance budgets (append throughput, redraw latency) via `criterion`

## Non-Goals and Guardrails
- Do not introduce web-runtime assumptions into core logic.
- Do not couple core internals directly to GTK widgets.
- Do not bypass architecture boundaries for short-term speed.
- Do not merge behavior changes without explicit acceptance criteria.

## PR Expectations
Each PR must include:
- Behavioral summary of what changed
- Which parity checklist items were touched
- Tests added/updated by category
- Any deliberate deviation from v5.1 behavior and rationale
