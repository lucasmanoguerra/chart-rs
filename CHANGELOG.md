# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog,
and this project adheres to Semantic Versioning.

## [Unreleased]

### Added
- No changes yet.

## [0.0.15-alpha.0] - 2026-02-11

### Added
- Axis tick density/collision baseline (`R-002`) with deterministic spacing-aware label selection.
- Render-frame axis label counts now adapt to viewport size while preserving deterministic output.
- New axis-layout regression tests and benchmark coverage for narrow viewport behavior.

## [0.0.14-alpha.0] - 2026-02-11

### Added
- Render pipeline baseline (`R-001`) with deterministic line/text primitives in `RenderFrame`.
- Real cairo/pango/pangocairo backend execution path with external cairo-context support for GTK drawing callbacks.
- Public `ChartEngine::build_render_frame` and `ChartEngine::render_on_cairo_context` APIs.
- New render-focused integration/property tests and criterion benchmark coverage.

## [0.0.13-alpha.0] - 2026-02-11

### Added
- Wheel pan + kinetic pan interaction baseline (`C-014`) with deterministic step-based behavior.
- Public APIs for notch-normalized wheel pan and tunable kinetic pan integration.
- New integration/property tests for wheel-pan span invariants and kinetic-pan decay/stopping behavior.
- Criterion benchmarks for wheel-pan and kinetic-pan interaction step cost.

## [0.0.12-alpha.0] - 2026-02-11

### Added
- Wheel zoom controller baseline (`C-013`) with deterministic notch-normalized zoom factors.
- Public wheel zoom API anchored to pixel coordinates with strict input validation.
- New integration/property tests for wheel zoom direction, no-op semantics, and anchor stability.
- Criterion benchmark for wheel zoom interaction step cost.

## [0.0.11-alpha.0] - 2026-02-11

### Added
- Crosshair mode baseline (`C-012`) with explicit `Magnet` and `Normal` behavior in the public API.
- New integration/property tests validating deterministic snap-on/snap-off crosshair behavior.
- Criterion benchmarks comparing pointer-move cost for magnet vs normal crosshair modes.

## [0.0.10-alpha.0] - 2026-02-11

### Added
- OHLC bar series projection baseline (`C-011`) with deterministic stem/tick geometry over active scales.
- Visible-window and overscan OHLC bar projection helpers for candle data.
- New integration/property tests covering tick-width validation and OHLC ordering invariants.
- Criterion benchmark for OHLC bar projection throughput.

## [0.0.9-alpha.0] - 2026-02-11

### Added
- Histogram series projection baseline (`C-010`) with deterministic bar geometry anchored to a configurable baseline.
- Visible-window and overscan histogram projection helpers for point data.
- New integration/property tests covering histogram width and bar-axis invariants.
- Criterion benchmark for histogram projection throughput.

## [0.0.8-alpha.0] - 2026-02-11

### Added
- Baseline series projection baseline (`C-009`) with deterministic line + above/below fill geometry.
- Visible-window and overscan baseline projection helpers for point data.
- New integration/property tests covering baseline clamp invariants.
- Criterion benchmark for baseline projection throughput.

## [0.0.7-alpha.0] - 2026-02-11

### Added
- Area series projection baseline (`C-008`) with deterministic line/fill geometry in `core` + `api`.
- Visible-window and overscan area projection helpers for point data.
- New integration/property tests for area geometry invariants and visible-range behavior.
- Criterion benchmark for area projection throughput.

## [0.0.6-alpha.0] - 2026-02-11

### Added
- Advanced marker placement baseline (`E-001`) with deterministic lane collision handling and visible-window projection.
- Plugin hooks baseline (`E-002`) with bounded extension points and read-only engine context.
- Deterministic plugin event dispatch integrated across data updates, interaction, viewport updates, and rendering.
- New integration tests for plugin lifecycle/event behavior and benchmark coverage for multi-plugin dispatch overhead.

## [0.0.5-alpha.0] - 2026-02-11

### Added
- Serializable chart bootstrap config and deterministic engine snapshot APIs.
- Stable series metadata ordering for reproducible snapshot fixtures.
- Optional telemetry bootstrap module (`telemetry` feature) backed by `tracing-subscriber`.
- Optional parallel candle projection path (`parallel-projection` feature) backed by `rayon`.
- Extended in-code documentation across core scaling/candlestick/data modules.
- New snapshot regression tests and property-based tests for snapping/snapshot invariants.
- New criterion benchmarks for large candle projection and snapshot JSON serialization paths.

## [0.0.4-alpha.0] - 2026-02-11

### Added
- Crosshair baseline state (`visible`, cursor coordinates, snap coordinates).
- Nearest-point/candle crosshair snapping in `ChartEngine`.
- Decimal/time primitives using `rust-decimal` and `chrono` for early type-system integration.
- `smallvec`-based candidate selection in crosshair snapping hot path.
- New tests for crosshair behavior and decimal/time constructors.
- Parity checklist progress update for C-004 crosshair baseline.

## [0.0.3-alpha.0] - 2026-02-11

### Added
- Base OHLC candlestick model (`OhlcBar`) with input validation.
- Deterministic candlestick geometry projection from time/price scales.
- `ChartEngine` support for candle storage, price autoscale from candles, and candle projection.
- Unit/integration/property tests for candlestick invariants and geometry consistency.
- Parity checklist progress update for C-003 candlestick basics.

## [0.0.2-alpha.0] - 2026-02-11

### Added
- Base `TimeScale` implementation with deterministic domain/pixel mapping.
- Base `PriceScale` implementation with inverted Y-axis mapping.
- Price autoscaling baseline from chart data (`PriceScale::from_data`).
- `ChartEngine` support for time/price mapping and runtime price autoscale.
- Extended unit/integration/property tests for time and price scales.

## [0.0.1-alpha.0] - 2026-02-11

### Added
- Initial project governance and architecture baseline.
- Modular crate scaffold for core/render/interaction/api/platform_gtk/extensions.
- Test harness with unit, integration, and property-test examples.
- Benchmark harness with criterion.
- GitHub Actions workflows for CI, security, and scheduled benchmarks.
- Initial repository bootstrap.
