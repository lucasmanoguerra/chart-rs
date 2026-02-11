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
- `render` backends execute primitives only (no scale math or interaction decisions)
- `cairo-backend` supports:
  - offscreen image-surface rendering
  - external cairo-context rendering (used by GTK `DrawingArea`)
- `platform_gtk` keeps widget lifecycle/event wiring isolated from render/domain code
