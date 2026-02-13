# Lightweight Charts v5.1 Axis Cadence Reference

This note captures the directed parity references used for axis tick/label
cadence validation against Lightweight Charts v5.1 behavior.

## Reference Inputs

- Official project:
  - https://github.com/tradingview/lightweight-charts
- Local manual visual baseline:
  - `reference_UI/Captura desde 2026-02-12 20-28-20.png`

## Target Behavioral Contract

- time axis:
  - zoom-out should produce sparser tick/label cadence
  - intermediate zoom should increase cadence versus zoom-out
  - zoom-in should permit denser tick/label cadence
  - labels must remain collision-safe under min-spacing filtering
- price axis:
  - vertical scale zoom-out should reduce label cadence
  - intermediate scale zoom should remain non-decreasing toward zoom-in cadence
  - vertical scale zoom-in should increase label cadence
  - labels must remain collision-safe under min-spacing filtering

## Automated Evidence

- Directed comparison tests:
  - `tests/lightweight_axis_tick_cadence_reference_tests.rs`
    - `lightweight_v51_reference_time_axis_tick_cadence_tracks_intermediate_zoom_windows`
    - `lightweight_v51_reference_price_axis_tick_cadence_tracks_multi_step_scale_zoom`
- Core spacing/collision guards:
  - `tests/render_axis_layout_tests.rs`
    - `major_time_labels_are_retained_and_collision_safe_under_mixed_zoom_density`
- Visual-fixture zoom extremes:
  - `tests/fixtures/axis_section_sizing/axis_section_sizing_corpus.json`
    - `zoom-extreme-axis-density-time-out`
    - `zoom-extreme-axis-density-time-in`
    - `zoom-extreme-axis-density-price-out`
    - `zoom-extreme-axis-density-price-in`

## Review Guidance

When cadence logic changes, review both:

1. test cadence progression outcomes:
   - time: `zoom-out < mid1 <= mid2 <= in1 <= in2`
   - price: `zoom-out-2 <= zoom-out-1 < baseline < zoom-in-1 <= zoom-in-2`
2. verify major labels are not dropped by minor-label collisions in mixed zoom-density windows
3. fixture PNG updates in `tests/fixtures/axis_section_sizing/reference_png/`
