use chart_rs::api::{ChartEngine, ChartEngineConfig, TimeScaleZoomLimitBehavior};
use chart_rs::core::{DataPoint, TimeScaleTuning, Viewport};
use chart_rs::render::NullRenderer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 60.0).with_price_domain(0.0, 300.0);
    let mut engine = ChartEngine::new(renderer, config)?;

    let points: Vec<DataPoint> = (0..300)
        .map(|i| {
            let x = i as f64;
            let y = 100.0 + (x / 12.0).sin() * 15.0 + x * 0.2;
            DataPoint::new(x, y)
        })
        .collect();
    engine.set_data(points);
    engine.fit_time_to_data(TimeScaleTuning::default())?;
    engine.set_time_scale_zoom_limit_behavior(TimeScaleZoomLimitBehavior {
        min_bar_spacing_px: 8.0,
        max_bar_spacing_px: Some(48.0),
    })?;

    let before = engine.time_visible_range();
    let pan_dt = engine.wheel_pan_time_visible(240.0, 0.15)?;
    let touch_pan_dt = engine.touch_drag_pan_time_visible(80.0, 10.0)?;
    let zoom_factor = engine.wheel_zoom_time_visible(-120.0, 640.0, 0.2, 0.5)?;
    let after = engine.time_visible_range();

    println!("visible range before: {:?}", before);
    println!("pan delta (time units): {:.4}", pan_dt);
    println!("touch pan delta (time units): {:.4}", touch_pan_dt);
    println!("zoom factor applied: {:.6}", zoom_factor);
    println!("zoom limits: {:?}", engine.time_scale_zoom_limit_behavior());
    println!("visible range after: {:?}", after);

    Ok(())
}
