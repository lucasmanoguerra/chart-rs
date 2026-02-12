# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog,
and this project adheres to Semantic Versioning.

## [Unreleased]

### Added
- Price-scale visible-window autoscale parity baseline (`C-058`) with deterministic opt-in autoscale refresh on time-range navigation changes.
- Extended `PriceScaleRealtimeBehavior` with `autoscale_on_time_range_change` and wired it through core visible-range lifecycle flow (`emit_visible_range_changed`) covering pan/zoom/fit/scroll/resize paths.

## [0.0.34-alpha.0] - 2026-02-12

### Added
- Defaults parity block (`C-056`) aligning core v5.1 baseline knobs available in the Rust API:
  `priceScale.scaleMargins` default (`top=0.2`, `bottom=0.1`), crosshair large-dashed baseline stroke style, and default hidden axis tick marks.
- Layout parity block (`C-057`) adding strict section-boundary clipping to prevent overlap between plot, time-axis, and price-axis sections under edge and overflow scenarios.
- New section-boundary regression coverage in `tests/render_axis_layout_tests.rs` for price/last-price labels and crosshair label boxes.
- Bootstrap crosshair axis-label box style config parity baseline (`C-055`) with deterministic engine-init support for crosshair time/price label-box fill/border/corner policy.
- Crosshair axis-label box style API parity baseline (`C-054`) with dedicated controller methods `crosshair_axis_label_box_style_behavior` and `set_crosshair_axis_label_box_style_behavior`.
- Bootstrap crosshair axis-label style config parity baseline (`C-053`) with deterministic engine-init support for crosshair time/price label color/font/offset/inset policy.
- Crosshair axis-label style API parity baseline (`C-052`) with dedicated controller methods `crosshair_axis_label_style_behavior` and `set_crosshair_axis_label_style_behavior`.
- Bootstrap crosshair axis-label visibility config parity baseline (`C-051`) with deterministic engine-init support for time/price label/box/border visibility toggles.
- Crosshair axis-label visibility API parity baseline (`C-050`) with dedicated controller methods `crosshair_axis_label_visibility_behavior` and `set_crosshair_axis_label_visibility_behavior`.
- Bootstrap crosshair guide-line stroke-style config parity baseline (`C-049`) with deterministic engine-init support for shared/per-axis guide-line color/width/style policy.
- Crosshair guide-line stroke-style API parity baseline (`C-048`) with dedicated controller methods `crosshair_guide_line_style_behavior` and `set_crosshair_guide_line_style_behavior`.
- Bootstrap crosshair guide-line visibility config parity baseline (`C-047`) with deterministic engine-init support for shared/per-axis guide-line toggles.
- Crosshair guide-line visibility API parity baseline (`C-046`) with dedicated controller methods `crosshair_guide_line_behavior` and `set_crosshair_guide_line_behavior`.
- Property-test parity baseline (`C-045`) for full-replacement data canonicalization invariants in `set_data` / `set_candles`.
- Bootstrap last-price behavior config parity baseline (`C-044`) with deterministic engine-init support for line/label visibility, trend-color mode, and source-mode policy via `last_price_behavior`.
- Last-price behavior API parity baseline (`C-043`) with dedicated controller methods `last_price_behavior` and `set_last_price_behavior`.
- Full-replacement data canonicalization parity baseline (`C-042`) with deterministic invalid-sample filtering, time sorting, and duplicate-timestamp replacement in `set_data` / `set_candles`.
- Bootstrap last-price source-mode config parity baseline (`C-041`) with deterministic engine-init support for `RenderStyle.last_price_source_mode`.
- Bootstrap axis-label formatter-policy config parity baseline (`C-040`) with deterministic engine-init support for time-axis and price-axis label bootstrap options.
- Bootstrap time-scale edge/resize/realtime-policy config parity baseline (`C-039`) with deterministic engine-init support for constraint/resize/realtime append behavior.
- Bootstrap time-scale zoom-policy config parity baseline (`C-038`) with deterministic engine-init support for right-edge scroll-zoom anchoring and zoom limits.
- Bootstrap price-scale realtime-policy config parity baseline (`C-037`) with deterministic engine-init support for autoscale-on-set/update behavior.
- Bootstrap time-scale navigation config parity baseline (`C-036`) with deterministic engine-init support for initial navigation/right-offset behavior.
- Bootstrap interaction-input config parity baseline (`C-035`) with deterministic engine-init support for initial scroll/scale gate settings.
- Bootstrap price-scale config parity baseline (`C-034`) with deterministic engine-init support for initial mode/inversion/margins.
- Price-scale autoscale-on-set parity baseline (`C-033`) with opt-in autoscale refresh on full data replacement (`set_data` / `set_candles`).
- Crosshair hidden-mode parity baseline (`C-032`) with deterministic `CrosshairMode::Hidden` behavior.
- Price-scale margins parity baseline (`C-031`) with deterministic top/bottom whitespace controls (`scaleMargins`).
- New price-scale API contract: `PriceScaleMarginBehavior` plus controller methods `price_scale_margin_behavior` and `set_price_scale_margin_behavior`.
- Price-scale inversion parity baseline (`C-030`) with deterministic `invertScale` mapping behavior.
- New price-scale API methods: `price_scale_inverted` and `set_price_scale_inverted`.
- Touch-drag interaction parity baseline (`C-029`) with deterministic horizontal/vertical touch pan gates.
- New time-scale API method: `touch_drag_pan_time_visible(delta_x_px, delta_y_px)` integrating `scroll_horz_touch_drag` / `scroll_vert_touch_drag` behavior.
- Price-scale realtime autoscale parity baseline (`C-028`) with opt-in autoscale refresh on incremental append/update flows.
- New price-scale API contract: `PriceScaleRealtimeBehavior` plus controller methods `price_scale_realtime_behavior` and `set_price_scale_realtime_behavior`.
- Crosshair startup-mode parity baseline (`C-027`) with deterministic bootstrap configuration support.
- Extended `ChartEngineConfig` with `crosshair_mode` and builder `with_crosshair_mode`, applied during engine initialization.
- Time-scale zoom-limit parity baseline (`C-026`) with deterministic bar-spacing bounds aligned to `minBarSpacing` behavior.
- New time-scale API contract: `TimeScaleZoomLimitBehavior` with `min_bar_spacing_px` and optional `max_bar_spacing_px`, plus controller methods `time_scale_zoom_limit_behavior` and `set_time_scale_zoom_limit_behavior`.
- Interaction input parity baseline (`C-025`) with granular `handleScroll`/`handleScale` sub-path gates.
- Extended `InteractionInputBehavior` with per-input toggles: `scroll_mouse_wheel`, `scroll_pressed_mouse_move`, `scroll_horz_touch_drag`, `scroll_vert_touch_drag`, `scale_mouse_wheel`, and `scale_pinch`.
- Scroll-zoom anchoring parity baseline (`C-024`) with deterministic `rightBarStaysOnScroll` policy for wheel/pinch zoom.
- New time-scale API contract: `TimeScaleScrollZoomBehavior` plus controller methods `time_scale_scroll_zoom_behavior` and `set_time_scale_scroll_zoom_behavior`.
- New integration coverage: `tests/time_scale_scroll_zoom_behavior_tests.rs`.
- Pixel right-margin parity baseline (`C-023`) with deterministic `rightOffsetPixels` priority semantics over bar-based right offset.
- New time-scale API methods: `time_scale_right_offset_px` and `set_time_scale_right_offset_px`.
- New integration coverage: `tests/time_scale_right_offset_pixels_tests.rs`.
- Scroll-position parity baseline (`C-022`) with deterministic bar-based introspection and direct positioning aligned to Lightweight Charts `scrollPosition`/`scrollToPosition`.
- New time-scale API methods: `time_scroll_position_bars` and `scroll_time_to_position_bars`.
- New integration coverage: `tests/time_scale_scroll_position_tests.rs`.
- Scroll-to-realtime parity baseline (`C-021`) with deterministic command semantics aligned to Lightweight Charts `scrollToRealTime`.
- New time-scale API method: `scroll_time_to_realtime`.
- New integration coverage: `tests/time_scale_scroll_to_realtime_tests.rs`.
- Realtime update parity baseline (`C-020`) with deterministic append-or-replace semantics aligned to non-decreasing timestamp feeds.
- New incremental API methods: `update_point` and `update_candle` (append when newer, replace when equal, reject older timestamps).
- New integration coverage: `tests/realtime_update_semantics_tests.rs`.
- Realtime append parity baseline (`C-019`) with deterministic continuous time-range shift when the viewport is tracking the right edge.
- New API contract: `TimeScaleRealtimeAppendBehavior` plus controller methods `time_scale_realtime_append_behavior` and `set_time_scale_realtime_append_behavior`.
- Realtime append integration in `append_point`/`append_candle` with right-edge tolerance policy and composition with time-scale navigation/edge constraints.
- New integration coverage: `tests/time_scale_realtime_append_behavior_tests.rs`.
- Viewport-resize anchoring parity baseline (`C-018`) with deterministic visible-range lock behavior across viewport width changes.
- New API contracts: `TimeScaleResizeAnchor` and `TimeScaleResizeBehavior`, plus controller methods `time_scale_resize_behavior` and `set_time_scale_resize_behavior`.
- New integration coverage: `tests/time_scale_resize_behavior_tests.rs`.
- Time-scale right-offset/spacing parity baseline (`C-017`) with deterministic navigation synthesis using `right_offset_bars` and optional `bar_spacing_px`.
- New API contract: `TimeScaleNavigationBehavior` plus controller methods `time_scale_navigation_behavior` and `set_time_scale_navigation_behavior`.
- New integration coverage: `tests/time_scale_navigation_behavior_tests.rs`.
- Interaction input gating parity baseline (`C-016`) with deterministic scroll/scale enable flags aligned to `handleScroll` / `handleScale` behavior families.
- New API contract: `InteractionInputBehavior` plus controller methods `interaction_input_behavior` and `set_interaction_input_behavior`.
- New interaction API path: `pinch_zoom_time_visible` with no-op semantics when scale handling is disabled.
- New integration coverage: `tests/interaction_input_behavior_tests.rs`.
- Time-scale fixed-edge parity baseline (`C-015`) with deterministic `fix_left_edge` / `fix_right_edge` navigation constraints against full-range bounds.
- New API contract: `TimeScaleEdgeBehavior` plus controller methods `time_scale_edge_behavior` and `set_time_scale_edge_behavior`.
- New integration coverage: `tests/time_scale_edge_behavior_tests.rs`.
- GTK/Relm4 diagnostics bridge parity baseline (`R-089`) with adapter hooks to publish crosshair formatter diagnostics and versioned snapshot JSON during draw lifecycle.
- New GTK adapter APIs: `set_crosshair_diagnostics_hook`, `clear_crosshair_diagnostics_hook`, `set_snapshot_json_hook`, `clear_snapshot_json_hook`, `crosshair_formatter_diagnostics_json_contract_v1_pretty`, and `snapshot_json_contract_v1_pretty`.
- Snapshot/diagnostics schema guard parity baseline (`R-088`) with backward-compatible JSON parsers that accept both legacy raw payloads and versioned contract wrappers.
- New compatibility APIs: `EngineSnapshot::from_json_compat_str` and `CrosshairFormatterDiagnostics::from_json_compat_str`.
- Crosshair diagnostics JSON export parity baseline (`R-087`) with stable raw and versioned (`v1`) export contracts.
- New JSON contract APIs: `snapshot_json_contract_v1_pretty`, `crosshair_formatter_diagnostics_json_pretty`, and `crosshair_formatter_diagnostics_json_contract_v1_pretty`.
- New contract payload types: `EngineSnapshotJsonContractV1`, `CrosshairFormatterDiagnosticsJsonContractV1`, and schema constants `ENGINE_SNAPSHOT_JSON_SCHEMA_V1` / `CROSSHAIR_DIAGNOSTICS_JSON_SCHEMA_V1`.
- Crosshair snapshot/diagnostics coherence hardening parity baseline (`R-086`) with integration/property assertions that `EngineSnapshot.crosshair_formatter` and `crosshair_formatter_diagnostics()` remain mode/generation-aligned across lifecycle transitions.
- New snapshot/diagnostics coherence tests in `tests/api_snapshot_tests.rs` and `tests/property_api_tests.rs`.
- Crosshair legacy/context API contract matrix parity baseline (`R-085`) with explicit per-axis lifecycle/action mapping for mode, storage slots, and generation semantics.
- New technical document: `docs/crosshair-formatter-contract-matrix.md`.
- Crosshair lifecycle-transition benchmark parity baseline (`R-084`) with deterministic context-cache hot-path coverage under crosshair-mode/visible-range changes.
- New benchmark: `bench_crosshair_axis_label_formatter_context_lifecycle_transitions` in `benches/core_math_bench.rs`.
- Crosshair formatter diagnostics parity baseline (`R-083`) with consolidated per-axis override-mode, generation, and cache-stat observability APIs.
- New API methods: `crosshair_formatter_diagnostics` and `clear_crosshair_formatter_caches`.
- New diagnostics contract: `CrosshairFormatterDiagnostics`.
- GTK4/Relm4 integration docs parity baseline (`R-082`) with practical context-aware crosshair formatter wiring guidance for host applications.
- New integration document: `docs/gtk-relm4-crosshair-formatters.md`.
- Crosshair formatter lifecycle property-coverage parity baseline (`R-081`) with deterministic mixed-transition scenarios for legacy/context overrides, invalidation triggers, and snapshot export roundtrip.
- New property-test coverage for crosshair formatter lifecycle and snapshot parity (`tests/property_api_tests.rs`).
- Crosshair axis-label formatter API hardening parity baseline (`R-080`) with deterministic override-mode and generation introspection per axis.
- New API methods: `crosshair_time_label_formatter_override_mode`, `crosshair_price_label_formatter_override_mode`, and `crosshair_label_formatter_generations`.
- Crosshair axis-label formatter snapshot/export parity baseline (`R-079`) with deterministic snapshot fields for per-axis override mode and formatter generations.
- New snapshot contracts: `CrosshairFormatterOverrideMode` and `CrosshairFormatterSnapshot` included in `EngineSnapshot`.
- New snapshot test coverage for crosshair formatter state export/roundtrip behavior.
- Crosshair axis-label formatter context invalidation lifecycle parity baseline (`R-078`) with deterministic cache invalidation on crosshair-mode and visible-range transitions.
- New render-frame test coverage for crosshair formatter-context cache lifecycle invalidation behavior.
- Crosshair axis-label formatter context cache-key parity baseline (`R-077`) with deterministic per-axis cache partitioning by formatter generation, source mode, visible span, and quantized label inputs.
- New render-frame/property tests and criterion benchmark coverage for crosshair formatter-context cache-key behavior.
- Crosshair axis-label formatter context parity baseline (`R-076`) with deterministic per-axis context delivery (visible span and source mode) for crosshair formatter overrides.
- New API methods: `set_crosshair_time_label_formatter_with_context`, `clear_crosshair_time_label_formatter_with_context`, `set_crosshair_price_label_formatter_with_context`, and `clear_crosshair_price_label_formatter_with_context`.
- New public context contracts: `CrosshairTimeLabelFormatterContext`, `CrosshairPriceLabelFormatterContext`, and `CrosshairLabelSourceMode`.
- New render-frame/property tests and criterion benchmark coverage for crosshair axis-label formatter-context behavior.
- Crosshair axis-label numeric precision parity baseline (`R-075`) with deterministic shared precision fallback plus independent per-axis precision overrides for time/price crosshair labels.
- New render-style knobs: `crosshair_label_numeric_precision`, `crosshair_time_label_numeric_precision`, and `crosshair_price_label_numeric_precision`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label numeric-precision behavior.
- Crosshair axis-label text transform parity baseline (`R-074`) with deterministic shared prefix/suffix fallback plus independent per-axis overrides for time/price crosshair labels.
- New render-style knobs: `crosshair_label_prefix`, `crosshair_label_suffix`, `crosshair_time_label_prefix`, `crosshair_time_label_suffix`, `crosshair_price_label_prefix`, and `crosshair_price_label_suffix`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label text-transform behavior.
- Crosshair axis-label formatter fallback/cache parity baseline (`R-073`) with deterministic per-axis cache policy for formatter overrides.
- New API methods: `crosshair_time_label_cache_stats`, `clear_crosshair_time_label_cache`, `crosshair_price_label_cache_stats`, and `clear_crosshair_price_label_cache`.
- New render-frame/property tests and criterion benchmark coverage for crosshair formatter override cache-hot behavior.
- Crosshair axis-label formatter override parity baseline (`R-072`) with deterministic independent formatter overrides for time and price crosshair labels.
- New API methods: `set_crosshair_time_label_formatter`, `clear_crosshair_time_label_formatter`, `set_crosshair_price_label_formatter`, and `clear_crosshair_price_label_formatter`.
- New render-frame/property tests and criterion benchmark coverage for crosshair axis-label formatter override behavior.
- Crosshair guide-line combined visibility gate parity baseline (`R-071`) with deterministic shared visibility control composed with per-axis line toggles.
- New render-style knob: `show_crosshair_lines` combined with `show_crosshair_horizontal_line` and `show_crosshair_vertical_line`.
- New render-frame/style/property tests and criterion benchmark coverage for shared crosshair guide-line visibility gating behavior.
- Crosshair guide-line per-axis width parity baseline (`R-070`) with deterministic independent width controls for horizontal and vertical crosshair lines.
- New render-style knobs: `crosshair_horizontal_line_width` and `crosshair_vertical_line_width` with shared fallback to `crosshair_line_width`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair guide-line per-axis width behavior.
- Crosshair guide-line per-axis color parity baseline (`R-069`) with deterministic independent color controls for horizontal and vertical crosshair lines.
- New render-style knobs: `crosshair_horizontal_line_color` and `crosshair_vertical_line_color` with shared fallback to `crosshair_line_color`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair guide-line per-axis color behavior.
- Crosshair guide-line per-axis stroke-style parity baseline (`R-068`) with deterministic independent dash-pattern controls for horizontal and vertical crosshair lines.
- New render-style knobs: `crosshair_line_style`, `crosshair_horizontal_line_style`, and `crosshair_vertical_line_style`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair guide-line per-axis stroke-style behavior.

### Changed
- Internal API modularization: moved render-style enums and `RenderStyle` default contract from `src/api/mod.rs` into `src/api/render_style.rs` with unchanged public re-exports.
- Internal API modularization: moved axis formatter policies/configs into `src/api/axis_config.rs`, axis-label formatting/quantization helpers into `src/api/axis_label_format.rs`, axis tick/window helpers into `src/api/axis_ticks.rs` and `src/api/data_window.rs`, data mutation methods into `src/api/data_controller.rs`, engine metadata/data/viewport accessor methods into `src/api/engine_accessors.rs`, axis label config controller methods into `src/api/axis_label_controller.rs`, price-resolution helpers into `src/api/price_resolver.rs`, layout helpers into `src/api/layout_helpers.rs`, crosshair snap helpers into `src/api/snap_resolver.rs`, cache-profile helpers into `src/api/cache_profile.rs`, plugin dispatch helpers into `src/api/plugin_dispatch.rs`, plugin registry methods into `src/api/plugin_registry.rs`, interaction controller methods into `src/api/interaction_controller.rs`, label formatter/cache lifecycle methods into `src/api/label_formatter_controller.rs`, scale-access methods into `src/api/scale_access.rs`, time-scale range/pan/zoom/fit controller methods into `src/api/time_scale_controller.rs`, series geometry/marker projection methods into `src/api/series_projection.rs`, snapshot serialization/state export methods into `src/api/snapshot_controller.rs`, render-frame assembly and axis/crosshair label formatting helpers into `src/api/render_frame_builder.rs`, visible-window access methods into `src/api/visible_window_access.rs`, price-scale access methods into `src/api/price_scale_access.rs`, interaction validation into `src/api/interaction_validation.rs`, label-cache types/logic into `src/api/label_cache.rs`, and validation routines into `src/api/validation.rs` with unchanged public re-exports.

### Added
- Crosshair axis-label box per-axis z-order parity baseline (`R-067`) with deterministic independent draw-order controls for time and price label boxes.
- New render-style knobs: `crosshair_label_box_z_order_policy`, `crosshair_time_label_box_z_order_policy`, and `crosshair_price_label_box_z_order_policy`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis z-order behavior.
- Crosshair axis-label box per-axis jitter-stabilization parity baseline (`R-066`) with deterministic independent position-quantization controls for time and price label boxes.
- New render-style knobs: `crosshair_label_box_stabilization_step_px`, `crosshair_time_label_box_stabilization_step_px`, and `crosshair_price_label_box_stabilization_step_px`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis jitter-stabilization behavior.
- Crosshair axis-label box per-axis clipping-margin parity baseline (`R-065`) with deterministic independent clip-inset controls for time and price label boxes under `ClipToAxis`.
- New render-style knobs: `crosshair_label_box_clip_margin_px`, `crosshair_time_label_box_clip_margin_px`, and `crosshair_price_label_box_clip_margin_px`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis clipping-margin behavior.
- Crosshair axis-label box per-axis visibility-priority parity baseline (`R-064`) with deterministic overlap resolution controls for time and price label boxes.
- New render-style knobs: `crosshair_label_box_visibility_priority`, `crosshair_time_label_box_visibility_priority`, and `crosshair_price_label_box_visibility_priority`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis visibility-priority behavior.
- Crosshair axis-label box per-axis overflow-policy parity baseline (`R-063`) with deterministic independent `ClipToAxis`/`AllowOverflow` controls for time and price label boxes.
- New render-style knobs: `crosshair_label_box_overflow_policy`, `crosshair_time_label_box_overflow_policy`, and `crosshair_price_label_box_overflow_policy`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis overflow-policy behavior.
- Crosshair axis-label box per-axis horizontal-anchor parity baseline (`R-062`) with deterministic independent horizontal anchoring controls for time and price label boxes.
- New render-style knobs: `crosshair_label_box_horizontal_anchor`, `crosshair_time_label_box_horizontal_anchor`, and `crosshair_price_label_box_horizontal_anchor`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis horizontal-anchor behavior.
- Crosshair axis-label box per-axis vertical-anchor parity baseline (`R-061`) with deterministic independent vertical anchoring controls for time and price label boxes.
- New render-style knobs: `crosshair_label_box_vertical_anchor`, `crosshair_time_label_box_vertical_anchor`, and `crosshair_price_label_box_vertical_anchor`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis vertical-anchor behavior.
- Crosshair axis-label box per-axis text-alignment parity baseline (`R-060`) with deterministic independent alignment controls for time and price label boxes.
- New render-style knobs: `crosshair_label_box_text_h_align`, `crosshair_time_label_box_text_h_align`, and `crosshair_price_label_box_text_h_align`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis text-alignment behavior.
- Crosshair axis-label box per-axis min-width parity baseline (`R-059`) with deterministic independent minimum-width controls for time and price label boxes.
- New render-style knobs: `crosshair_label_box_min_width_px`, `crosshair_time_label_box_min_width_px`, and `crosshair_price_label_box_min_width_px`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis min-width behavior.
- Crosshair axis-label box per-axis fill-color parity baseline (`R-058`) with deterministic independent time/price box fill controls and shared fallback compatibility.
- New render-style knobs: `crosshair_time_label_box_color` and `crosshair_price_label_box_color`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis fill-color behavior.
- Crosshair axis-label box per-axis text-policy parity baseline (`R-057`) with deterministic independent manual text-color and auto-contrast controls for time and price label boxes.
- New render-style knobs: `crosshair_time_label_box_text_color`, `crosshair_price_label_box_text_color`, `crosshair_time_label_box_auto_text_contrast`, and `crosshair_price_label_box_auto_text_contrast`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis text-policy behavior.
- Crosshair axis-label box per-axis width-mode parity baseline (`R-056`) with deterministic independent `FitText`/`FullAxis` controls for time and price label boxes.
- New render-style knobs: `crosshair_time_label_box_width_mode` and `crosshair_price_label_box_width_mode` with fallback to shared `crosshair_label_box_width_mode`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis width-mode behavior.
- Crosshair axis-label box per-axis corner-radius parity baseline (`R-055`) with deterministic independent corner-radius controls for time and price label boxes.
- New render-style knobs: `crosshair_time_label_box_corner_radius_px` and `crosshair_price_label_box_corner_radius_px`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis corner-radius behavior.
- Crosshair axis-label box per-axis border-style parity baseline (`R-054`) with deterministic independent border color/width controls for time and price label boxes.
- New render-style knobs: `crosshair_time_label_box_border_color`, `crosshair_time_label_box_border_width_px`, `crosshair_price_label_box_border_color`, and `crosshair_price_label_box_border_width_px`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis border-style behavior.
- Crosshair axis-label box per-axis padding parity baseline (`R-053`) with deterministic independent X/Y padding controls for time and price label boxes.
- New render-style knobs: `crosshair_time_label_box_padding_x_px`, `crosshair_time_label_box_padding_y_px`, `crosshair_price_label_box_padding_x_px`, and `crosshair_price_label_box_padding_y_px`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box per-axis padding behavior.
- Crosshair axis-label font-size parity baseline (`R-052`) with deterministic dedicated font-size controls for time and price crosshair labels.
- New render-style knobs: `crosshair_time_label_font_size_px` and `crosshair_price_label_font_size_px`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label font-size behavior.
- Crosshair axis-label horizontal-inset parity baseline (`R-051`) with deterministic dedicated horizontal inset controls for time and price crosshair labels.
- New render-style knobs: `crosshair_time_label_padding_x_px` and `crosshair_price_label_padding_right_px`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label horizontal-inset behavior.
- Crosshair axis-label vertical-offset parity baseline (`R-050`) with deterministic dedicated Y-offset controls for time and price crosshair labels.
- New render-style knobs: `crosshair_time_label_offset_y_px` and `crosshair_price_label_offset_y_px`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label vertical-offset behavior.
- Crosshair axis-label box border-visibility parity baseline (`R-049`) with deterministic independent border toggles for time and price axis labels.
- New render-style knobs: `show_crosshair_time_label_box_border` and `show_crosshair_price_label_box_border`.
- New render-frame/property tests and criterion benchmark coverage for crosshair axis-label box border-visibility behavior.
- Crosshair axis-label box width-mode parity baseline (`R-048`) with deterministic `FitText`/`FullAxis` layout behavior on time/price axis panels.
- New render-style knob: `crosshair_label_box_width_mode` with `CrosshairLabelBoxWidthMode::{FitText, FullAxis}`.
- New render-frame/property tests and criterion benchmark coverage for crosshair axis-label box width-mode behavior.
- Crosshair axis-label box auto-contrast text parity baseline (`R-047`) with deterministic luminance-driven text color policy and manual text-color override.
- New render-style knobs: `crosshair_label_box_text_color` and `crosshair_label_box_auto_text_contrast`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box text-contrast behavior.
- Crosshair axis-label box border/radius parity baseline (`R-046`) with deterministic border width/color and corner-radius styling.
- New render-style knobs: `crosshair_label_box_border_width_px`, `crosshair_label_box_border_color`, and `crosshair_label_box_corner_radius_px`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label border/radius behavior.
- Crosshair axis-label box parity baseline (`R-045`) with deterministic fit-text background boxes for crosshair time/price axis labels.
- New render-style knobs: `crosshair_label_box_color`, `crosshair_label_box_padding_x_px`, `crosshair_label_box_padding_y_px`, `show_crosshair_time_label_box`, and `show_crosshair_price_label_box`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label box behavior.
- Crosshair axis-label parity baseline (`R-044`) with deterministic time/price crosshair labels projected into axis panels.
- New render-style knobs: `crosshair_time_label_color`, `crosshair_price_label_color`, `crosshair_axis_label_font_size_px`, `show_crosshair_time_label`, and `show_crosshair_price_label`.
- New render-frame/style/property tests and criterion benchmark coverage for crosshair axis-label render behavior.
- Crosshair guide-line parity baseline (`R-043`) with deterministic plot-pane horizontal/vertical crosshair line rendering.
- New render-style knobs: `crosshair_line_color`, `crosshair_line_width`, `show_crosshair_horizontal_line`, and `show_crosshair_vertical_line`.
- New render-frame/style tests and criterion benchmark coverage for crosshair guide-line render behavior.
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
- Major time-axis tick-mark style parity baseline (`R-037`) with deterministic dedicated color/width controls independent from regular time-axis tick marks.
- New render-style knobs: `major_time_tick_mark_color` and `major_time_tick_mark_width`.
- New render-frame/style tests and criterion benchmark coverage for major tick-mark style behavior.
- Major time-axis tick-mark length parity baseline (`R-038`) with deterministic dedicated length control independent from regular time-axis tick marks.
- New render-style knob: `major_time_tick_mark_length_px`.
- New render-frame/style tests and criterion benchmark coverage for major tick-mark length behavior.
- Major time-axis tick-mark visibility parity baseline (`R-039`) with deterministic show/hide behavior independent from regular time-axis tick marks.
- New render-style knob: `show_major_time_tick_marks`.
- New render-frame/style tests and criterion benchmark coverage for major tick-mark visibility behavior.
- Major time-axis label-offset parity baseline (`R-040`) with deterministic dedicated Y-offset control independent from regular time-axis labels.
- New render-style knob: `major_time_label_offset_y_px`.
- New render-frame/style tests and criterion benchmark coverage for major-label offset behavior.
- Time-axis border visibility parity baseline (`R-041`) with deterministic show/hide behavior independent from right-side price-axis border.
- New render-style knob: `show_time_axis_border`.
- New render-frame/style tests and criterion benchmark coverage for time-axis border visibility behavior.
- Price-axis border visibility parity baseline (`R-042`) with deterministic show/hide behavior independent from bottom time-axis border.
- New render-style knob: `show_price_axis_border`.
- New render-frame/style tests and criterion benchmark coverage for price-axis border visibility behavior.

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
