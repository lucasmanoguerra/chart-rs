# Examples

Ejemplos gráficos e interactivos para `chart-rs` usando:

- GTK4 (`platform_gtk`)
- Cairo (`cairo-backend`)
- Datos reales de Binance (`/api/v3/klines`)

## Requisitos

- Dependencias del sistema para GTK4/Cairo/Pango instaladas.
- Acceso a internet (para consultar Binance REST API).

## Ejecutar un ejemplo

Todos los ejemplos de esta carpeta se ejecutan con:

```bash
cargo run --features desktop --example <nombre_ejemplo>
```

Ejemplo:

```bash
cargo run --features desktop --example gtk_binance_basic
```

## Lista de ejemplos

- `gtk_binance_basic`
  - Carga `BTCUSDT` (1m), render básico interactivo.
- `gtk_binance_live_poll`
  - Carga inicial y actualización periódica (polling) cada 3s.
- `gtk_binance_multi_pane_volume`
  - Vista en múltiples panes: velas + volumen.
- `gtk_binance_crosshair_modes`
  - Cambio de modo de crosshair con teclado (`M`, `N`, `H`).
- `gtk_binance_price_scale_modes`
  - Cambio de escala de precio con teclado (`1`, `2`, `3`).
- `gtk_binance_axis_interactions`
  - Drag en ejes para escalar y doble click para reset.
- `gtk_binance_symbol_switch`
  - Selector de símbolo (`BTCUSDT`, `ETHUSDT`, `BNBUSDT`, `SOLUSDT`) en caliente.
- `gtk_binance_logical_inspector`
  - Inspección del filled logical slot más cercano bajo el cursor.

## Interacciones base (instaladas en los ejemplos)

- Movimiento de mouse: crosshair.
- Drag en zona de plot: pan horizontal.
- Wheel vertical: zoom temporal.
- Wheel horizontal: pan temporal.
- Drag en eje de precio: zoom de precio.
- Drag en eje de tiempo: zoom temporal.

## Notas

- Código compartido de Binance/GTK: `examples/shared/binance.rs`.
- Si Binance responde con error o rate-limit, el ejemplo muestra el error en UI o stderr.
