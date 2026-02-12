use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{OhlcBar, Viewport};
use chart_rs::render::NullRenderer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 600), 0.0, 50.0).with_price_domain(90.0, 220.0);
    let mut engine = ChartEngine::new(renderer, config)?;

    let mut candles = Vec::new();
    for i in 0..50 {
        let time = i as f64;
        let open = 120.0 + (time / 8.0).sin() * 8.0 + time * 0.15;
        let close = open + if i % 2 == 0 { 2.4 } else { -1.7 };
        let high = open.max(close) + 1.2;
        let low = open.min(close) - 1.0;
        candles.push(OhlcBar::new(time, open, high, low, close)?);
    }

    engine.set_candles(candles);
    engine.pointer_move(500.0, 280.0);

    let snapshot = engine.snapshot(8.0)?;
    let frame = engine.build_render_frame()?;

    println!("candle geometry count: {}", snapshot.candle_geometry.len());
    println!("snapshot price domain: {:?}", snapshot.price_domain);
    println!(
        "frame primitives: lines={} rects={} texts={}",
        frame.lines.len(),
        frame.rects.len(),
        frame.texts.len()
    );

    Ok(())
}
