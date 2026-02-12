use chart_rs::api::{ChartEngine, ChartEngineConfig, PriceScaleRealtimeBehavior};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

fn seed_points() -> Vec<DataPoint> {
    vec![DataPoint::new(0.0, 10.0), DataPoint::new(1.0, 20.0)]
}

fn seed_candles() -> Vec<OhlcBar> {
    vec![OhlcBar::new(0.0, 45.0, 60.0, 40.0, 55.0).expect("candle")]
}

#[test]
fn realtime_price_behavior_defaults_to_enabled() {
    let engine = build_engine();
    assert!(engine.price_scale_realtime_behavior().autoscale_on_data_set);
    assert!(
        engine
            .price_scale_realtime_behavior()
            .autoscale_on_data_update
    );
    assert!(
        engine
            .price_scale_realtime_behavior()
            .autoscale_on_time_range_change
    );
}

#[test]
fn append_point_does_not_autoscale_when_disabled() {
    let mut engine = build_engine();
    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: false,
        autoscale_on_data_update: false,
        autoscale_on_time_range_change: false,
    });
    engine.set_data(seed_points());
    engine
        .autoscale_price_from_data()
        .expect("initial autoscale from points");
    let before = engine.price_domain();

    engine.append_point(DataPoint::new(2.0, 300.0));
    let after = engine.price_domain();
    assert!((after.0 - before.0).abs() <= 1e-12);
    assert!((after.1 - before.1).abs() <= 1e-12);
}

#[test]
fn append_point_autoscales_when_enabled() {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine
        .autoscale_price_from_data()
        .expect("initial autoscale from points");
    let before = engine.price_domain();

    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: false,
        autoscale_on_data_update: true,
        autoscale_on_time_range_change: false,
    });
    engine.append_point(DataPoint::new(2.0, 300.0));
    let after = engine.price_domain();
    assert!(after.1 > before.1);
}

#[test]
fn update_point_replace_autoscales_when_enabled() {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine
        .autoscale_price_from_data()
        .expect("initial autoscale from points");
    let before = engine.price_domain();

    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: false,
        autoscale_on_data_update: true,
        autoscale_on_time_range_change: false,
    });
    engine
        .update_point(DataPoint::new(1.0, 250.0))
        .expect("replace point");
    let after = engine.price_domain();
    assert!(after.1 > before.1);
}

#[test]
fn candles_take_priority_for_realtime_autoscale_source() {
    let mut engine = build_engine();
    engine.set_data(seed_points());
    engine.set_candles(seed_candles());
    engine
        .autoscale_price_from_candles()
        .expect("initial autoscale from candles");
    let before = engine.price_domain();

    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: false,
        autoscale_on_data_update: true,
        autoscale_on_time_range_change: false,
    });
    engine.append_point(DataPoint::new(2.0, 1_000.0));
    let after = engine.price_domain();

    assert!((after.0 - before.0).abs() <= 1e-12);
    assert!((after.1 - before.1).abs() <= 1e-12);
}

#[test]
fn append_candle_autoscales_when_enabled() {
    let mut engine = build_engine();
    engine.set_candles(seed_candles());
    engine
        .autoscale_price_from_candles()
        .expect("initial autoscale from candles");
    let before = engine.price_domain();

    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: false,
        autoscale_on_data_update: true,
        autoscale_on_time_range_change: false,
    });
    engine.append_candle(OhlcBar::new(1.0, 60.0, 90.0, 58.0, 88.0).expect("candle"));
    let after = engine.price_domain();
    assert!(after.1 > before.1);
}

#[test]
fn set_data_autoscales_when_data_set_policy_enabled() {
    let mut engine = build_engine();
    let before = engine.price_domain();
    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: true,
        autoscale_on_data_update: false,
        autoscale_on_time_range_change: false,
    });
    engine.set_data(seed_points());
    let after = engine.price_domain();
    assert!((after.0 - before.0).abs() > 1e-9 || (after.1 - before.1).abs() > 1e-9);
}

#[test]
fn set_candles_autoscales_when_data_set_policy_enabled() {
    let mut engine = build_engine();
    let before = engine.price_domain();
    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: true,
        autoscale_on_data_update: false,
        autoscale_on_time_range_change: false,
    });
    engine.set_candles(seed_candles());
    let after = engine.price_domain();
    assert!((after.0 - before.0).abs() > 1e-9 || (after.1 - before.1).abs() > 1e-9);
}

#[test]
fn pan_time_visible_autoscales_when_time_range_change_policy_is_enabled() {
    let mut engine = build_engine();
    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: false,
        autoscale_on_data_update: false,
        autoscale_on_time_range_change: true,
    });
    engine.set_data(vec![
        DataPoint::new(0.0, 1_000.0),
        DataPoint::new(1.0, 20.0),
        DataPoint::new(2.0, 22.0),
        DataPoint::new(3.0, 24.0),
    ]);
    engine
        .set_time_visible_range(0.0, 3.0)
        .expect("set initial visible range");
    let before = engine.price_domain();

    engine
        .pan_time_visible_by_pixels(-400.0)
        .expect("pan visible range");
    let after = engine.price_domain();

    assert!(after.1 < before.1);
}

#[test]
fn zoom_time_visible_does_not_autoscale_when_time_range_change_policy_is_disabled() {
    let mut engine = build_engine();
    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: false,
        autoscale_on_data_update: false,
        autoscale_on_time_range_change: false,
    });
    engine.set_data(vec![
        DataPoint::new(0.0, 1_000.0),
        DataPoint::new(1.0, 20.0),
        DataPoint::new(2.0, 22.0),
        DataPoint::new(3.0, 24.0),
    ]);
    engine
        .autoscale_price_from_data()
        .expect("baseline full-range autoscale");
    let before = engine.price_domain();

    engine
        .set_time_visible_range(0.0, 3.0)
        .expect("set visible range");
    engine
        .zoom_time_visible_around_time(2.0, 3.0, 1e-6)
        .expect("zoom visible range");

    assert_eq!(engine.price_domain(), before);
}
