use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, TimeScaleTuning, Viewport};
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::NullRenderer;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(960, 540), 0.0, 100.0).with_price_domain(0.0, 400.0);
    let mut engine = ChartEngine::new(renderer, config)?;

    let points: Vec<DataPoint> = (0..120)
        .map(|i| {
            let x = i as f64;
            let y = 150.0 + (x / 6.0).sin() * 20.0 + x * 0.4;
            DataPoint::new(x, y)
        })
        .collect();
    engine.set_data(points);
    engine.fit_time_to_data(TimeScaleTuning::default())?;

    engine.set_crosshair_time_label_formatter(Arc::new(|value| format!("T-L:{value:.2}")));
    engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, context| {
        format!("T-C:{value:.2}|span:{:.1}", context.visible_span_abs)
    }));
    engine.set_crosshair_price_label_formatter_with_context(Arc::new(|value, context| {
        format!("P-C:{value:.2}|src:{:?}", context.source_mode)
    }));

    engine.set_crosshair_mode(CrosshairMode::Normal);
    engine.pointer_move(420.0, 210.0);
    let frame = engine.build_render_frame()?;

    println!(
        "render frame: lines={} rects={} texts={}",
        frame.lines.len(),
        frame.rects.len(),
        frame.texts.len()
    );
    println!(
        "snapshot crosshair formatter: {:?}",
        engine.snapshot(7.0)?.crosshair_formatter
    );
    println!(
        "diagnostics: {:?}",
        engine.crosshair_formatter_diagnostics()
    );

    Ok(())
}
