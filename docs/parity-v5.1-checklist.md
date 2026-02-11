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
| C-001 | Time Scale | Logical-to-pixel mapping | done | Matches v5.1 behavior for visible range and spacing | `tests/core_scale_tests.rs`, `tests/property_scale_tests.rs`, `tests/api_tuning_tests.rs` | Full/visible range controls and fit-to-data tuning are implemented. |
| C-002 | Price Scale | Autoscale baseline | done | Stable autoscale with sparse/volatile data | `tests/core_scale_tests.rs`, `tests/property_scale_tests.rs`, `tests/api_tuning_tests.rs`, `tests/api_smoke_tests.rs` | Tuned autoscale for points/candles with padding controls is implemented. |
| C-003 | Series | Candlestick rendering basics | done | OHLC bars render with deterministic geometry | `tests/candlestick_tests.rs`, `tests/candlestick_visible_tests.rs`, `tests/property_candlestick_tests.rs` | OHLC validation, deterministic projection, and visible-range candle projection are implemented. |
| C-004 | Interaction | Crosshair baseline | done | Pointer movement updates crosshair and labels deterministically | `tests/crosshair_tests.rs`, `tests/property_api_tests.rs` | Crosshair now exposes deterministic snapped pixel + logical labels (time/price) for points/candles. |
| C-005 | Interaction | Visible range pan/zoom baseline | done | Drag and zoom operations move visible range deterministically around anchor points | `tests/interaction_pan_zoom_tests.rs`, `tests/property_api_tests.rs` | Time scale pan/zoom APIs are implemented for both logical-time and pixel anchors. |
| C-006 | Series | Line series projection baseline | done | Line data maps to deterministic segment geometry over active scales | `tests/line_series_tests.rs`, `tests/property_line_series_tests.rs` | Core and API line-segment projection helpers are implemented. |
| C-007 | Series | Visible data window selection | done | Point/candle series can be filtered by visible logical range with deterministic ordering | `tests/visible_data_window_tests.rs`, `tests/property_api_tests.rs` | Core windowing helpers and API visible-window methods (with overscan) are implemented. |
| C-008 | Series | Area series projection baseline | done | Area data maps to deterministic line/fill geometry over active scales | `tests/area_series_tests.rs`, `tests/property_area_series_tests.rs` | Core and API area geometry projection (visible + overscan variants) are implemented with explicit baseline-closed polygons. |
| C-009 | Series | Baseline series projection baseline | done | Baseline data maps to deterministic line + above/below fill geometry over active scales | `tests/baseline_series_tests.rs`, `tests/property_baseline_series_tests.rs` | Core and API baseline geometry projection (visible + overscan variants) are implemented with explicit baseline-closed polygons and clamped split regions. |
| C-010 | Series | Histogram series projection baseline | done | Histogram values map to deterministic bar geometry anchored to a baseline | `tests/histogram_series_tests.rs`, `tests/property_histogram_series_tests.rs` | Core and API histogram projection (visible + overscan variants) are implemented with explicit bar width validation and baseline anchoring. |
| C-011 | Series | OHLC bar series projection baseline | done | OHLC bars map to deterministic stem/tick geometry over active scales | `tests/bar_series_tests.rs`, `tests/property_bar_series_tests.rs` | Core and API bar projection (visible + overscan variants) are implemented with explicit tick width validation and OHLC ordering invariants. |

## Extensions

| ID | Area | Feature | Status | Acceptance Criteria | Test Evidence | Notes |
|---|---|---|---|---|---|---|
| E-001 | Markers | Advanced marker placement | done | Marker collision and alignment match expected rules | `tests/markers_tests.rs`, `tests/property_markers_tests.rs` | Deterministic marker placement with lane-based collision avoidance and visible-window projection is implemented. |
| E-002 | Plugins | Custom extension hooks | done | Extension points allow bounded custom logic without core coupling | `tests/plugins_tests.rs` | Plugin hooks with deterministic event dispatch and read-only engine context are implemented in the `extensions` layer. |
