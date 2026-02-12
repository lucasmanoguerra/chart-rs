# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog,
and this project adheres to Semantic Versioning.

## [Unreleased]

### Added
- Price-axis label typography parity baseline (`R-022`) with deterministic configurable axis-label font size and vertical offset.
- New render-style knobs: `price_axis_label_font_size_px` and `price_axis_label_offset_y_px`.
- New render-frame/style tests and criterion benchmark coverage for price-axis label typography behavior.
- Last-price label offset parity baseline (`R-023`) with deterministic configurable vertical offset from marker Y.
- New render-style knob: `last_price_label_offset_y_px`.
- New render-frame/style tests and criterion benchmark coverage for last-price label offset behavior.
- Last-price label right-inset parity baseline (`R-024`) with deterministic configurable right-side padding in non-box mode.
- New render-style knob: `last_price_label_padding_right_px`.
- New render-frame/style tests and criterion benchmark coverage for last-price label right-inset behavior.
- Price-axis tick-mark visibility parity baseline (`R-025`) with deterministic show/hide behavior for short axis marks.
- New render-style knob: `show_price_axis_tick_marks`.
- New render-frame/style tests and criterion benchmark coverage for tick-mark visibility behavior.
- Price-axis horizontal grid-line visibility parity baseline (`R-026`) with deterministic show/hide behavior.
- New render-style knob: `show_price_axis_grid_lines`.
- New render-frame/style tests and criterion benchmark coverage for horizontal grid-line visibility behavior.
- Price-axis regular-label visibility parity baseline (`R-027`) with deterministic show/hide behavior.
- New render-style knob: `show_price_axis_labels`.
- New render-frame/style tests and criterion benchmark coverage for regular-label visibility behavior.
- Price-axis horizontal grid-line style parity baseline (`R-028`) with deterministic dedicated color/width controls.
- New render-style knobs: `price_axis_grid_line_color` and `price_axis_grid_line_width`.
- New render-frame/style tests and criterion benchmark coverage for horizontal grid-line style behavior.
- Time-axis regular-label typography parity baseline (`R-029`) with deterministic configurable label font size, vertical offset, and short tick-mark length.
- New render-style knobs: `time_axis_label_font_size_px`, `time_axis_label_offset_y_px`, and `time_axis_tick_mark_length_px`.
- New render-frame/style tests and criterion benchmark coverage for time-axis regular-label typography behavior.
- Time-axis regular-label visibility parity baseline (`R-030`) with deterministic show/hide behavior.
- New render-style knob: `show_time_axis_labels`.
- New render-frame tests and criterion benchmark coverage for time-axis label visibility behavior.
- Time-axis tick-mark visibility parity baseline (`R-031`) with deterministic show/hide behavior.
- New render-style knob: `show_time_axis_tick_marks`.
- New render-frame/style tests and criterion benchmark coverage for time-axis tick-mark visibility behavior.
- Time-axis tick-mark style parity baseline (`R-032`) with deterministic dedicated color/width controls.
- New render-style knobs: `time_axis_tick_mark_color` and `time_axis_tick_mark_width`.
- New render-frame/style tests and criterion benchmark coverage for time-axis tick-mark style behavior.
- Time-axis label color parity baseline (`R-033`) with deterministic dedicated color control independent from price-axis labels.
- New render-style knob: `time_axis_label_color`.
- New render-frame/style tests and criterion benchmark coverage for time-axis label color behavior.
- Major time-axis label visibility parity baseline (`R-034`) with deterministic show/hide behavior independent from regular time labels.
- New render-style knob: `show_major_time_labels`.
- New render-frame/style tests and criterion benchmark coverage for major-label visibility behavior.
- Major time-axis grid visibility parity baseline (`R-035`) with deterministic show/hide behavior independent from regular time-grid lines.
- New render-style knob: `show_major_time_grid_lines`.
- New render-frame/style tests and criterion benchmark coverage for major-grid visibility behavior.
- Major time-axis label color parity baseline (`R-036`) with deterministic dedicated color control independent from regular time-axis labels.
- New render-style knob: `major_time_label_color`.
- New render-frame/style tests and criterion benchmark coverage for major-label color behavior.

## [0.0.33-alpha.0] - 2026-02-12

### Added
- Price-axis tick-mark style parity baseline (`R-021`) with deterministic dedicated color/width controls.
- New render-style knobs: `price_axis_tick_mark_color` and `price_axis_tick_mark_width`.
- New render-frame/style tests and criterion benchmark coverage for dedicated tick-mark styling behavior.

## [0.0.32-alpha.0] - 2026-02-12

### Added
- Price-axis inset parity baseline (`R-020`) with deterministic right-side label anchor and axis tick-mark extension controls.
- New render-style knobs: `price_axis_label_padding_right_px` and `price_axis_tick_mark_length_px`.
- New render-frame/style tests and criterion benchmark coverage for price-axis inset policy behavior.

## [0.0.31-alpha.0] - 2026-02-11

### Added
- Last-price label-box width parity baseline (`R-019`) with deterministic full-axis and fit-text width modes.
- New render-style knobs: `last_price_label_box_width_mode`, `last_price_label_box_padding_x_px`, and `last_price_label_box_min_width_px`.
- New render-frame/style tests and criterion benchmark coverage for fit-text width behavior.

## [0.0.30-alpha.0] - 2026-02-11

### Added
- Last-price label-box style extension baseline (`R-018`) with deterministic border, corner-radius, and auto-contrast text policy.
- New render-style knobs: `last_price_label_box_border_width_px`, `last_price_label_box_border_color`, `last_price_label_box_corner_radius_px`, and `last_price_label_box_auto_text_contrast`.
- New render/frame/style/backend tests and benchmark coverage for rounded/bordered label-box rendering.

## [0.0.29-alpha.0] - 2026-02-11

### Added
- Last-price label-box parity baseline (`R-017`) with deterministic filled axis-panel box behind latest-price label text.
- New render-style knobs for label-box behavior: `show_last_price_label_box`, `last_price_label_box_use_marker_color`, `last_price_label_box_color`, `last_price_label_box_text_color`, and `last_price_label_box_padding_y_px`.
- New render-frame/style/backend tests and benchmark coverage for label-box rendering.

## [0.0.28-alpha.0] - 2026-02-11

### Added
- Last-price source mode baseline (`R-016`) with deterministic selection between latest full-series sample and latest visible-range sample.
- New render-style knob `last_price_source_mode` with `LastPriceSourceMode::{LatestData, LatestVisible}`.
- New render-frame tests and benchmark coverage for visible-range marker-source behavior under pan/zoom.

## [0.0.27-alpha.0] - 2026-02-11

### Added
- Last-price trend color policy baseline (`R-015`) with deterministic up/down/neutral marker coloring from latest-vs-previous sample comparison.
- New render-style knobs: `last_price_use_trend_color`, `last_price_up_color`, `last_price_down_color`, and `last_price_neutral_color`.
- New render-frame/style tests and benchmark coverage for trend-driven marker coloring.

## [0.0.26-alpha.0] - 2026-02-11

### Added
- Last-price label collision filter baseline (`R-014`) with deterministic exclusion radius around the latest-price marker.
- New render-style knob `last_price_label_exclusion_px` to tune overlap filtering behavior.
- New render-style/frame tests and benchmark coverage for collision-filtered axis labels.

## [0.0.25-alpha.0] - 2026-02-11

### Added
- Last-price marker parity baseline (`R-013`) with deterministic line/label rendering from the newest point/candle sample.
- New render-style knobs for last-price marker color/width/font-size and visibility toggles.
- New render/property tests and benchmark coverage for latest-price marker behavior.

## [0.0.24-alpha.0] - 2026-02-11

### Added
- Price-axis label cache baseline (`R-012`) with deterministic cache keys for built-in/custom formatter paths.
- New `ChartEngine` cache stats/clear APIs for price labels (`price_label_cache_stats`, `clear_price_label_cache`).
- New tests and benchmark coverage for repeated redraw cache-hit behavior.

## [0.0.23-alpha.0] - 2026-02-11

### Added
- Price-axis log ladder parity baseline (`R-011`) with deterministic 1/2/5 decade ticks in log mode.
- Log tick downsampling now preserves endpoints and domain direction for stable axis labeling.
- New log-ladder regression tests and benchmark coverage for tick generation/render paths.

## [0.0.22-alpha.0] - 2026-02-11

### Added
- Price scale mode parity baseline (`R-010`) with `Linear`/`Log` mapping and runtime mode switching API in `ChartEngine`.
- Log-mode autoscale now applies tuning in transformed domain to keep deterministic positive domains.
- New log-mode regression tests and benchmark coverage for price-axis render frame generation.

## [0.0.21-alpha.0] - 2026-02-11

### Added
- Price-axis display mode parity baseline (`R-009`) with deterministic `Normal`, `Percentage`, and `IndexedTo100` label transforms.
- Configurable/derived display base-price support with explicit validation for percentage/indexed modes.
- Extended price-axis formatter tests and benchmark coverage for display-transform paths.

## [0.0.20-alpha.0] - 2026-02-11

### Added
- Price-axis formatter parity baseline (`R-008`) with fixed-decimal, adaptive, and min-move policies.
- Deterministic min-move rounding with optional trailing-zero trimming and locale-aware rendering.
- New price-axis formatter tests and benchmark coverage for min-move formatting path.

## [0.0.19-alpha.0] - 2026-02-11

### Added
- Time-axis major tick parity baseline (`R-007`) with deterministic boundary classification for session start/end and local-midnight transitions.
- New render-style knobs for major ticks (`major_grid_line_color`, `major_grid_line_width`, `major_time_label_font_size_px`).
- New render-style regression test coverage and benchmark for major-tick styled frame generation.

## [0.0.18-alpha.0] - 2026-02-11

### Added
- Time-axis session/timezone parity baseline (`R-006`) with fixed-offset timezone alignment for UTC-based label policies.
- Optional trading-session envelope that preserves explicit session-boundary labels while collapsing in-session intraday labels to time-only output.
- Additional formatter validation coverage for invalid session/timezone inputs.
- New benchmark coverage for session+timezone formatter throughput.

## [0.0.17-alpha.0] - 2026-02-11

### Added
- Time-axis zoom-aware formatter baseline (`R-005`) with `UtcAdaptive` policy selection by visible span.
- In-engine time-axis label cache with hit/miss stats for redraw optimization.
- New tests for adaptive formatter behavior and cache-hit verification.
- New benchmark coverage for hot-path cached time-axis labeling.

## [0.0.16-alpha.0] - 2026-02-11

### Added
- Time-axis formatter baseline (`R-003`) with locale presets and custom formatter injection.
- Built-in time label policies for logical decimals and UTC datetime formatting.
- Price-scale visual styling baseline (`R-004`) with configurable grid/axis style contract.
- Plot/axis panel split and deterministic grid rendering closer to Lightweight Charts conventions.
- New formatter/style regression tests and narrow-axis benchmark coverage.

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
