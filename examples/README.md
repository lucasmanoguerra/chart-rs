# Examples

Manual runnable examples for local parity checks and behavior exploration.

## Run

- `cargo run --example crosshair_formatter_lifecycle`
- `cargo run --example snapshot_diagnostics_contract`
- `cargo run --example time_scale_interactions`
- `cargo run --example candlestick_projection_snapshot`
- `cargo run --example gtk_interaction_lab --features desktop`
- `cargo run --example gtk_projection_gallery --features desktop`
- `cargo run --example gtk_live_data_workbench --features desktop`
- `cargo run --example gtk_tradingview_binance --features desktop`
- `cargo run --example gtk_binance_axis_lab --features desktop`
- `cargo run --example gtk_binance_projection_probe --features desktop`
- `cargo run --example gtk_binance_contracts_lab --features desktop`
- `cargo run --example gtk_binance_replay_workbench --features desktop`
- `cargo run --example gtk_binance_scale_window_lab --features desktop`
- `cargo run --example gtk_binance_defaults_normal_live --features desktop`
- `cargo run --example gtk_binance_axis_scale_interactions --features desktop`

## Scope

- `crosshair_formatter_lifecycle`: formatter mode transitions and context-aware behavior.
- `snapshot_diagnostics_contract`: coherence checks between snapshot export and diagnostics APIs, including v1 JSON contracts.
- `time_scale_interactions`: pan/zoom interaction flow over visible time range, incluyendo `scroll_time_to_realtime`, APIs de `scroll_position`, margen derecho en píxeles (`right_offset_px`), límites de spacing (`min/max bar spacing`), pan táctil (`touch_drag_pan_time_visible`) y anclaje right-edge en zoom (`right_bar_stays_on_scroll`).
- `candlestick_projection_snapshot`: candle projection and render-frame primitive inspection.
- `gtk_interaction_lab`: entorno UI GTK para validar crosshair, pan/zoom, modos Magnet/Normal, Linear/Log, `invertScale`, `scaleMargins`, restricciones `fix_left_edge`/`fix_right_edge`, gates de interacción globales y granulares (`handle_*`, wheel/drag/pinch), navegación temporal (`right_offset_bars`/`bar_spacing_px`), políticas de resize (`lock + anchor`) y diagnostics hooks; inicia con crosshair `Normal` vía `ChartEngineConfig`.
- `gtk_projection_gallery`: galería UI para inspección visual de `project_*` (line/area/baseline/histogram/bar/candles/markers) con overscan.
- `gtk_live_data_workbench`: flujo UI con feed en vivo (`set_*`/`append_*`/`update_*`), seguimiento de rango con política realtime append (`preserve_right_edge_on_append`/tolerancia), autoscale de precio realtime (`autoscale_on_data_set`/`autoscale_on_data_update`), plugin events y snapshot/diagnostics hooks.
- `gtk_tradingview_binance`: ejemplo UI tipo plataforma de trading con velas/volumen, carga real de Binance Spot Klines, controles de escala/display/rango, autoscale realtime en carga+updates y interacción estilo TradingView.
- `gtk_binance_axis_lab`: laboratorio de ejes con Binance real para probar `TimeAxisLabelConfig`/`PriceAxisLabelConfig`, timezone/session, custom formatters y ciclos de cache.
- `gtk_binance_projection_probe`: inspección en vivo de `project_visible_*` con datos reales (candles/bars/line/area/baseline/histogram/markers).
- `gtk_binance_contracts_lab`: validación manual de contratos JSON snapshot/diagnostics v1 y lifecycle de formatters crosshair (none/legacy/context).
- `gtk_binance_replay_workbench`: replay de mercado real con seed parcial y flujo `set_*`/`append_*`, útil para validar lifecycle de ingestión incremental, follow-tail y autoscale de precio en realtime.
- `gtk_binance_scale_window_lab`: pruebas de escala/viewport con datos reales (`visible_*`, overscan, pan/zoom y round-trip `pixel <-> time/price`).
- `gtk_binance_defaults_normal_live`: ejemplo base con opciones por default del engine, crosshair en modo `Normal`, carga histórica de Binance vía `uiKlines` y actualización realtime por WebSocket de klines/candles (`update_candle`).
- `gtk_binance_axis_scale_interactions`: vista compacta tipo TradingView (gris/monocromo) con `Candlestick` + Binance `uiKlines` para validar interacciones por zona: drag/scroll en eje de precio (escala vertical), drag/scroll en eje de tiempo (zoom horizontal) y doble click en eje de precio (`axisDoubleClickReset`).
