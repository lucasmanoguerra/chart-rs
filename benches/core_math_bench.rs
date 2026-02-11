use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{
    DataPoint, LinearScale, OhlcBar, PriceScale, TimeScale, Viewport, points_in_time_window,
    project_candles, project_line_segments,
};
use chart_rs::render::NullRenderer;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_linear_scale_round_trip(c: &mut Criterion) {
    let viewport = Viewport::new(1920, 1080);
    let scale = LinearScale::new(0.0, 10_000.0).expect("valid scale");

    c.bench_function("linear_scale_round_trip", |b| {
        b.iter(|| {
            let px = scale
                .domain_to_pixel(4_321.123, viewport)
                .expect("to pixel");
            let _ = scale.pixel_to_domain(px, viewport).expect("from pixel");
        })
    });
}

fn bench_candle_projection_10k(c: &mut Criterion) {
    let viewport = Viewport::new(1920, 1080);
    let time_scale = TimeScale::new(0.0, 10_001.0).expect("valid time scale");
    let price_scale = PriceScale::new(0.0, 2_500.0).expect("valid price scale");

    let bars: Vec<OhlcBar> = (0..10_000)
        .map(|i| {
            let t = i as f64;
            let base = 100.0 + t * 0.05;
            let open = base;
            let close = if i % 2 == 0 { base + 1.0 } else { base - 1.0 };
            let low = open.min(close) - 0.75;
            let high = open.max(close) + 0.75;
            OhlcBar::new(t, open, high, low, close).expect("valid generated bar")
        })
        .collect();

    c.bench_function("candle_projection_10k", |b| {
        b.iter(|| {
            let _ = project_candles(
                black_box(&bars),
                black_box(time_scale),
                black_box(price_scale),
                black_box(viewport),
                black_box(7.0),
            )
            .expect("projection should succeed");
        })
    });
}

fn bench_line_projection_20k(c: &mut Criterion) {
    let viewport = Viewport::new(1920, 1080);
    let time_scale = TimeScale::new(0.0, 20_001.0).expect("valid time scale");
    let price_scale = PriceScale::new(0.0, 5_000.0).expect("valid price scale");

    let points: Vec<DataPoint> = (0..20_000)
        .map(|i| {
            let t = i as f64;
            let y = 1_000.0 + (t * 0.07).sin() * 250.0 + t * 0.02;
            DataPoint::new(t, y)
        })
        .collect();

    c.bench_function("line_projection_20k", |b| {
        b.iter(|| {
            let _ = project_line_segments(
                black_box(&points),
                black_box(time_scale),
                black_box(price_scale),
                black_box(viewport),
            )
            .expect("line projection should succeed");
        })
    });
}

fn bench_visible_window_points_100k(c: &mut Criterion) {
    let points: Vec<DataPoint> = (0..100_000)
        .map(|i| {
            let x = i as f64;
            let y = (x * 0.02).sin() * 100.0 + x * 0.001;
            DataPoint::new(x, y)
        })
        .collect();

    c.bench_function("visible_window_points_100k", |b| {
        b.iter(|| {
            let _ =
                points_in_time_window(black_box(&points), black_box(45_000.0), black_box(55_000.0));
        })
    });
}

fn bench_engine_snapshot_json_2k(c: &mut Criterion) {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(1600, 900), 0.0, 2_001.0)
        .with_price_domain(0.0, 2_500.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let bars: Vec<OhlcBar> = (0..2_000)
        .map(|i| {
            let t = i as f64;
            let base = 400.0 + t * 0.03;
            let open = base;
            let close = if i % 2 == 0 { base + 2.0 } else { base - 2.0 };
            let low = open.min(close) - 1.0;
            let high = open.max(close) + 1.0;
            OhlcBar::new(t, open, high, low, close).expect("valid generated bar")
        })
        .collect();

    engine.set_series_metadata("series-id", "candles-main");
    engine.set_series_metadata("series-type", "candlestick");
    engine.set_candles(bars);

    c.bench_function("engine_snapshot_json_2k", |b| {
        b.iter(|| {
            let _ = engine
                .snapshot_json_pretty(black_box(7.0))
                .expect("snapshot json should succeed");
        })
    });
}

criterion_group!(
    benches,
    bench_linear_scale_round_trip,
    bench_candle_projection_10k,
    bench_line_projection_20k,
    bench_visible_window_points_100k,
    bench_engine_snapshot_json_2k
);
criterion_main!(benches);
