# GTK4/Relm4 Crosshair Formatter Integration

This guide shows a practical integration pattern for context-aware crosshair
formatters when embedding `chart-rs` in GTK4/Relm4 apps.

## Goals

- Keep UI code deterministic and easy to reason about.
- Use context-aware crosshair formatters (`visible_span_abs`, `source_mode`).
- Avoid stale cache behavior during mode/range transitions.

## Minimal Engine Setup

```rust
use chart_rs::api::{
    ChartEngine, ChartEngineConfig, CrosshairLabelSourceMode, RenderStyle,
};
use chart_rs::core::Viewport;
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::NullRenderer;
use std::sync::Arc;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 86_400.0).with_price_domain(0.0, 200.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_render_style(RenderStyle {
            show_crosshair_time_label_box: false,
            show_crosshair_price_label_box: false,
            ..engine.render_style()
        })
        .expect("set style");

    // Context-aware time formatter:
    engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, context| {
        let source = match context.source_mode {
            CrosshairLabelSourceMode::SnappedData => "snap",
            CrosshairLabelSourceMode::PointerProjected => "ptr",
        };
        format!("t={value:.2} [{source}] span={:.1}s", context.visible_span_abs)
    }));

    // Context-aware price formatter:
    engine.set_crosshair_price_label_formatter_with_context(Arc::new(|value, context| {
        let source = match context.source_mode {
            CrosshairLabelSourceMode::SnappedData => "snap",
            CrosshairLabelSourceMode::PointerProjected => "ptr",
        };
        format!("p={value:.2} [{source}] span={:.1}s", context.visible_span_abs)
    }));

    engine
}
```

## GTK4 DrawingArea Wiring

Use pointer motion to update crosshair state, then queue redraw.

```rust
// inside GTK setup:
// drawing_area.add_controller(motion_controller);
// motion_controller.connect_motion(...)

motion_controller.connect_motion(move |_, x, y| {
    let mut engine = engine_rc.borrow_mut();
    engine.pointer_move(x, y);
    drawing_area.queue_draw();
});
```

If you support pointer leave:

```rust
motion_controller.connect_leave(move |_| {
    let mut engine = engine_rc.borrow_mut();
    engine.pointer_leave();
    drawing_area.queue_draw();
});
```

## Relm4 Message Pattern

A practical Relm4 shape is to convert UI events to explicit messages:

- `PointerMoved { x, y }`
- `PointerLeft`
- `CrosshairModeChanged(CrosshairMode)`
- `VisibleRangeChanged { start, end }`

Each update mutates `ChartEngine`, then triggers a redraw.

## Lifecycle Notes

- `set_crosshair_mode(...)` clears context-aware crosshair caches as part of
  lifecycle invalidation.
- Time-range changes (`set_time_visible_range`, pan/zoom/fit/reset) clear
  context-aware crosshair caches through visible-range change flow.
- You can inspect formatter state from host code:
  - `crosshair_time_label_formatter_override_mode()`
  - `crosshair_price_label_formatter_override_mode()`
  - `crosshair_label_formatter_generations()`

## Diagnostics Bridge Hooks (GTK Adapter)

`GtkChartAdapter` can publish diagnostics and versioned snapshot payloads each
draw pass, allowing Relm4 models or debug panels to consume stable contracts.

```rust
use chart_rs::platform_gtk::GtkChartAdapter;

adapter.set_crosshair_diagnostics_hook(|diagnostics| {
    // forward to telemetry/debug panel
    eprintln!("diag={diagnostics:?}");
});

adapter.set_snapshot_json_hook(7.0, |snapshot_json| {
    // persist / compare against fixtures
    eprintln!("snapshot-json-bytes={}", snapshot_json.len());
});
```

You can also pull one-shot payloads:

- `crosshair_formatter_diagnostics_json_contract_v1_pretty()`
- `snapshot_json_contract_v1_pretty(body_width_px)`

## Recommended Host-Side Rules

- Prefer context-aware formatters for user-facing axis labels.
- Use formatter-generation and override-mode introspection in debug panels.
- Keep formatter closures pure and side-effect free.
- If you swap business formatting profiles, do so through explicit setter calls
  instead of hidden global state.
