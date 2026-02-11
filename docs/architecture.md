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
- property tests with `proptest`
- benchmark updates with `criterion` for performance-sensitive paths

## Scale Strategy

- `TimeScale`
  - tracks full range and visible range
  - supports fit-to-data with configurable left/right padding
- `PriceScale`
  - supports tuned autoscale from points or candles
  - supports `PriceScaleMode::Linear` and `PriceScaleMode::Log`
  - log mode validates strictly-positive domains and applies tuning in transformed log space
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
  - pointer move sets visible state and updates coordinates
  - nearest snap candidate is selected from points/candles
  - pointer leave hides crosshair and clears snap state

## Render Strategy

- `api` builds a deterministic `RenderFrame` containing backend-agnostic line/text primitives
- axis tick density is selected from viewport size and filtered with deterministic spacing rules
- time-axis labels are produced via policy+locale config with optional custom formatter injection
- time-axis formatter supports zoom-aware adaptive detail, fixed-offset timezone alignment, and optional session-boundary semantics
- time-axis major ticks (session boundaries/local-midnight) can be emphasized through deterministic style knobs
- price-axis labels support fixed/adaptive precision, min-move rounding, and normal/percent/indexed display modes via deterministic API config
- latest-price line/label marker can be rendered from newest point/candle sample with style toggles
- price-axis label selection can exclude deterministic overlap zones around the last-price marker
- last-price marker can optionally resolve deterministic up/down/neutral colors from latest vs previous sample
- last-price marker source policy can switch between full-series latest sample and newest visible-range sample
- in-engine price-label caching reuses deterministic label text across repeated redraws
- in-engine time-label caching keeps redraw behavior deterministic under all formatter policies
- plot and price-axis panels are styled through a deterministic render-style contract
- `render` backends execute primitives only (no scale math or interaction decisions)
- `cairo-backend` supports:
  - offscreen image-surface rendering
  - external cairo-context rendering (used by GTK `DrawingArea`)
- `platform_gtk` keeps widget lifecycle/event wiring isolated from render/domain code
