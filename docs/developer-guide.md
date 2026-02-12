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
  - `PriceScaleMode`
  - `PriceScaleTuning`
  - autoscale tuning + inverted Y mapping (linear/log mode aware)
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

Key files:
- `mod.rs` (engine orchestration, state transitions, frame assembly)
- `render_style.rs` (style enums + `RenderStyle` default contract)
- `axis_config.rs` (time/price axis formatter config and policy types)
- `axis_label_format.rs` (axis label formatting, display transforms, and quantization helpers)
- `label_cache.rs` (label cache keys/profiles, cache stores, and cache stats/types)
- `validation.rs` (render/axis config validation functions)
- `axis_ticks.rs` (axis tick density/spacing selection helpers)
- `data_window.rs` (visible-window expansion and marker window filtering helpers)
- `data_controller.rs` (public data-series mutation methods)
- `engine_accessors.rs` (public engine metadata/data/viewport accessor methods)
- `axis_label_controller.rs` (public time/price axis label config controller methods)
- `interaction_validation.rs` (kinetic-pan validation helpers)
- `price_resolver.rs` (latest/previous price sample and marker color/text resolution helpers)
- `layout_helpers.rs` (crosshair/axis label layout math helpers)
- `snap_resolver.rs` (crosshair nearest-sample snapping helpers for points/candles)
- `cache_profile.rs` (time/price label cache-profile resolution helpers)
- `plugin_dispatch.rs` (plugin event context/build + dispatch helpers)
- `plugin_registry.rs` (public plugin lifecycle/registry methods)
- `interaction_controller.rs` (public crosshair/pan/kinetic interaction controller methods)
- `scale_access.rs` (public time-scale mapping/range accessor methods)
- `time_scale_controller.rs` (public time-scale range/pan/zoom/fit controller methods)
- `series_projection.rs` (public series geometry/markers projection methods)
- `snapshot_controller.rs` (public snapshot serialization/state export methods)
- `render_frame_builder.rs` (render-frame assembly and axis/crosshair label formatting helpers)
- `label_formatter_controller.rs` (public axis/crosshair label formatter + label-cache lifecycle methods and cache stats/clear APIs)
- `visible_window_access.rs` (public visible-window point/candle accessor methods)
- `price_scale_access.rs` (public price-scale map/domain/mode/autoscale methods)

Responsibilities:
- orchestration between core + interaction + renderer
- time visible range controls and fit-to-data
- price autoscale from points/candles (default and tuned)
- crosshair snapping behavior
- price scale mode switching (`Linear` / `Log`) with domain validation
- time-axis formatter policy + locale/custom formatter injection
- price-axis formatter policy + display-mode + custom formatter injection
- zoom-aware adaptive time-axis formatting and label-cache metrics
- price-axis label-cache metrics for redraw hot paths (`price_label_cache_stats`)
- crosshair axis-label formatter override cache metrics for redraw hot paths (`crosshair_time_label_cache_stats`, `crosshair_price_label_cache_stats`)
- latest-price marker controls (line/label style + visibility toggles)
- latest-price label exclusion radius to avoid overlapping price-axis labels
- optional trend-aware last-price marker color policy (up/down/neutral)
- configurable last-price source mode (`LatestData` or `LatestVisible`) for pan/zoom behavior parity
- optional last-price axis label box (filled price-box) with configurable fill/text colors, border/radius, and contrast policy
- configurable last-price label-box width policy (`FullAxis` / `FitText`) with deterministic text-width estimation, horizontal padding, and minimum width guardrails
- configurable price-axis inset policy for right-side label padding and tick-mark extension length
- configurable price-axis tick-mark style policy (dedicated color/width separate from axis border)
- configurable price-axis label typography policy (font size and vertical offset from tick position)
- configurable last-price label vertical offset policy (independent from last-price font-size)
- configurable last-price label right inset policy (independent from regular price-axis label inset)
- configurable price-axis tick-mark visibility policy (show/hide short horizontal marks)
- configurable price-axis horizontal grid-line visibility policy (show/hide per-axis grid strokes)
- configurable price-axis regular-label visibility policy (show/hide non-marker price labels)
- configurable price-axis horizontal grid-line style policy (color/width independent from time-grid lines)
- configurable time-axis regular-label typography policy (font size, vertical offset, and short tick-mark length)
- configurable time-axis regular-label visibility policy (show/hide non-major time labels)
- configurable time-axis short tick-mark visibility policy (show/hide vertical axis marks)
- configurable time-axis short tick-mark style policy (dedicated color/width independent from axis border)
- configurable time-axis label color policy (dedicated label color independent from price-axis labels)
- configurable major time-axis label visibility policy (show/hide major labels independently from regular labels)
- configurable major time-axis grid visibility policy (show/hide major grid lines independently from regular grid lines)
- configurable major time-axis label color policy (dedicated major-label color independent from regular time-axis labels)
- configurable major time-axis label vertical offset policy (dedicated major-label Y offset independent from regular time-axis labels)
- configurable major time-axis tick-mark style policy (dedicated major tick-mark color/width/length independent from regular time-axis ticks)
- configurable major time-axis tick-mark visibility policy (show/hide major axis marks independently from regular time-axis ticks)
- configurable time-axis border visibility policy (show/hide bottom axis border independently from right price-axis border)
- configurable price-axis border visibility policy (show/hide right axis border independently from bottom time-axis border)
- configurable crosshair guide-line render policy (dedicated color/width and independent horizontal/vertical visibility toggles)
- configurable crosshair guide-line combined visibility gate policy (`show_crosshair_lines`) applied together with per-axis visibility toggles
- configurable crosshair axis-label render policy (dedicated time/price label colors, font size, and independent time/price visibility toggles)
- configurable crosshair axis-label formatter override policy per axis (independent time/price formatter overrides with fallback to axis formatter policies)
- configurable crosshair axis-label text-transform policy per axis (shared prefix/suffix fallback plus independent time/price prefix/suffix overrides)
- configurable crosshair axis-label numeric-precision policy per axis (shared precision fallback plus independent time/price precision overrides)
- configurable crosshair axis-label formatter context policy per axis (visible span + source mode for context-aware time/price formatter overrides)
- configurable crosshair axis-label formatter context cache-key policy per axis (context-aware formatter caches partition by source mode and visible span)
- configurable crosshair axis-label formatter context invalidation lifecycle (context-aware formatter caches clear on crosshair-mode and visible-range transitions)
- snapshot/export parity for crosshair formatter lifecycle state (override mode per axis and formatter generations)
- hardened crosshair formatter lifecycle introspection API (`crosshair_*_label_formatter_override_mode`, `crosshair_label_formatter_generations`) for host-side state diagnostics
- property-based lifecycle coverage for crosshair formatter transitions (legacy/context set/clear, context invalidation triggers, snapshot parity)
- configurable crosshair axis-label box policy (deterministic fit-text boxes with dedicated fill, padding, and independent time/price visibility toggles)
- configurable crosshair axis-label box border/radius policy (deterministic border width/color and corner-radius styling)
- configurable crosshair axis-label box text policy (manual text color or automatic contrast from box fill luminance)
- configurable crosshair axis-label box width-mode policy (`FitText`/`FullAxis`) with shared default and optional per-axis overrides
- configurable crosshair axis-label box border visibility policy (independent time/price border toggles)
- configurable crosshair axis-label vertical-offset policy (independent time/price Y offsets)
- configurable crosshair axis-label horizontal-inset policy (independent time/price X insets)
- configurable crosshair axis-label font-size policy (independent time/price font sizes)
- configurable crosshair axis-label box padding policy per axis (independent time/price X/Y padding)
- configurable crosshair axis-label box border-style policy per axis (independent time/price border color/width)
- configurable crosshair axis-label box corner-radius policy per axis (independent time/price corner radii)
- configurable crosshair axis-label box text policy per axis (independent time/price manual text color and auto-contrast toggles)
- configurable crosshair axis-label box fill-color policy per axis (independent time/price fill colors)
- configurable crosshair axis-label box min-width policy per axis (independent time/price minimum-width constraints)
- configurable crosshair axis-label box text-alignment policy per axis (independent time/price text alignment with shared fallback)
- configurable crosshair axis-label box vertical-anchor policy per axis (independent time/price vertical anchoring with shared fallback)
- configurable crosshair axis-label box horizontal-anchor policy per axis (independent time/price horizontal anchoring with shared fallback)
- configurable crosshair axis-label box overflow policy per axis (`ClipToAxis`/`AllowOverflow`) with shared fallback
- configurable crosshair axis-label box visibility-priority policy per axis (`KeepBoth`/`PreferTime`/`PreferPrice`) for overlap resolution
- configurable crosshair axis-label box clipping-margin policy per axis (independent time/price clip insets when using `ClipToAxis`)
- configurable crosshair axis-label box jitter-stabilization policy per axis (independent time/price position quantization step in px)
- configurable crosshair axis-label box z-order policy per axis (`PriceAboveTime`/`TimeAbovePrice`) with shared fallback
- configurable crosshair guide-line stroke-style policy per axis (`Solid`/`Dashed`/`Dotted`) with shared fallback
- configurable crosshair guide-line color policy per axis (independent horizontal/vertical colors with shared fallback to `crosshair_line_color`)
- configurable crosshair guide-line width policy per axis (independent horizontal/vertical widths with shared fallback to `crosshair_line_width`)
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
- price-axis ticks are generated in transformed scale space, then mapped back to raw prices (linear/log-safe)
- log-mode price-axis ticks prefer deterministic 1/2/5 decade ladders with endpoint-preserving downsampling
- repeated redraws reuse deterministic time-label cache entries (`time_label_cache_stats`)
- repeated redraws reuse deterministic price-label cache entries (`price_label_cache_stats`)
- render frame can include deterministic latest-price marker primitives from newest point/candle sample
- price-axis labels can skip deterministic overlap zones around the last-price label (`last_price_label_exclusion_px`)
- last-price marker can optionally use deterministic trend colors (`last_price_use_trend_color`) with up/down/neutral overrides
- last-price marker source can target full-series latest sample or newest visible-window sample (`last_price_source_mode`)
- render frame supports deterministic filled rectangles for axis price-box visuals (`RectPrimitive`)
- price-box labels support deterministic border width/color, corner radius, and optional auto-contrast text color resolution
- price-box width is deterministic: either full-axis panel width or fit-text width computed from estimator + horizontal padding and clamped to axis bounds
- price-axis label anchor and tick-mark extension are deterministic style knobs (`price_axis_label_padding_right_px`, `price_axis_tick_mark_length_px`)
- price-axis tick-mark stroke can be tuned independently from axis border styling (`price_axis_tick_mark_color`, `price_axis_tick_mark_width`)
- price-axis label font size/offset are deterministic style knobs (`price_axis_label_font_size_px`, `price_axis_label_offset_y_px`)
- last-price label Y anchor offset is a deterministic style knob (`last_price_label_offset_y_px`)
- last-price label right inset is a deterministic style knob (`last_price_label_padding_right_px`) for non-box mode
- price-axis short tick-mark visibility is a deterministic style knob (`show_price_axis_tick_marks`)
- price-axis horizontal grid visibility is a deterministic style knob (`show_price_axis_grid_lines`)
- price-axis regular-label visibility is a deterministic style knob (`show_price_axis_labels`)
- price-axis horizontal grid style is deterministic (`price_axis_grid_line_color`, `price_axis_grid_line_width`)
- time-axis regular-label font size/offset/tick length are deterministic style knobs (`time_axis_label_font_size_px`, `time_axis_label_offset_y_px`, `time_axis_tick_mark_length_px`)
- time-axis regular-label visibility is a deterministic style knob (`show_time_axis_labels`)
- time-axis short tick-mark visibility is a deterministic style knob (`show_time_axis_tick_marks`)
- time-axis short tick-mark style is deterministic (`time_axis_tick_mark_color`, `time_axis_tick_mark_width`)
- time-axis label color is a deterministic style knob (`time_axis_label_color`)
- major time-axis label visibility is a deterministic style knob (`show_major_time_labels`)
- major time-axis grid visibility is a deterministic style knob (`show_major_time_grid_lines`)
- major time-axis label color is a deterministic style knob (`major_time_label_color`)
- major time-axis label vertical offset is a deterministic style knob (`major_time_label_offset_y_px`)
- major time-axis tick-mark style is a deterministic style knob (`major_time_tick_mark_color`, `major_time_tick_mark_width`, `major_time_tick_mark_length_px`)
- major time-axis tick-mark visibility is a deterministic style knob (`show_major_time_tick_marks`)
- time-axis border visibility is a deterministic style knob (`show_time_axis_border`)
- price-axis border visibility is a deterministic style knob (`show_price_axis_border`)
- crosshair guide lines are deterministic style knobs (`crosshair_line_color`, `crosshair_line_width`, `show_crosshair_horizontal_line`, `show_crosshair_vertical_line`)
- crosshair axis labels are deterministic style knobs (`crosshair_time_label_color`, `crosshair_price_label_color`, `crosshair_axis_label_font_size_px`, `show_crosshair_time_label`, `show_crosshair_price_label`)
- crosshair axis-label text transform is deterministic per axis (`crosshair_label_prefix`, `crosshair_label_suffix`, `crosshair_time_label_prefix`, `crosshair_time_label_suffix`, `crosshair_price_label_prefix`, `crosshair_price_label_suffix`)
- crosshair axis-label numeric precision is deterministic per axis (`crosshair_label_numeric_precision`, `crosshair_time_label_numeric_precision`, `crosshair_price_label_numeric_precision`)
- crosshair axis-label boxes are deterministic style knobs (`crosshair_label_box_color`, `crosshair_label_box_padding_x_px`, `crosshair_label_box_padding_y_px`, `show_crosshair_time_label_box`, `show_crosshair_price_label_box`)
- crosshair axis-label boxes support deterministic border/radius style knobs (`crosshair_label_box_border_width_px`, `crosshair_label_box_border_color`, `crosshair_label_box_corner_radius_px`)
- crosshair axis-label box text color is deterministic with manual/auto-contrast policy (`crosshair_label_box_text_color`, `crosshair_label_box_auto_text_contrast`)
- crosshair axis-label boxes support deterministic width mode selection with shared default and per-axis overrides (`crosshair_label_box_width_mode`, `crosshair_time_label_box_width_mode`, `crosshair_price_label_box_width_mode`)
- crosshair axis-label box border visibility is deterministic per axis (`show_crosshair_time_label_box_border`, `show_crosshair_price_label_box_border`)
- crosshair axis-label vertical offsets are deterministic per axis (`crosshair_time_label_offset_y_px`, `crosshair_price_label_offset_y_px`)
- crosshair axis-label horizontal insets are deterministic per axis (`crosshair_time_label_padding_x_px`, `crosshair_price_label_padding_right_px`)
- crosshair axis-label font sizes are deterministic per axis (`crosshair_time_label_font_size_px`, `crosshair_price_label_font_size_px`)
- crosshair axis-label box paddings are deterministic per axis (`crosshair_time_label_box_padding_x_px`, `crosshair_time_label_box_padding_y_px`, `crosshair_price_label_box_padding_x_px`, `crosshair_price_label_box_padding_y_px`)
- crosshair axis-label box border styles are deterministic per axis (`crosshair_time_label_box_border_color`, `crosshair_time_label_box_border_width_px`, `crosshair_price_label_box_border_color`, `crosshair_price_label_box_border_width_px`)
- crosshair axis-label box corner radii are deterministic per axis (`crosshair_time_label_box_corner_radius_px`, `crosshair_price_label_box_corner_radius_px`)
- crosshair axis-label box text policy is deterministic per axis (`crosshair_time_label_box_text_color`, `crosshair_price_label_box_text_color`, `crosshair_time_label_box_auto_text_contrast`, `crosshair_price_label_box_auto_text_contrast`)
- crosshair axis-label box fill colors are deterministic per axis (`crosshair_time_label_box_color`, `crosshair_price_label_box_color`)
- crosshair axis-label box min-widths are deterministic per axis (`crosshair_label_box_min_width_px`, `crosshair_time_label_box_min_width_px`, `crosshair_price_label_box_min_width_px`)
- crosshair axis-label box text alignment is deterministic per axis (`crosshair_label_box_text_h_align`, `crosshair_time_label_box_text_h_align`, `crosshair_price_label_box_text_h_align`)
- crosshair axis-label box vertical anchor is deterministic per axis (`crosshair_label_box_vertical_anchor`, `crosshair_time_label_box_vertical_anchor`, `crosshair_price_label_box_vertical_anchor`)
- crosshair axis-label box horizontal anchor is deterministic per axis (`crosshair_label_box_horizontal_anchor`, `crosshair_time_label_box_horizontal_anchor`, `crosshair_price_label_box_horizontal_anchor`)
- crosshair axis-label box overflow policy is deterministic per axis (`crosshair_label_box_overflow_policy`, `crosshair_time_label_box_overflow_policy`, `crosshair_price_label_box_overflow_policy`)
- crosshair axis-label box visibility priority is deterministic per axis (`crosshair_label_box_visibility_priority`, `crosshair_time_label_box_visibility_priority`, `crosshair_price_label_box_visibility_priority`)
- crosshair axis-label box clipping margin is deterministic per axis (`crosshair_label_box_clip_margin_px`, `crosshair_time_label_box_clip_margin_px`, `crosshair_price_label_box_clip_margin_px`)
- crosshair axis-label box stabilization step is deterministic per axis (`crosshair_label_box_stabilization_step_px`, `crosshair_time_label_box_stabilization_step_px`, `crosshair_price_label_box_stabilization_step_px`)
- crosshair axis-label box z-order is deterministic per axis (`crosshair_label_box_z_order_policy`, `crosshair_time_label_box_z_order_policy`, `crosshair_price_label_box_z_order_policy`)
- crosshair guide-line stroke style is deterministic per axis (`crosshair_line_style`, `crosshair_horizontal_line_style`, `crosshair_vertical_line_style`)
- crosshair guide-line color is deterministic per axis (`crosshair_line_color`, `crosshair_horizontal_line_color`, `crosshair_vertical_line_color`)
- crosshair guide-line width is deterministic per axis (`crosshair_line_width`, `crosshair_horizontal_line_width`, `crosshair_vertical_line_width`)
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
- `Linear` mode pads directly in raw-price space
- `Log` mode validates strictly-positive prices and applies span/padding in log space
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
- `tests/price_scale_mode_tests.rs`
  - linear/log mode switching behavior and log-autoscale invariants
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
  - deterministic render-frame construction, latest-price marker behavior, and null-renderer command counts
- `tests/render_cairo_backend_tests.rs`
  - cairo backend command execution and external-context rendering behavior
- `tests/property_render_frame_tests.rs`
  - render-frame determinism and finite-geometry invariants
- `tests/render_axis_layout_tests.rs`
  - axis label density/collision behavior for narrow vs wide viewports
- `tests/time_axis_formatter_tests.rs`
  - time-axis policy/locale formatting, adaptive zoom behavior, and label-cache hit behavior
- `tests/price_axis_formatter_tests.rs`
  - price-axis formatting policies, display modes, formatter override behavior, and cache-hit validation
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
