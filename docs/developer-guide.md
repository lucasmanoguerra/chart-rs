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
- `baseline_series.rs`
  - `BaselineVertex`
  - `BaselineGeometry`
  - `project_baseline_geometry`
- `histogram_series.rs`
  - `HistogramBar`
  - `project_histogram_bars`
- `bar_series.rs`
  - `BarGeometry`
  - `project_bars`
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
- `CrosshairMode::Magnet` snaps to nearest mapped data/candle candidate
- `CrosshairMode::Normal` follows pointer coordinates without snapping
- wheel delta is normalized to 120-step notches for deterministic zoom factors
- wheel pan preserves visible-range span while shifting window deterministically
- kinetic pan uses deterministic step integration with explicit decay tuning

### `src/api`
Main public facade (`ChartEngine`, `ChartEngineConfig`).

Responsibilities:
- orchestration between core + interaction + renderer
- time visible range controls and fit-to-data
- price autoscale from points/candles (default and tuned)
- crosshair snapping behavior
- time-axis formatter policy + locale/custom formatter injection
- price-axis formatter policy + display-mode + custom formatter injection
- zoom-aware adaptive time-axis formatting and label-cache metrics
- timezone/session-aware time-axis labeling for trading-hour style charts
- major time-tick visual emphasis for session/day boundaries
- render style contract for grid/axis parity tuning

### `src/render`
Renderer trait boundary and backend implementations.

- `RenderFrame`
- `LinePrimitive`
- `TextPrimitive`
- `Renderer`
- `NullRenderer`
- feature-gated cairo backend (`CairoRenderer`, `CairoContextRenderer`)

Render invariants:
- frame construction is deterministic for fixed engine state
- axis labels use spacing-aware collision filtering
- label density scales with viewport size within fixed min/max bounds
- time-axis labels support built-in policy+locale and explicit custom formatter injection
- time-axis UTC policies can align to fixed-offset local timezones and optional session windows
- session/day boundary ticks can render with dedicated major grid/label styling
- price-axis labels support fixed/adaptive precision, min-move rounding, and normal/percent/indexed display modes
- repeated redraws reuse deterministic time-label cache entries (`time_label_cache_stats`)
- render style controls grid/border/axis panel visuals without leaking backend logic into `api`

## 3) Data Flow

Typical runtime flow:

1. create engine with initial time/price domains
2. push/update points and/or candles
3. call fit/autoscale APIs to derive tuned domains
4. pointer/wheel events update crosshair and visible-range interaction state
5. `build_render_frame` materializes deterministic primitives (series + axes + labels)
6. renderer consumes immutable frame data

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
- `tests/baseline_series_tests.rs`
  - deterministic baseline line/split-fill geometry and visible-window behavior
- `tests/property_baseline_series_tests.rs`
  - baseline clamp invariants and finite-geometry properties
- `tests/histogram_series_tests.rs`
  - deterministic histogram geometry, width validation, and visible-window behavior
- `tests/property_histogram_series_tests.rs`
  - histogram axis/bar invariants and finite-geometry properties
- `tests/bar_series_tests.rs`
  - deterministic OHLC bar geometry, tick-width validation, and visible-window behavior
- `tests/property_bar_series_tests.rs`
  - OHLC bar ordering/visibility invariants and finite-geometry properties
- `tests/crosshair_tests.rs`
  - interaction-level crosshair snapping and mode-switch behavior
- `tests/interaction_wheel_zoom_tests.rs`
  - deterministic wheel-zoom direction, no-op, and anchor-stability behavior
- `tests/interaction_kinetic_pan_tests.rs`
  - deterministic wheel-pan and kinetic-pan stepping behavior
- `tests/render_frame_tests.rs`
  - deterministic render-frame construction and null-renderer command counts
- `tests/render_cairo_backend_tests.rs`
  - cairo backend command execution and external-context rendering behavior
- `tests/property_render_frame_tests.rs`
  - render-frame determinism and finite-geometry invariants
- `tests/render_axis_layout_tests.rs`
  - axis label density/collision behavior for narrow vs wide viewports
- `tests/time_axis_formatter_tests.rs`
  - time-axis policy/locale formatting, adaptive zoom behavior, and label-cache hit behavior
- `tests/price_axis_formatter_tests.rs`
  - price-axis formatting policies, display modes, and formatter override behavior
- `tests/render_style_tests.rs`
  - render-style validation and grid/axis visual contract behavior
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
