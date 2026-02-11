use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, OhlcBar, PriceScaleTuning, TimeScaleTuning, Viewport};
use chart_rs::render::NullRenderer;

#[test]
fn fit_time_to_data_uses_mixed_sources() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 1.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_data(vec![DataPoint::new(10.0, 1.0), DataPoint::new(20.0, 2.0)]);
    engine.set_candles(vec![
        OhlcBar::new(30.0, 10.0, 20.0, 5.0, 15.0).expect("valid bar"),
    ]);

    let tuning = TimeScaleTuning {
        left_padding_ratio: 0.1,
        right_padding_ratio: 0.1,
        min_span_absolute: 1.0,
    };

    engine.fit_time_to_data(tuning).expect("fit time");

    let (full_start, full_end) = engine.time_full_range();
    assert_eq!(full_start, 10.0);
    assert_eq!(full_end, 30.0);

    let (visible_start, visible_end) = engine.time_visible_range();
    assert!((visible_start - 8.0).abs() <= 1e-9);
    assert!((visible_end - 32.0).abs() <= 1e-9);
}

#[test]
fn set_and_reset_time_visible_range() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_time_visible_range(20.0, 40.0)
        .expect("set visible");
    assert_eq!(engine.time_visible_range(), (20.0, 40.0));

    engine.reset_time_visible_range();
    assert_eq!(engine.time_visible_range(), (0.0, 100.0));
}

#[test]
fn autoscale_price_from_data_tuned_applies_padding() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 10.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_data(vec![DataPoint::new(1.0, 10.0), DataPoint::new(2.0, 20.0)]);

    let tuning = PriceScaleTuning {
        top_padding_ratio: 0.2,
        bottom_padding_ratio: 0.1,
        min_span_absolute: 0.000_001,
    };

    engine
        .autoscale_price_from_data_tuned(tuning)
        .expect("autoscale data");

    let (min, max) = engine.price_domain();
    assert!((min - 9.0).abs() <= 1e-9);
    assert!((max - 22.0).abs() <= 1e-9);
}

#[test]
fn autoscale_price_from_candles_tuned_applies_padding() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 10.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_candles(vec![
        OhlcBar::new(1.0, 100.0, 120.0, 90.0, 110.0).expect("valid bar"),
        OhlcBar::new(2.0, 110.0, 130.0, 100.0, 105.0).expect("valid bar"),
    ]);

    let tuning = PriceScaleTuning {
        top_padding_ratio: 0.1,
        bottom_padding_ratio: 0.1,
        min_span_absolute: 0.000_001,
    };

    engine
        .autoscale_price_from_candles_tuned(tuning)
        .expect("autoscale candles");

    let (min, max) = engine.price_domain();
    assert!((min - 86.0).abs() <= 1e-9);
    assert!((max - 134.0).abs() <= 1e-9);
}
