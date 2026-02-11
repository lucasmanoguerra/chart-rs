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
| C-001 | Time Scale | Logical-to-pixel mapping | not started | Matches v5.1 behavior for visible range and spacing | TBD | |
| C-002 | Price Scale | Autoscale baseline | not started | Stable autoscale with sparse/volatile data | TBD | |
| C-003 | Series | Candlestick rendering basics | not started | OHLC bars render with deterministic geometry | TBD | |
| C-004 | Interaction | Crosshair baseline | not started | Pointer movement updates crosshair and labels deterministically | TBD | |

## Extensions

| ID | Area | Feature | Status | Acceptance Criteria | Test Evidence | Notes |
|---|---|---|---|---|---|---|
| E-001 | Markers | Advanced marker placement | not started | Marker collision and alignment match expected rules | TBD | |
| E-002 | Plugins | Custom extension hooks | not started | Extension points allow bounded custom logic without core coupling | TBD | |
