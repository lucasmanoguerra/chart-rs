# Crosshair Formatter Contract Matrix

This matrix defines how per-axis crosshair formatter APIs interact.

## Time Axis

| Action | Resulting Mode | Legacy Formatter Stored | Context Formatter Stored | Generation |
|---|---|---|---|---|
| initial state | `None` | no | no | unchanged |
| `set_crosshair_time_label_formatter(...)` | `Legacy` | yes | no (cleared) | `+1` |
| `set_crosshair_time_label_formatter_with_context(...)` | `Context` | no (cleared) | yes | `+1` |
| `clear_crosshair_time_label_formatter()` | `None` | no | no (cleared) | `+1` |
| `clear_crosshair_time_label_formatter_with_context()` | `None` | unchanged | no | `+1` |

## Price Axis

| Action | Resulting Mode | Legacy Formatter Stored | Context Formatter Stored | Generation |
|---|---|---|---|---|
| initial state | `None` | no | no | unchanged |
| `set_crosshair_price_label_formatter(...)` | `Legacy` | yes | no (cleared) | `+1` |
| `set_crosshair_price_label_formatter_with_context(...)` | `Context` | no (cleared) | yes | `+1` |
| `clear_crosshair_price_label_formatter()` | `None` | no | no (cleared) | `+1` |
| `clear_crosshair_price_label_formatter_with_context()` | `None` | unchanged | no | `+1` |

## Shared Lifecycle Rules

- Context-aware crosshair caches are invalidated on:
  - `set_crosshair_mode(...)`
  - visible-range transitions (`set/reset/pan/zoom/fit` time range flow)
- Context-aware cache keys partition by:
  - formatter generation
  - source mode (`SnappedData` / `PointerProjected`)
  - quantized visible span
  - quantized axis input values
- Snapshot/export contract mirrors API state:
  - `EngineSnapshot.crosshair_formatter.*`
  - `crosshair_formatter_diagnostics()`
- JSON export contracts:
  - raw payloads: `snapshot_json_pretty(...)`, `crosshair_formatter_diagnostics_json_pretty()`
  - versioned payloads: `snapshot_json_contract_v1_pretty(...)`, `crosshair_formatter_diagnostics_json_contract_v1_pretty()`
  - backward-compatible parse helpers: `EngineSnapshot::from_json_compat_str(...)`, `CrosshairFormatterDiagnostics::from_json_compat_str(...)`
