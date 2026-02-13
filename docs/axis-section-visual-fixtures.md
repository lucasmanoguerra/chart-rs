# Axis-Section Visual Fixture Corpus

This document describes the lightweight visual-fixture corpus used to monitor
axis-section sizing drift.

## Scope

The corpus targets layout behavior around:

- `plot_right` and price-axis section width
- `plot_bottom` and time-axis section height
- axis-label density side effects under typography pressure

It is intentionally deterministic and backend-independent: fixtures validate
render-frame layout signatures rather than pixel-level image diffs.

## Fixture Files

- Input + expected signatures:
  - `tests/fixtures/axis_section_sizing/axis_section_sizing_corpus.json`
- Regression test runner:
  - `tests/visual_fixture_axis_section_sizing_tests.rs`
- Artifact path validation:
  - `tests/visual_fixture_axis_section_png_artifacts_tests.rs`
- PNG generator:
  - `src/bin/generate_axis_section_fixture_pngs.rs`

Each fixture defines:

- viewport, time range, and price domain
- optional visible-range override for zoomed cadence scenarios:
  - `time_visible_range_override`
- optional autoscale control for deterministic price-domain stress:
  - `disable_autoscale_on_data_set`
- optional price-axis interaction replay for cadence setup:
  - `price_axis_scale_steps` (`axis_drag_scale_price` tuples)
- deterministic data samples
- style/config overrides that create layout pressure scenarios
- expected signature fields:
  - `plot_right_px`
  - `plot_bottom_px`
  - `price_axis_width_px`
  - `time_axis_height_px`
  - `price_label_count`
  - `time_label_count`
  - `major_time_label_count`
  - optional locale/timezone sentinels:
    - `leftmost_time_label_text`
    - `top_price_label_text`
    - `major_time_tick_mark_count`
- optional artifact fields:
  - `artifacts.reference_png_relpath`
  - `price_axis_display_base_override` (`zero`/`nan`/`pos_inf`/`neg_inf`) for fixture-only non-finite display-base literals

Current scenario groups include:

- baseline and typography-pressure cases
- tiny viewport clamp stress
- narrow-domain high-precision price-label stress
- sparse-series wide-range stress
- extreme positive/negative magnitude stress
- mixed price display-mode stress (`Normal`, `Percentage`, `IndexedTo100`)
- locale/timezone/session formatting stress (`EnUs`/`EsEs`, UTC/fixed-offset, session vs no-session)
- day-rollover + offset-extreme stress (`UTC+14`/`UTC-12`) with wrap-around session boundaries
- second-level sub-minute stress (`show_seconds=true`) with/without session boundaries
- `UtcAdaptive` threshold-cutover stress (`~600s`, `~172800s`) for second/minute/date pattern transitions
- `LogicalDecimal` high-precision stress (`precision=10`) across `EnUs`/`EsEs`
- `MinMove` trim stress (`trim_trailing_zeros=true/false`) across `EnUs`/`EsEs`
- display-mode base fallback stress (`Percentage`/`IndexedTo100` with `base_price=None`)
- display-mode explicit invalid-base fallback stress (`Percentage`/`IndexedTo100` with `base_price=0`, `NaN`, `+inf`, `-inf`)
- extreme zoom cadence stress for axis density:
  - `zoom-extreme-axis-density-time-out`
  - `zoom-extreme-axis-density-time-in`
  - `zoom-extreme-axis-density-price-out`
  - `zoom-extreme-axis-density-price-in`
- mixed zoom-density collision-priority stress for major/minor time labels:
  - `time-axis-pressure` (signature/PNG updates track intentional major-label retention changes)

## PNG References

Generate/refresh PNG references declared in fixture manifests:

```bash
cargo run --features cairo-backend --bin generate_axis_section_fixture_pngs
```

Generated artifacts are committed under:

- `tests/fixtures/axis_section_sizing/reference_png/`

## Manual UI Baseline

In addition to fixture signatures and generated PNGs, keep one manual baseline
capture under:

- `reference_UI/Captura desde 2026-02-12 20-28-20.png`

This baseline is used for PR-level visual sanity checks around:

- plot/time/price section proportion
- right-axis label density and clipping
- time-axis label/tick readability
- overall chart spacing behavior vs the expected GTK host look

When render/layout-related code changes, complete the standardized checklist
below.

### Universal Steps

1. regenerate fixture PNGs (`cargo run --features cairo-backend --bin generate_axis_section_fixture_pngs`)
2. compare changed fixture PNGs with the manual baseline capture
3. document deliberate visual deviations in PR notes

### Change-Type Checklists

#### Layout Changes (axis sections, panel sizing, clamp logic)

- verify plot/time/price section proportions remain balanced in narrow and wide viewports
- verify right-axis and time-axis labels do not overlap or clip at section boundaries
- verify tiny-viewport scenarios preserve non-negative plot space and readable labels
- verify fixture signature fields (`plot_right_px`, `plot_bottom_px`, section sizes, label counts) changed only when intentional

#### Render Changes (primitives, style toggles, visibility policies)

- verify label/tick/grid/border visibility toggles match expected on/off behavior
- verify no unintended drift in stroke widths/colors and label anchoring
- verify last-price and crosshair visual elements stay inside intended sections
- verify changed PNGs remain consistent with manual baseline chart spacing and readability

#### Formatter Changes (locale, display mode, fallback paths)

- verify locale separators are preserved (`EnUs` decimal point, `EsEs` decimal comma)
- verify display-mode suffix contract is preserved (`%` only for `Percentage`)
- verify fallback edge-cases (`base_price=None`, invalid base, no-data domain fallback) remain visually stable
- verify textual sentinels (`leftmost_time_label_text`, `top_price_label_text`) changed only when intentional

## Validation

Run:

```bash
cargo test --all-features --test visual_fixture_axis_section_sizing_tests
cargo test --all-features --test visual_fixture_axis_section_png_artifacts_tests
```

If adaptive sizing behavior changes intentionally, update expected signatures in
`tests/fixtures/axis_section_sizing/axis_section_sizing_corpus.json` together
with parity/docs evidence.
