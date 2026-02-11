# Developer Guide

This guide is the practical, code-oriented reference for contributors.

For governance and quality rules, read:
- `AGENTS.md`
- `CONTRIBUTING.md`

For architecture and parity status, read:
- `docs/architecture.md`
- `docs/parity-v5.1-checklist.md`

## 1) Local Setup

Required tooling:
- Rust stable (see `rust-toolchain.toml`)
- system dependencies for cairo/pango/gtk4 when running `--all-features`

Primary local checks:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## 2) Module Map

### `src/core`
Domain layer with deterministic math and model invariants.

- `types.rs`
  - `Viewport`
  - `DataPoint`
- `scale.rs`
  - `LinearScale` (generic domain/pixel mapper)
- `time_scale.rs`
  - `TimeScale`
  - `TimeScaleTuning`
  - visible/full range logic
- `price_scale.rs`
  - `PriceScale`
  - `PriceScaleTuning`
  - autoscale tuning + inverted Y mapping
- `candlestick.rs`
  - `OhlcBar`
  - `CandleGeometry`
  - `project_candles`
- `line_series.rs`
  - `LineSegment`
  - `project_line_segments`
- `area_series.rs`
  - `AreaVertex`
  - `AreaGeometry`
  - `project_area_geometry`
- `primitives.rs`
  - `chrono` and `rust-decimal` helpers for strongly-typed construction

Core invariants:
- all public numeric inputs must be finite
- invalid ranges return `ChartError`, never panic
- mapping behavior is deterministic for fixed input

### `src/interaction`
Pointer and interaction state machine.

- `InteractionMode`
- `CrosshairState`
- `InteractionState`

Interaction invariants:
- pointer move enables crosshair visibility
- pointer leave clears visibility and snap state
- snapping is decided in `api` using mapped chart data/candles

### `src/api`
Main public facade (`ChartEngine`, `ChartEngineConfig`).

Responsibilities:
- orchestration between core + interaction + renderer
- time visible range controls and fit-to-data
- price autoscale from points/candles (default and tuned)
- crosshair snapping behavior

### `src/render`
Renderer trait boundary and backend implementations.

- `RenderFrame`
- `Renderer`
- `NullRenderer`
- feature-gated cairo backend

## 3) Data Flow

Typical runtime flow:

1. create engine with initial time/price domains
2. push/update points and/or candles
3. call fit/autoscale APIs to derive tuned domains
4. pointer events update crosshair and snapping state
5. renderer consumes immutable frame data

## 4) Scale Tuning Details

### Time Scale
`TimeScaleTuning` parameters:
- `left_padding_ratio`
- `right_padding_ratio`
- `min_span_absolute`

Fit behavior:
- computes full range from points/candles
- expands degenerate range with `min_span_absolute`
- applies left/right padding to produce visible range

### Price Scale
`PriceScaleTuning` parameters:
- `top_padding_ratio`
- `bottom_padding_ratio`
- `min_span_absolute`

Autoscale behavior:
- computes min/max from points or candle low/high
- expands degenerate range with `min_span_absolute`
- applies top/bottom padding
- keeps inverted Y behavior (`higher price` => `smaller pixel y`)

## 5) Crosshair Baseline Behavior

Current baseline logic:
- pointer move updates crosshair x/y and enables visibility
- engine computes nearest X candidate from:
  - line/point series
  - candle close prices
- nearest candidate selected with `smallvec` fixed-capacity hot-path buffer
- pointer leave hides crosshair and clears snap coordinates

## 6) Testing Playbook

Where to add tests:

- `tests/core_scale_tests.rs`
  - deterministic scale behavior and edge cases
- `tests/property_scale_tests.rs`
  - invariant properties for tuned/round-trip mapping
- `tests/candlestick_tests.rs`
  - deterministic candle geometry and autoscale behavior
- `tests/property_candlestick_tests.rs`
  - wick/body ordering and geometry invariants
- `tests/line_series_tests.rs`
  - deterministic line segment geometry and visible-window mapping
- `tests/property_line_series_tests.rs`
  - line projection count and finite-geometry invariants
- `tests/area_series_tests.rs`
  - deterministic area line/fill geometry and overscan behavior
- `tests/property_area_series_tests.rs`
  - area geometry count/baseline and finiteness invariants
- `tests/crosshair_tests.rs`
  - interaction-level crosshair snapping behavior
- `tests/decimal_time_tests.rs`
  - typed constructor conversions
- `tests/api_tuning_tests.rs`
  - public API contracts for fit/autoscale tuning

Required expectation:
- every new behavior must have at least one deterministic test
- non-trivial math/state changes should include property tests

## 7) Contributor Patterns

When adding a feature:

1. add/adjust core types and invariants
2. expose behavior via `api` methods
3. add deterministic tests
4. add property tests where invariants matter
5. update parity checklist entry evidence
6. update changelog if release-impacting

When adding dependencies:
- include a concrete usage in code (avoid speculative dependencies)
- document why dependency is needed
- ensure `clippy` + tests remain green

## 8) Release Workflow

Current release style is alpha pre-release tags:
- `v0.0.X-alpha.0`

Flow:
1. feature PR merged to `main`
2. release prep commit updates version + changelog
3. tag push triggers release dry-run workflow
4. publish GitHub pre-release

Keep release notes scoped to:
- user-visible behavior
- API changes
- parity progress
