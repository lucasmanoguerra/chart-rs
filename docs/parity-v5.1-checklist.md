# Lightweight Charts v5.1 Parity Checklist

Use this document to track parity progress.

## Status Legend

- `not started`
- `in progress`
- `done`
- `blocked`

## Core

| ID | Area | Feature | Status | Acceptance Criteria | Test Evidence | Notes |
|---|---|---|---|---|---|---|
| C-001 | Time Scale | Logical-to-pixel mapping | in progress | Matches v5.1 behavior for visible range and spacing | `tests/core_scale_tests.rs`, `tests/property_scale_tests.rs` | Base time scale mapping implemented; parity tuning pending. |
| C-002 | Price Scale | Autoscale baseline | in progress | Stable autoscale with sparse/volatile data | `tests/core_scale_tests.rs`, `tests/property_scale_tests.rs`, `tests/api_smoke_tests.rs` | Base autoscale and inverted Y mapping implemented; advanced behavior pending. |
| C-003 | Series | Candlestick rendering basics | in progress | OHLC bars render with deterministic geometry | `tests/candlestick_tests.rs`, `tests/property_candlestick_tests.rs` | Base OHLC validation + deterministic candle geometry projection implemented. |
| C-004 | Interaction | Crosshair baseline | in progress | Pointer movement updates crosshair and labels deterministically | `tests/crosshair_tests.rs` | Base crosshair visibility + nearest-point/candle snapping implemented. |

## Extensions

| ID | Area | Feature | Status | Acceptance Criteria | Test Evidence | Notes |
|---|---|---|---|---|---|---|
| E-001 | Markers | Advanced marker placement | not started | Marker collision and alignment match expected rules | TBD | |
| E-002 | Plugins | Custom extension hooks | not started | Extension points allow bounded custom logic without core coupling | TBD | |
