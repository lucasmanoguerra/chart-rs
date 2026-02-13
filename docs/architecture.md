# Architecture Overview

`chart-rs` follows a modular architecture with strict dependency direction.

## Layers

- `core`
  - pure domain model
  - scale math and deterministic transformations
- `interaction`
  - event/state transitions
  - chart intent orchestration
- `render`
  - render contracts
  - backend implementations
- `api`
  - Rust-idiomatic public interface
  - split into focused submodules (for example `api::render_style`, `api::axis_config`, `api::axis_label_format`, `api::axis_ticks`, `api::data_window`, `api::data_controller`, `api::engine_accessors`, `api::axis_label_controller`, `api::price_resolver`, `api::layout_helpers`, `api::snap_resolver`, `api::cache_profile`, `api::plugin_dispatch`, `api::plugin_registry`, `api::interaction_controller`, `api::label_formatter_controller`, `api::scale_access`, `api::time_scale_controller`, `api::series_projection`, `api::snapshot_controller`, `api::json_contract`, `api::render_frame_builder`, `api::visible_window_access`, `api::price_scale_access`, `api::label_cache`, `api::validation`, `api::interaction_validation`) to keep responsibilities narrow
- `platform_gtk` (feature-gated)
  - GTK4/Relm4 adapter
- `extensions`
  - optional feature-gated advanced capabilities

## Dependency Rules

Allowed direction:
- `api` -> `core`, `interaction`, `render`
- `platform_gtk` -> `api`
- `extensions` -> `api`, `core`, `interaction`, `render`

Forbidden direction:
- `core` -> `platform_gtk`
- direct GTK coupling in `core`/`interaction`
- mixed responsibilities inside one module

## Testing Model

Each feature requires:
- unit tests for local invariants
- integration tests for cross-module behavior
- fixture-driven differential tests for parity-sensitive interaction traces against Lightweight reference behavior envelopes (for example time-scale zoom/pan/`rightOffsetPixels` replay, price-scale transformed/autoscale replay, and advanced wheel/pinch/kinetic/crosshair interaction replay)
- property tests with `proptest`, including transform-invariance coverage for price display-mode fallback behavior under invalid explicit bases, `base_price=None` resolution paths, no-data domain-fallback paths, and locale/suffix coherence across mixed fallback routes
- benchmark updates with `criterion` for performance-sensitive paths, including explicit display-mode fallback formatter/cache hot-vs-cold cost tracking, zoom-adaptive axis-density zoom-in/out render-cost tracking, and transformed-base/sparse-index parity benches, with CI guard validation of ratio/latency budgets across all guarded paths
- differential trace import/export tooling is part of the parity workflow (`src/bin/differential_trace_tool.rs`) to convert Lightweight-style captures into deterministic replay fixtures and regenerate expected step metrics
- differential tooling also supports direct raw Lightweight interaction-capture import without manual normalization (`import-lwc-interaction`) with alias mapping for wheel/touch/crosshair event families
- visual differential harness coverage compares Cairo-rendered PNG output against committed Lightweight-style baseline images with explicit max/mean channel-diff tolerance contracts

## Scale Strategy

- `TimeScale`
  - supports bootstrap configuration from `ChartEngineConfig` for initial navigation/right-offset options
  - supports bootstrap configuration from `ChartEngineConfig` for zoom anchoring/limit policies
  - supports bootstrap configuration from `ChartEngineConfig` for edge/resize/realtime-append policies
  - exposes Lightweight-style logical-index transform primitives through `TimeIndexCoordinateSpace` (`index_to_coordinate`, `coordinate_to_logical_index`, ceil discrete index conversion, right-offset pan/zoom anchor helpers)
  - can derive and rebuild visible range from explicit `barSpacing/rightOffset` runtime state (`derive_visible_bar_spacing_and_right_offset`, `set_visible_range_from_bar_spacing_and_right_offset`)
  - tracks full range and visible range
  - supports fit-to-data with configurable left/right padding
  - supports optional fixed-edge clamping of visible range against full-range bounds (`fix_left_edge` / `fix_right_edge`)
  - supports optional right-offset/spacing synthesis based on reference time-step estimation (`right_offset_bars` / `bar_spacing_px`)
  - supports optional zoom-spacing bounds (`TimeScaleZoomLimitBehavior`) for deterministic zoom-out/zoom-in clamp behavior
  - supports optional pixel-based right-margin override with priority over bar-based offset (`time_scale_right_offset_px`), including zoom-limit/resize hardening that preserves pixel-margin semantics under constrained spans and extreme resize/zoom-limit transitions
  - supports optional scroll-zoom right-edge anchoring policy (`right_bar_stays_on_scroll`)
  - supports optional viewport-resize anchoring policy (`Left` / `Center` / `Right`) with lock toggle for deterministic range updates under width changes
  - supports optional realtime append tail-tracking policy with right-edge tolerance controls (`preserve_right_edge_on_append` / `right_edge_tolerance_bars`)
  - supports optional realtime price autoscale policy on incremental data updates (`PriceScaleRealtimeBehavior`)
  - supports deterministic scroll-to-realtime reattachment command (`scroll_time_to_realtime`) composed with navigation and edge constraints
  - supports deterministic bar-based scroll position introspection and explicit positioning (`time_scroll_position_bars` / `scroll_time_to_position_bars`)
  - exposes public logical-index utilities for host adapters: coordinate->logical mapping policy, discrete ceil conversion, logical->coordinate projection, nearest filled-slot introspection, and sparse next/prev navigation (`map_pixel_to_logical_index`, `map_pixel_to_logical_index_ceil`, `map_logical_index_to_pixel`, `nearest_filled_logical_slot_at_pixel`, `next_filled_logical_index`, `prev_filled_logical_index`)
- interaction input paths can be host-gated with explicit scroll/scale toggles plus granular wheel/drag/pinch controls (`InteractionInputBehavior`) while pointer-crosshair flow remains independent
- interaction input gates can be bootstrapped through `ChartEngineConfig` for deterministic startup behavior
- touch-style pan path uses explicit API (`touch_drag_pan_time_visible`) with independent horizontal/vertical gate semantics, axis-specific input validation, and dominant-axis normalization (`width` for horizontal, `height` for vertical)
- price-axis vertical panning path uses explicit API (`axis_drag_pan_price`) with anchor-preserving domain translation, finite-input validation, and no-op semantics when axis drag-scale interaction is gated off
- axis double-click reset paths use explicit APIs (`axis_double_click_reset_price_scale`, `axis_double_click_reset_time_scale`) with `InteractionInputBehavior::scale_axis_double_click_reset` gate semantics and deterministic changed/no-change signaling
- realtime data-ingest update paths enforce non-decreasing time and deterministic append-or-replace semantics (`update_point` / `update_candle`)
- realtime data-ingest paths can optionally trigger deterministic best-effort price autoscale refresh after full-replacement (`set_*`) and append/update mutations
- realtime autoscale policies can be bootstrapped via `ChartEngineConfig` for deterministic startup behavior
- `PriceScale`
  - supports bootstrap configuration from `ChartEngineConfig` for mode/inversion/margins
  - supports tuned autoscale from points or candles
  - exposes transformed-domain coordinate primitives through `PriceCoordinateSpace` with explicit margin/internal-height mapping semantics
  - supports `PriceScaleMode::Linear`, `PriceScaleMode::Log`, `PriceScaleMode::Percentage`, and `PriceScaleMode::IndexedTo100`
  - supports optional inverted mapping direction for `invertScale` parity
  - supports optional top/bottom axis whitespace margins for `scaleMargins` parity
  - log mode validates strictly-positive domains and applies tuning in transformed log space
  - transformed display modes resolve a non-zero base value and keep deterministic coordinate roundtrip behavior
  - transformed display modes support explicit and dynamic base-source policies (`DomainStart`, `First/LastData`, `First/LastVisibleData`) with runtime refresh hooks, deterministic cross-series timestamp precedence, candle tie-break for equal timestamps, and visible-window fallback to full-data candidates
  - log mode tick selection favors deterministic 1/2/5 decade ladders for axis-label parity
  - always maps higher prices to smaller Y pixel values (inverted axis)

Tuning contracts:
- all ratio values must be finite and non-negative
- degenerate ranges are expanded with explicit minimum span values
- tuned domains must contain source data ranges

## Interaction Strategy

- interaction state is stored in `interaction` module
- data/candle snapping decisions are computed in `api` layer
- crosshair baseline:
  - startup crosshair mode is configurable at engine bootstrap (`ChartEngineConfig::crosshair_mode`)
  - pointer move sets visible state and updates coordinates
  - nearest snap candidate is selected from points/candles, with sparse-index nearest-slot hints (`ignoreWhitespace`-style) plus exact-distance fallback to preserve deterministic nearest-pixel behavior on irregular windows
  - hidden mode keeps crosshair state non-visible on pointer movement
  - pointer leave hides crosshair and clears snap state

## Render Strategy

- `api` builds a deterministic `RenderFrame` containing backend-agnostic line/rect/text primitives
- axis tick density is zoom-adaptive (time visible/full span and price domain/series span) and filtered with deterministic spacing rules; layout tests lock collision-safe and near-even label spacing under zoom/pan/price-scale-drag interaction changes
- zoom-density scaling uses a deterministic non-linear zoom-ratio curve with neutral-band stabilization (`density_scale_from_zoom_ratio`) so intermediate zoom ranges remain cadence-responsive without breaking min-spacing guards
- time-axis collision filtering applies major/minor-prioritized spacing selection so major labels remain stable under mixed zoom-density pressure (`select_positions_with_min_spacing_prioritized`)
- directed Lightweight v5.1 cadence references include intermediate time-zoom windows and multi-step price-scale zoom windows to validate progression envelopes beyond simple out/in monotonic checks
- zoom-adaptive axis-density render-cost drift is guarded by criterion benches and CI budget checks (`axis_density_zoom_adaptive_out_render`, `axis_density_zoom_adaptive_in_render`, `scripts/check_axis_density_bench_regressions.py`)
- transformed-base and sparse logical-index hot-path drift is guarded by criterion benches and CI budget checks (`price_scale_transformed_base_dynamic_refresh_visible_window`, `time_scale_sparse_nearest_filled_slot_lookup`, `time_scale_sparse_next_prev_filled_lookup`, `scripts/check_new_parity_bench_regressions.py`)
- visual regression drift is guarded by fixture-driven PNG diffs (`tests/lightweight_visual_differential_tests.rs`, `tests/fixtures/lightweight_visual_differential/visual_baseline_corpus.json`) using explicit per-fixture channel-diff budgets
- visual fixture corpus includes candlestick/log-scale + price-axis drag-scale and session/timezone + time-axis drag-scale cases to stress both axes under parity image diffs
- CI includes a dedicated parity guard job for visual/property drift (`parity_visual_property_guard` in `.github/workflows/ci.yml`) running visual PNG diffs and elevated proptest stress for extreme scale round-trip invariants
- time-axis labels are produced via policy+locale config with optional custom formatter injection
- time-axis formatter supports zoom-aware adaptive detail, fixed-offset timezone alignment, and optional session-boundary semantics
- time-axis major ticks (session boundaries/local-midnight) can be emphasized through deterministic style knobs
- price-axis labels support fixed/adaptive precision, min-move rounding, and normal/percent/indexed display modes via deterministic API config
- latest-price line/label marker can be rendered from newest point/candle sample with style toggles
- axis-section sizing applies a deterministic adaptive pass so time/price panel dimensions can expand when formatter/typography pressure exceeds requested static sizes
- axis-section visual-fixture manifests lock deterministic layout signatures for baseline/pressure scenarios, providing drift guards for adaptive sizing changes
- fixture schema supports explicit time visible-range overrides for zoom-extreme cadence regression coverage (`time_visible_range_override`), deterministic price-domain lock for cadence stress (`disable_autoscale_on_data_set`), and optional replay of price-axis scale interactions (`price_axis_scale_steps`)
- fixture manifests can drive deterministic Cairo PNG reference export for manual visual-review workflows without changing core render-frame contracts
- manual baseline capture under `reference_UI/` complements fixture PNG review with a stable GTK-host visual sanity anchor, and PR workflow includes an explicit visual-check gate for render/layout changes
- manual visual-review checklist is standardized by change type (`layout`, `render`, `formatter`) to keep PR reviews consistent across geometry, style, and formatter changes
- fixture corpus includes extreme/sparse layout stress cases (tiny viewports, narrow high-precision domains, sparse wide-range data) to harden adaptive-sizing regression detection
- fixture corpus includes mixed price display-mode stress for large-value ranges (`Normal`, `Percentage`, `IndexedTo100`) to monitor transform-sensitive axis-width drift
- fixture corpus includes explicit invalid-base fallback stress for display transforms (`base_price=0`, `NaN`, `+inf`, `-inf`) in `Percentage`/`IndexedTo100`, with non-finite fixture literals supplied through manifest override tokens
- fixture corpus includes dedicated price-axis zoom-extreme cadence fixtures (`zoom-extreme-axis-density-price-out`, `zoom-extreme-axis-density-price-in`) in addition to time-axis zoom-extreme fixtures
- price-axis label selection can exclude deterministic overlap zones around the last-price marker
- last-price marker can optionally resolve deterministic up/down/neutral colors from latest vs previous sample
- last-price marker source policy can switch between full-series latest sample and newest visible-range sample
- latest-price axis label can optionally render as a deterministic filled price-box on the axis panel
- latest-price price-box can apply deterministic border/radius styling and auto-contrast text color policy
- latest-price price-box width policy supports deterministic full-axis or fit-text modes with explicit horizontal padding/min-width guards
- price-axis label right inset and axis tick-mark extension length are deterministic style-level controls
- price-axis tick-mark color/width are style-level controls independent from axis border styling
- price-axis label font size/vertical offset are deterministic style-level controls
- last-price label vertical offset is a deterministic style-level control independent from font-size
- last-price label right inset is a deterministic style-level control independent from regular axis-label inset
- price-axis short tick-mark visibility is a deterministic style-level control
- price-axis horizontal grid-line visibility is a deterministic style-level control
- price-axis regular-label visibility is a deterministic style-level control
- price-axis horizontal grid-line color/width are deterministic style-level controls independent from time-grid styling
- time-axis regular-label font size/offset/tick length are deterministic style-level controls
- time-axis regular-label visibility is a deterministic style-level control
- time-axis short tick-mark visibility is a deterministic style-level control
- time-axis short tick-mark color/width are deterministic style-level controls independent from axis-border styling
- time-axis label color is a deterministic style-level control independent from price-axis label color
- major time-axis label visibility is a deterministic style-level control independent from regular time labels
- major time-axis grid visibility is a deterministic style-level control independent from regular time-grid lines
- major time-axis label color is a deterministic style-level control independent from regular time-axis label color
- major time-axis label vertical offset is a deterministic style-level control independent from regular time-axis labels
- major time-axis tick-mark color/width/length are deterministic style-level controls independent from regular time-axis tick-mark styling
- major time-axis tick-mark visibility is a deterministic style-level control independent from regular time-axis tick marks
- time-axis border visibility is a deterministic style-level control independent from right-side price-axis border visibility
- price-axis border visibility is a deterministic style-level control independent from bottom time-axis border visibility
- crosshair guide-line color/width and horizontal/vertical visibility are deterministic style-level controls resolved from interaction state
- crosshair guide-line color is independently configurable per axis with shared fallback (`crosshair_horizontal_line_color`, `crosshair_vertical_line_color`, `crosshair_line_color`)
- crosshair guide-line width is independently configurable per axis with shared fallback (`crosshair_horizontal_line_width`, `crosshair_vertical_line_width`, `crosshair_line_width`)
- crosshair guide-line visibility supports deterministic shared gating with per-axis toggles (`show_crosshair_lines && show_crosshair_{horizontal,vertical}_line`)
- visual diff corpus artifacts are part of the render contract surface; intentional style-default changes must be synchronized by baseline regeneration before CI parity guard execution
- crosshair time/price axis-label color, font-size, and visibility are deterministic style-level controls resolved from interaction snap state
- crosshair time/price axis labels support deterministic independent formatter overrides with fallback to axis formatter policies
- crosshair time/price axis labels support deterministic prefix/suffix text transforms with shared fallback and per-axis overrides
- crosshair time/price axis labels support deterministic numeric-precision overrides with shared fallback and per-axis controls
- crosshair time/price formatter overrides can receive deterministic context (visible span + source mode) without leaking interaction internals into renderer backends
- context-aware crosshair formatter caches partition deterministically by formatter generation, source mode, visible span, and quantized label inputs
- context-aware crosshair formatter caches are invalidated on crosshair-mode and visible-range lifecycle transitions to keep cache state bounded and deterministic
- engine snapshots export deterministic crosshair formatter lifecycle state (per-axis override mode + generation counters) for regression/debug tooling
- formatter lifecycle introspection is exposed via explicit API accessors so host adapters can observe mode/generation without touching internals
- a consolidated diagnostics surface exposes per-axis formatter mode/generation/cache state for host debug and health probes
- a technical contract matrix documents legacy/context per-axis formatter lifecycle semantics and snapshot/diagnostics coherence expectations (`docs/crosshair-formatter-contract-matrix.md`)
- snapshot/diagnostics exports support versioned JSON contracts with schema guards plus backward-compatible parsing of legacy raw payloads
- property-level lifecycle tests cover formatter mode transitions, context invalidation boundaries, and snapshot export determinism
- GTK4/Relm4 adapter flows should treat crosshair formatter updates as explicit message-driven state transitions (pointer/mode/range events) and may bridge diagnostics/snapshot contract payloads through draw-time hooks for host observability
- crosshair time/price axis-label boxes support deterministic fit-text sizing with style-level fill/padding and independent per-axis visibility controls
- crosshair axis-label boxes support deterministic border/radius styling with clamped corner geometry for backend-stable output
- crosshair axis-label boxes support deterministic manual or auto-contrast text-color resolution without backend-specific text-measurement dependencies
- crosshair axis-label boxes support deterministic width-mode resolution (`FitText`/`FullAxis`) with shared default and per-axis overrides
- crosshair axis-label box border visibility can be toggled deterministically and independently for time/price axis labels
- crosshair axis-label anchor Y offsets are deterministic and independently configurable for time/price labels
- crosshair axis-label horizontal insets are deterministic and independently configurable for time/price labels
- crosshair axis-label font sizes are deterministic and independently configurable for time/price labels
- crosshair axis-label box paddings are deterministic and independently configurable per axis/panel
- crosshair axis-label box border style is deterministic and independently configurable per axis/panel
- crosshair axis-label box corner radius is deterministic and independently configurable per axis/panel
- crosshair axis-label box text policy is deterministic and independently configurable per axis/panel
- crosshair axis-label box fill color is deterministic and independently configurable per axis/panel
- crosshair axis-label box minimum width is deterministic and independently configurable per axis/panel
- crosshair axis-label box text alignment is deterministic and independently configurable per axis/panel
- crosshair axis-label box vertical anchor is deterministic and independently configurable per axis/panel
- crosshair axis-label box horizontal anchor is deterministic and independently configurable per axis/panel
- crosshair axis-label box overflow policy is deterministic and independently configurable per axis/panel
- crosshair axis-label box visibility priority is deterministic and independently configurable per axis/panel
- crosshair axis-label box clipping margin is deterministic and independently configurable per axis/panel
- crosshair axis-label box stabilization step is deterministic and independently configurable per axis/panel
- crosshair axis-label box z-order is deterministic and independently configurable per axis/panel
- crosshair guide-line stroke style is deterministic and independently configurable per axis/panel
- in-engine price-label caching reuses deterministic label text across repeated redraws
- in-engine time-label caching keeps redraw behavior deterministic under all formatter policies
- in-engine crosshair override formatter caching keeps per-axis override redraw behavior deterministic with explicit fallback to axis-label cache paths
- plot and price-axis panels are styled through a deterministic render-style contract
- `render` backends execute primitives only (no scale math or interaction decisions)
- `cairo-backend` supports:
  - offscreen image-surface rendering
  - external cairo-context rendering (used by GTK `DrawingArea`)
- `platform_gtk` keeps widget lifecycle/event wiring isolated from render/domain code
- `platform_gtk` may expose thin host-facing bridge helpers for engine coordinate/introspection APIs (for example sparse logical-index nearest/next/prev lookup utilities) without re-implementing core math
