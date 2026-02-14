# Developer Guide

This guide is the practical, code-oriented reference for contributors.

For governance and quality rules, read:
- `AGENTS.md`
- `CONTRIBUTING.md`

For architecture and parity status, read:
- `docs/architecture.md`
- `docs/parity-v5.1-checklist.md`
- `docs/gtk-relm4-crosshair-formatters.md`
- `docs/axis-section-visual-fixtures.md`

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
  - `TimeIndexCoordinateSpace`
  - `TimeScaleTuning`
  - visible/full range logic
  - Lightweight-style index/coordinate transforms (`index_to_coordinate`, `coordinate_to_logical_index`, ceil index conversion, right-offset pan/zoom helpers)
- `price_scale.rs`
  - `PriceScale`
  - `PriceCoordinateSpace`
  - `PriceScaleMode`
  - `PriceScaleTuning`
  - autoscale tuning + inverted Y mapping (`Linear` / `Log` / `Percentage` / `IndexedTo100`)
  - transformed-domain coordinate primitives (`transformed_to_pixel`, `pixel_to_transformed`) with explicit margin/internal-height semantics
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
- `CrosshairMode::Magnet` snaps to nearest mapped data/candle candidate, including sparse-index nearest-slot hints with correctness fallback for irregular windows
- `CrosshairMode::Normal` follows pointer coordinates without snapping
- `CrosshairMode::Hidden` keeps crosshair invisible regardless of pointer movement
- wheel delta is normalized to 120-step notches for deterministic zoom factors
- wheel pan preserves visible-range span while shifting window deterministically
- kinetic pan uses deterministic step integration with explicit decay tuning
- optional fixed-edge time-scale policy can constrain navigation to full-range bounds (`fix_left_edge` / `fix_right_edge`)
- optional interaction input gates can disable scroll/scale families and granular input paths (`handle_scroll`, `handle_scale`, `scroll_mouse_wheel`, `scroll_pressed_mouse_move`, `scroll_horz_touch_drag`, `scroll_vert_touch_drag`, `scale_mouse_wheel`, `scale_pinch`, `scale_axis_pressed_mouse_move`, `scale_axis_double_click_reset`)
- axis-scale interaction paths are available via `axis_drag_scale_price`, `axis_drag_scale_time`, `axis_double_click_reset_price_scale`, and `axis_double_click_reset_time_scale`, all gated by `scale_axis_*` options
- price-axis vertical pan is available through `axis_drag_pan_price`, preserves anchor projection/span, and is gated by `scale_axis_pressed_mouse_move`
- touch-style pan input is available through `touch_drag_pan_time_visible`, validates only enabled touch axes, and uses dominant-axis normalization (`width` for horizontal, `height` for vertical)
- optional time-scale navigation policy can synthesize right-margin/spacing behavior from data step estimation (`right_offset_bars` / `bar_spacing_px`)
- optional time-scale resize policy can lock visible range under viewport width changes using deterministic anchors (`lock_visible_range_on_resize` + `Left`/`Center`/`Right`)
- optional realtime append policy can preserve right-edge follow semantics with bar-based tolerance (`preserve_right_edge_on_append` / `right_edge_tolerance_bars`)

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
- `json_contract.rs` (versioned snapshot/diagnostics JSON contracts and backward-compatible parsers)
- `render_frame_builder.rs` (render-frame assembly and axis/crosshair label formatting helpers)
- `label_formatter_controller.rs` (public axis/crosshair label formatter + label-cache lifecycle methods and cache stats/clear APIs)
- `visible_window_access.rs` (public visible-window point/candle accessor methods)
- `price_scale_access.rs` (public price-scale map/domain/mode/autoscale methods)

Responsibilities:
- orchestration between core + interaction + renderer
- time visible range controls and fit-to-data
- bootstrap crosshair startup mode selection (`ChartEngineConfig::with_crosshair_mode`)
- bootstrap price-scale startup mode/inversion/margins (`ChartEngineConfig::with_price_scale_*`)
- bootstrap interaction input gates (`ChartEngineConfig::with_interaction_input_behavior`)
- bootstrap time-scale navigation/right-offset options (`ChartEngineConfig::with_time_scale_*`)
- bootstrap time-scale zoom options (`ChartEngineConfig::with_time_scale_scroll_zoom_behavior`, `with_time_scale_zoom_limit_behavior`)
- bootstrap time-scale edge/resize/realtime-append options (`ChartEngineConfig::with_time_scale_edge_behavior`, `with_time_scale_resize_behavior`, `with_time_scale_realtime_append_behavior`)
- bootstrap price-scale realtime autoscale policy (`ChartEngineConfig::with_price_scale_realtime_behavior`)
- bootstrap axis-label formatter policies (`ChartEngineConfig::with_time_axis_label_config`, `with_price_axis_label_config`)
- bootstrap last-price source mode policy (`ChartEngineConfig::with_last_price_source_mode`)
- bootstrap last-price behavior policy (`ChartEngineConfig::with_last_price_behavior`)
- bootstrap crosshair guide-line visibility policy (`ChartEngineConfig::with_crosshair_guide_line_behavior`)
- bootstrap crosshair guide-line stroke-style policy (`ChartEngineConfig::with_crosshair_guide_line_style_behavior`)
- bootstrap crosshair axis-label visibility policy (`ChartEngineConfig::with_crosshair_axis_label_visibility_behavior`)
- bootstrap crosshair axis-label style policy (`ChartEngineConfig::with_crosshair_axis_label_style_behavior`)
- bootstrap crosshair axis-label box style policy (`ChartEngineConfig::with_crosshair_axis_label_box_style_behavior`)
- optional time-scale edge constraints for visible-range navigation (`TimeScaleEdgeBehavior`)
- optional interaction input gating for host-controlled scroll/scale path enablement, including per-input wheel/drag/pinch and axis-scale option controls (`InteractionInputBehavior`)
- optional axis drag-scale and axis-reset interaction paths for both price and time (`axis_drag_scale_price`, `axis_drag_scale_time`, `axis_double_click_reset_price_scale`, `axis_double_click_reset_time_scale`)
- optional price-axis drag-pan interaction path (`axis_drag_pan_price`) with finite-input validation, anchor-preserving domain translation, and gate-aware no-op behavior
- optional time-scale navigation behavior for right-offset and spacing synthesis (`TimeScaleNavigationBehavior`)
- optional time-scale zoom-limit behavior for bar-spacing bounds (`TimeScaleZoomLimitBehavior`)
- optional pixel-based right-margin override with priority over bar-based right offset (`time_scale_right_offset_px`), with constrained zoom-limit/resize hardening that preserves margin semantics under extreme spans
- optional scroll-zoom right-edge anchoring policy (`right_bar_stays_on_scroll`)
- optional time-scale resize behavior for viewport resize anchoring policy (`TimeScaleResizeBehavior`)
- optional realtime append behavior for continuous tail tracking during incremental updates (`TimeScaleRealtimeAppendBehavior`)
- optional price-scale realtime autoscale behavior on incremental updates (`PriceScaleRealtimeBehavior`)
- explicit scroll-to-realtime command for deterministic tail reattachment (`scroll_time_to_realtime`)
- bar-based scroll position introspection and explicit positioning (`time_scroll_position_bars`, `scroll_time_to_position_bars`)
- pixel-to-logical-index mapping policy with sparse-slot control (`map_pixel_to_logical_index` + `TimeCoordinateIndexPolicy::{AllowWhitespace, IgnoreWhitespace}`)
- realtime update semantics for incremental feeds (`update_point` / `update_candle` append-or-replace with out-of-order rejection)
- deterministic canonicalization for full-replacement datasets (`set_data` / `set_candles`) with invalid-sample filtering, time sorting, and duplicate-timestamp replacement
- property-test coverage for canonicalization invariants under extreme/invalid input (`tests/property_data_set_canonicalization_tests.rs`)
- price autoscale from points/candles (default and tuned)
- optional autoscale refresh on realtime append/update flows (`autoscale_on_data_update`)
- optional autoscale refresh on full data replacement (`autoscale_on_data_set`)
- optional inverted price-axis mapping (`set_price_scale_inverted`)
- optional price-scale top/bottom whitespace margins (`set_price_scale_margin_behavior`)
- optional transformed-base policy for percentage/indexed modes (`set_price_scale_transformed_base_behavior`) with explicit or dynamic sources, deterministic cross-series precedence, candle tie-break on equal timestamps, and visible-empty fallback to full-data candidates
- crosshair snapping behavior
- price scale mode switching (`Linear` / `Log` / `Percentage` / `IndexedTo100`) with mode-safe domain/base validation
- time-scale logical-index utilities for hosts (`map_pixel_to_logical_index`, `map_pixel_to_logical_index_ceil`, `map_logical_index_to_pixel`, `nearest_filled_logical_slot_at_pixel`, `next_filled_logical_index`, `prev_filled_logical_index`)
- GTK adapter bridge for logical-index host tooling (`GtkChartAdapter::{map_*, nearest_filled_logical_slot_at_pixel, next_filled_logical_index, prev_filled_logical_index}`)
- fixture-driven differential parity harness for time-scale zoom/pan/`rightOffsetPixels` traces (`tests/lightweight_time_scale_differential_trace_tests.rs`, `tests/fixtures/lightweight_differential/`)
- fixture-driven differential parity harness for price-scale transformed/autoscale traces (`tests/lightweight_price_scale_differential_trace_tests.rs`, `tests/fixtures/lightweight_differential/price_scale_transformed_autoscale_trace.json`)
- fixture-driven differential parity harness for advanced interaction traces (wheel/pinch zoom, kinetic pan, crosshair mode/snap transitions) (`tests/lightweight_interaction_differential_trace_tests.rs`, `tests/fixtures/lightweight_differential/interaction_zoom_pan_kinetic_crosshair_trace.json`)
- interaction corpus includes advanced touch scenarios with multi-step pinch zoom, kinetic decay envelope assertions, and sparse-gap magnet snap checks (`touch-pinch-kinetic-gap-snap-advanced`)
- direct raw Lightweight interaction-capture import coverage without manual normalization (`tests/lightweight_raw_capture_import_tests.rs`, `tests/fixtures/lightweight_differential/lightweight_real_capture_interaction.raw.json`)
- direct raw Lightweight visual-capture import coverage without manual normalization (`tests/lightweight_visual_raw_capture_import_tests.rs`, `tests/fixtures/lightweight_differential/lightweight_real_capture_visual.raw.json`)
- trace import/export tooling for Lightweight capture interoperability (`cargo run --bin differential_trace_tool -- <export-time|import-time|export-price|import-price|export-interaction|import-interaction|import-lwc-interaction|import-lwc-visual> ...`)
- time-axis formatter policy + locale/custom formatter injection
- price-axis formatter policy + display-mode + custom formatter injection
- zoom-aware adaptive time-axis formatting and label-cache metrics
- price-axis label-cache metrics for redraw hot paths (`price_label_cache_stats`)
- crosshair axis-label formatter override cache metrics for redraw hot paths (`crosshair_time_label_cache_stats`, `crosshair_price_label_cache_stats`)
- latest-price marker controls (line/label style + visibility toggles)
- latest-price label exclusion radius to avoid overlapping price-axis labels
- optional trend-aware last-price marker color policy (up/down/neutral)
- configurable last-price source mode (`LatestData` or `LatestVisible`) for pan/zoom behavior parity
- dedicated last-price behavior API (`last_price_behavior` / `set_last_price_behavior`) for line visibility, label visibility, trend-color mode, and source policy control
- dedicated crosshair guide-line visibility API (`crosshair_guide_line_behavior` / `set_crosshair_guide_line_behavior`) for shared and per-axis line toggles
- dedicated crosshair guide-line style API (`crosshair_guide_line_style_behavior` / `set_crosshair_guide_line_style_behavior`) for shared/per-axis color-width-style policies
- dedicated crosshair axis-label visibility API (`crosshair_axis_label_visibility_behavior` / `set_crosshair_axis_label_visibility_behavior`) for time/price label, box, and border toggles
- dedicated crosshair axis-label style API (`crosshair_axis_label_style_behavior` / `set_crosshair_axis_label_style_behavior`) for time/price label color, font, offset, and inset policies
- dedicated crosshair axis-label box style API (`crosshair_axis_label_box_style_behavior` / `set_crosshair_axis_label_box_style_behavior`) for shared/per-axis box fill, border, and corner-radius policies
- optional last-price axis label box (filled price-box) with configurable fill/text colors, border/radius, and contrast policy
- configurable last-price label-box width policy (`FullAxis` / `FitText`) with deterministic text-width estimation, horizontal padding, and minimum width guardrails
- configurable price-axis inset policy for right-side label padding and tick-mark extension length
- adaptive axis-section sizing pass computes deterministic minimum panel dimensions from label/tick pressure and only expands configured axis sections when required
- axis-section visual-fixture corpus validates deterministic layout signatures (plot/time/price section geometry + label counts) from JSON manifests to catch adaptive-sizing drift
- fixture manifests can also export PNG references for manual visual review via `cargo run --features cairo-backend --bin generate_axis_section_fixture_pngs`
- corpus includes extreme/sparse stress scenarios (tiny viewport clamps, narrow-domain high-precision labels, sparse wide-range data)
- corpus includes mixed price display-mode stress (`Normal`, `Percentage`, `IndexedTo100`) under extreme value magnitudes
- corpus includes display-mode fallback edge-case stress for explicit invalid bases (`base_price=0`, `NaN`, `+inf`, `-inf`) in `Percentage`/`IndexedTo100`; fixture JSON uses `price_axis_display_base_override` tokens for non-finite literals
- manual visual baseline capture for GTK-host UI sanity checks is tracked in `reference_UI/Captura desde 2026-02-12 20-28-20.png` with review workflow documented in `docs/axis-section-visual-fixtures.md`
- manual visual review workflow is standardized by change type (`layout`, `render`, `formatter`) in `docs/axis-section-visual-fixtures.md` to reduce PR ambiguity
- criterion coverage includes display-mode fallback formatter/cache cost tracking with mixed fallback routes (`price_axis_display_mode_fallback_cache_hot_mixed`, `price_axis_display_mode_fallback_cache_cold_mixed`)
- criterion coverage includes transformed-base dynamic refresh and sparse logical-index host lookup hot paths (`price_scale_transformed_base_dynamic_refresh_visible_window`, `time_scale_sparse_nearest_filled_slot_lookup`, `time_scale_sparse_next_prev_filled_lookup`)
- CI guardrails validate fallback benchmark drift with ratio/latency budgets in scheduled and PR workflows (`scripts/check_fallback_bench_regressions.py`, `.github/workflows/bench.yml`, `.github/workflows/ci.yml`)
- CI guardrails validate transformed-base/sparse-index benchmark drift with ratio/latency budgets in scheduled and PR workflows (`scripts/check_new_parity_bench_regressions.py`, `.github/workflows/bench.yml`, `.github/workflows/ci.yml`)
- visual differential PNG harness compares rendered Cairo output against committed Lightweight-style baselines with explicit max/mean channel-diff budgets (`tests/lightweight_visual_differential_tests.rs`, `tests/fixtures/lightweight_visual_differential/visual_baseline_corpus.json`)
- visual harness can emit per-fixture `actual`/`baseline`/`diff` PNG artifacts plus `summary.json` when `LIGHTWEIGHT_VISUAL_DIFF_ARTIFACT_DIR` is set (used by CI artifact upload workflow)
- when intended render/style parity updates shift the visual output, refresh committed baselines via `cargo test --all-features -j 1 --test lightweight_visual_differential_tests -- --ignored --exact regenerate_lightweight_visual_baselines --test-threads=1` and re-run `cargo test-visual` to keep `max/mean=0` fixtures deterministic
- visual corpus includes candlestick/log-axis drag-scale and session/timezone time-axis drag-scale fixtures to keep time/price interaction parity under image-diff guardrails (`tests/fixtures/lightweight_visual_differential/reference_png/lwc-style-candles-log-axis-scale-price.png`, `tests/fixtures/lightweight_visual_differential/reference_png/lwc-style-timescale-session-timezone-axis-scale.png`)
- property/fuzz hardening includes extreme scale round-trip invariants for transformed price modes and constrained time-scale paths (`tests/property_extreme_scale_roundtrip_tests.rs`)
- dedicated CI parity guard runs visual PNG diffs plus elevated property stress profile (`parity_visual_property_guard` job in `.github/workflows/ci.yml`)
- criterion coverage includes zoom-adaptive axis-density render cost tracking for zoom-out/zoom-in paths (`axis_density_zoom_adaptive_out_render`, `axis_density_zoom_adaptive_in_render`)
- CI guardrails validate axis-density benchmark drift with zoom-in/out ratio and per-path latency budgets in scheduled and PR workflows (`scripts/check_axis_density_bench_regressions.py`, `.github/workflows/bench.yml`, `.github/workflows/ci.yml`)
- zoom-adaptive axis cadence uses an explicit non-linear zoom-ratio curve with neutral-band stabilization (`density_scale_from_zoom_ratio`) instead of direct square-root scaling, keeping intermediate zoom levels responsive while preserving min-spacing safeguards
- time-label collision filtering uses prioritized major/minor selection so major labels are retained when spacing collisions occur (`select_positions_with_min_spacing_prioritized`)
- directed Lightweight v5.1 cadence regression coverage includes intermediate time-axis zoom windows and multi-step price-axis scale-zoom windows (`tests/lightweight_axis_tick_cadence_reference_tests.rs`)
- fixture schema supports deterministic price-axis cadence stress setup through `disable_autoscale_on_data_set` and optional `price_axis_scale_steps` (`axis_drag_scale_price` replay)
- corpus includes dedicated price-axis zoom-extreme cadence fixtures (`zoom-extreme-axis-density-price-out`, `zoom-extreme-axis-density-price-in`)
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
- consolidated crosshair formatter diagnostics API (`crosshair_formatter_diagnostics`, `clear_crosshair_formatter_caches`) for per-axis mode/generation/cache observability
- snapshot/diagnostics coherence hardening tests for crosshair formatter lifecycle state (`tests/api_snapshot_tests.rs`, `tests/property_api_tests.rs`)
- versioned JSON export contracts and backward-compatible parsers for snapshot/diagnostics payloads (`snapshot_json_contract_v1_pretty`, `crosshair_formatter_diagnostics_json_contract_v1_pretty`, `EngineSnapshot::from_json_compat_str`, `CrosshairFormatterDiagnostics::from_json_compat_str`)
- technical API contract matrix for legacy/context crosshair formatters per axis (`docs/crosshair-formatter-contract-matrix.md`)
- lifecycle-transition benchmark coverage for context-aware crosshair formatter cache-hot behavior (`benches/core_math_bench.rs`)
- property-based lifecycle coverage for crosshair formatter transitions (legacy/context set/clear, context invalidation triggers, snapshot parity)
- GTK4/Relm4 integration reference for context-aware crosshair formatter lifecycle wiring (`docs/gtk-relm4-crosshair-formatters.md`)
- GTK4 adapter diagnostics bridge hooks for host observability pipelines (`set_crosshair_diagnostics_hook`, `set_snapshot_json_hook`)
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
- label density scales with zoom level (time visible/full span and price domain/series span) with deterministic min-spacing caps
- time-axis labels support built-in policy+locale and explicit custom formatter injection
- time-axis UTC policies can align to fixed-offset local timezones and optional session windows
- session/day boundary ticks can render with dedicated major grid/label styling
- price-axis labels support fixed/adaptive precision, min-move rounding, and normal/percent/indexed display modes
- price-axis ticks are generated in transformed scale space, then mapped back to raw prices (linear/log/percentage/indexed-safe)
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
- default axis tick-mark visibility follows Lightweight Charts v5.1 baseline (`show_price_axis_tick_marks=false`, `show_time_axis_tick_marks=false`, `show_major_time_tick_marks=false`)
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
- default crosshair guide-line stroke baseline follows Lightweight Charts v5.1 large-dashed style (`crosshair_line_style=LargeDashed`)
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
- axis/crosshair/last-price label layout is section-safe: price-side labels are vertically clamped to plot bounds and time-side labels are clamped to time-axis bounds to prevent section overlap
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
  - price-scale mode switching behavior and autoscale invariants
- `tests/price_scale_mode_display_transform_tests.rs`
  - transformed-domain behavior for `Percentage` / `IndexedTo100` plus axis-pan parity against linear mode
- `tests/price_scale_transformed_base_behavior_tests.rs`
  - explicit/dynamic transformed-base behavior for percentage/indexed scale modes
- `tests/time_scale_coordinate_policy_api_tests.rs`
  - public coordinate->logical-index policy plus ceil/index utility coverage on sparse ranges
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
  - axis label density/collision behavior for narrow/wide viewports, zoom-adaptive cadence, and zoom/pan/price-scale-drag spacing robustness (including minimum zoom-in gain envelope for time-axis density progression)
- `tests/lightweight_axis_tick_cadence_reference_tests.rs`
  - directed Lightweight v5.1 cadence comparisons for time/price axis zoom scenarios, locking intermediate progression (`out < mid <= in`) and non-flat zoom-in gains
- `tests/interaction_touch_pan_tests.rs`
  - touch-pan gate validation and vertical-path normalization behavior (`width` vs `height` axis normalization)
- `tests/visual_fixture_axis_section_sizing_tests.rs`
  - manifest-driven axis-section layout signatures to detect drift in adaptive section sizing, including price-axis zoom-extreme density fixtures
- `tests/visual_fixture_axis_section_png_artifacts_tests.rs`
  - validates that fixture-declared PNG reference artifact paths exist on disk
- `tests/time_axis_formatter_tests.rs`
  - time-axis policy/locale formatting, adaptive zoom behavior, and label-cache hit behavior
- `tests/price_axis_formatter_tests.rs`
  - price-axis formatting policies, display modes, formatter override behavior, and cache-hit/cache-cold validation (including mixed fallback route cache stats behavior)
- `tests/property_price_axis_display_mode_fallback_tests.rs`
  - property invariants for explicit invalid display-mode bases (`0`, `NaN`, `+/-inf`) falling back to deterministic `base_price=1` output
- `tests/property_price_axis_display_mode_none_fallback_tests.rs`
  - property invariants for `base_price=None` equivalence against resolved explicit base and fallback-to-1 behavior when resolved base is zero
- `tests/property_price_axis_display_mode_domain_fallback_tests.rs`
  - property invariants for `base_price=None` domain-based resolution with no series data loaded (`domain_min` path + zero-domain fallback-to-1 path)
- `tests/property_price_axis_display_mode_locale_suffix_fallback_tests.rs`
  - property invariants for mixed fallback routes asserting `%` suffix coherence (`Percentage` vs `IndexedTo100`) and locale decimal separator coherence (`EnUs` `.` vs `EsEs` `,`)
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
