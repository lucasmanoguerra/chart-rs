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
  - split into focused submodules (for example `api::render_style`) to keep responsibilities narrow
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

- `api` builds a deterministic `RenderFrame` containing backend-agnostic line/rect/text primitives
- axis tick density is selected from viewport size and filtered with deterministic spacing rules
- time-axis labels are produced via policy+locale config with optional custom formatter injection
- time-axis formatter supports zoom-aware adaptive detail, fixed-offset timezone alignment, and optional session-boundary semantics
- time-axis major ticks (session boundaries/local-midnight) can be emphasized through deterministic style knobs
- price-axis labels support fixed/adaptive precision, min-move rounding, and normal/percent/indexed display modes via deterministic API config
- latest-price line/label marker can be rendered from newest point/candle sample with style toggles
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
- crosshair time/price axis-label color, font-size, and visibility are deterministic style-level controls resolved from interaction snap state
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
- plot and price-axis panels are styled through a deterministic render-style contract
- `render` backends execute primitives only (no scale math or interaction decisions)
- `cairo-backend` supports:
  - offscreen image-surface rendering
  - external cairo-context rendering (used by GTK `DrawingArea`)
- `platform_gtk` keeps widget lifecycle/event wiring isolated from render/domain code
