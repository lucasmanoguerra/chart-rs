use chart_rs::api::{
    ChartEngine, ChartEngineConfig, TimeCoordinateIndexPolicy, TimeFilledLogicalSource,
};
use chart_rs::core::{DataPoint, OhlcBar, TimeScaleTuning, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

#[test]
fn map_pixel_to_logical_index_allow_vs_ignore_whitespace() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(10.0, 11.0),
        DataPoint::new(30.0, 12.0),
        DataPoint::new(40.0, 13.0),
    ]);
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit time");
    engine
        .set_time_visible_range(0.0, 40.0)
        .expect("visible range");

    // LWC parity: `indexToCoordinate(2.5)` maps back to float logical index 2.0
    // because `indexToCoordinate` uses +0.5 center-of-bar while
    // `coordinateToFloatIndex` does not.
    let logical_hole_px = engine
        .map_logical_index_to_pixel(2.5)
        .expect("logical to pixel")
        .expect("space");
    let allow = engine
        .map_pixel_to_logical_index(logical_hole_px, TimeCoordinateIndexPolicy::AllowWhitespace)
        .expect("allow whitespace")
        .expect("logical index");
    let ignore = engine
        .map_pixel_to_logical_index(logical_hole_px, TimeCoordinateIndexPolicy::IgnoreWhitespace)
        .expect("ignore whitespace")
        .expect("logical index");

    assert!((allow - 2.0).abs() <= 1e-9);
    assert!((ignore - 3.0).abs() <= 1e-9);
}

#[test]
fn map_pixel_to_logical_index_returns_none_when_reference_step_is_unavailable() {
    let engine = build_engine();
    let logical = engine
        .map_pixel_to_logical_index(100.0, TimeCoordinateIndexPolicy::AllowWhitespace)
        .expect("logical mapping");
    assert!(logical.is_none());
}

#[test]
fn map_pixel_to_logical_index_ceil_exposes_discrete_conversion() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(10.0, 11.0),
        DataPoint::new(30.0, 12.0),
        DataPoint::new(40.0, 13.0),
    ]);
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit time");
    engine
        .set_time_visible_range(0.0, 40.0)
        .expect("visible range");

    // Same LWC asymmetry as above: target float logical 2.2 by projecting 2.7.
    let logical_hole_px = engine
        .map_logical_index_to_pixel(2.7)
        .expect("logical to pixel")
        .expect("space");
    let allow = engine
        .map_pixel_to_logical_index_ceil(
            logical_hole_px,
            TimeCoordinateIndexPolicy::AllowWhitespace,
        )
        .expect("allow ceil")
        .expect("logical index");
    let ignore = engine
        .map_pixel_to_logical_index_ceil(
            logical_hole_px,
            TimeCoordinateIndexPolicy::IgnoreWhitespace,
        )
        .expect("ignore ceil")
        .expect("logical index");

    assert_eq!(allow, 3);
    assert_eq!(ignore, 3);
}

#[test]
fn map_logical_index_to_pixel_reflects_lwc_half_bar_asymmetry() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(10.0, 11.0),
        DataPoint::new(20.0, 12.0),
        DataPoint::new(30.0, 13.0),
    ]);
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit time");
    engine
        .set_time_visible_range(0.0, 30.0)
        .expect("visible range");

    let x = engine
        .map_logical_index_to_pixel(2.0)
        .expect("logical to pixel")
        .expect("space");
    let logical = engine
        .map_pixel_to_logical_index(x, TimeCoordinateIndexPolicy::AllowWhitespace)
        .expect("pixel to logical")
        .expect("logical value");
    assert!((logical - 1.5).abs() <= 1e-9);

    let x_aligned = engine
        .map_logical_index_to_pixel(2.5)
        .expect("logical to pixel")
        .expect("space");
    let logical_aligned = engine
        .map_pixel_to_logical_index(x_aligned, TimeCoordinateIndexPolicy::AllowWhitespace)
        .expect("pixel to logical")
        .expect("logical value");
    assert!((logical_aligned - 2.0).abs() <= 1e-9);
}

#[test]
fn nearest_filled_slot_at_pixel_exposes_source_slot_and_time() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(10.0, 11.0),
        DataPoint::new(20.0, 12.0),
    ]);
    engine.set_candles(vec![
        OhlcBar::new(10.0, 9.0, 11.0, 8.0, 11.0).expect("valid candle"),
    ]);
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit time");
    engine
        .set_time_visible_range(0.0, 20.0)
        .expect("visible range");

    let x = engine
        .map_logical_index_to_pixel(1.0)
        .expect("logical to pixel")
        .expect("space");
    let slot = engine
        .nearest_filled_logical_slot_at_pixel(x)
        .expect("nearest slot")
        .expect("slot");
    assert_eq!(slot.source, TimeFilledLogicalSource::Candles);
    assert_eq!(slot.slot, 0);
    assert!((slot.time - 10.0).abs() <= 1e-9);
    assert!((slot.logical_index - 1.0).abs() <= 1e-9);
}

#[test]
fn next_prev_filled_logical_index_skip_whitespace() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(10.0, 11.0),
        DataPoint::new(20.0, 12.0),
        DataPoint::new(50.0, 15.0),
    ]);
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .expect("fit time");
    engine
        .set_time_visible_range(0.0, 50.0)
        .expect("visible range");

    let next = engine
        .next_filled_logical_index(2.1)
        .expect("next logical index");
    let prev = engine
        .prev_filled_logical_index(4.9)
        .expect("prev logical index");

    assert_eq!(next, Some(5.0));
    assert_eq!(prev, Some(2.0));
}
