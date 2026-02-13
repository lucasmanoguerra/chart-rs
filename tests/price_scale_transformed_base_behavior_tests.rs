use chart_rs::api::{
    ChartEngine, ChartEngineConfig, PriceScaleTransformedBaseBehavior,
    PriceScaleTransformedBaseSource,
};
use chart_rs::core::{DataPoint, OhlcBar, PriceScaleMode, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(90.0, 110.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

#[test]
fn explicit_transformed_base_is_applied_for_percentage_mode() {
    let mut engine = build_engine();
    engine
        .set_price_scale_transformed_base_behavior(PriceScaleTransformedBaseBehavior {
            explicit_base_price: Some(200.0),
            dynamic_source: PriceScaleTransformedBaseSource::DomainStart,
        })
        .expect("set transformed base behavior");
    engine
        .set_price_scale_mode(PriceScaleMode::Percentage)
        .expect("set percentage mode");

    assert_eq!(engine.price_scale_transformed_base_value(), Some(200.0));

    let px = engine.map_price_to_pixel(220.0).expect("price to pixel");
    let roundtrip = engine.map_pixel_to_price(px).expect("pixel to price");
    assert!((roundtrip - 220.0).abs() <= 1e-9);
}

#[test]
fn dynamic_last_visible_base_updates_on_time_range_change() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(0.0, 100.0),
        DataPoint::new(10.0, 105.0),
        DataPoint::new(20.0, 110.0),
        DataPoint::new(30.0, 120.0),
    ]);
    engine
        .set_price_scale_transformed_base_behavior(PriceScaleTransformedBaseBehavior {
            explicit_base_price: None,
            dynamic_source: PriceScaleTransformedBaseSource::LastVisibleData,
        })
        .expect("set transformed base behavior");
    engine
        .set_price_scale_mode(PriceScaleMode::Percentage)
        .expect("set percentage mode");

    engine
        .set_time_visible_range(0.0, 20.0)
        .expect("set visible range");
    assert_eq!(engine.price_scale_transformed_base_value(), Some(110.0));

    engine
        .set_time_visible_range(20.0, 30.0)
        .expect("set visible range");
    assert_eq!(engine.price_scale_transformed_base_value(), Some(120.0));
}

#[test]
fn chart_config_bootstraps_transformed_base_behavior() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 10.0)
        .with_price_domain(90.0, 110.0)
        .with_price_scale_mode(PriceScaleMode::IndexedTo100)
        .with_price_scale_transformed_base_behavior(PriceScaleTransformedBaseBehavior {
            explicit_base_price: Some(95.0),
            dynamic_source: PriceScaleTransformedBaseSource::FirstData,
        });
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    assert_eq!(
        engine.price_scale_transformed_base_behavior(),
        PriceScaleTransformedBaseBehavior {
            explicit_base_price: Some(95.0),
            dynamic_source: PriceScaleTransformedBaseSource::FirstData,
        }
    );
    assert_eq!(engine.price_scale_transformed_base_value(), Some(95.0));
}

#[test]
fn invalid_explicit_transformed_base_is_rejected() {
    let mut engine = build_engine();
    let err = engine
        .set_price_scale_transformed_base_behavior(PriceScaleTransformedBaseBehavior {
            explicit_base_price: Some(0.0),
            dynamic_source: PriceScaleTransformedBaseSource::DomainStart,
        })
        .expect_err("zero base must fail");
    assert!(matches!(err, chart_rs::ChartError::InvalidData(_)));
}

#[test]
fn dynamic_base_tie_break_prefers_candle_source_on_equal_time() {
    let mut engine = build_engine();
    engine.set_data(vec![DataPoint::new(10.0, 101.0)]);
    engine.set_candles(vec![
        OhlcBar::new(10.0, 95.0, 106.0, 94.0, 106.0).expect("valid candle"),
    ]);
    engine
        .set_price_scale_transformed_base_behavior(PriceScaleTransformedBaseBehavior {
            explicit_base_price: None,
            dynamic_source: PriceScaleTransformedBaseSource::FirstData,
        })
        .expect("set transformed base behavior");
    engine
        .set_price_scale_mode(PriceScaleMode::Percentage)
        .expect("set percentage mode");

    assert_eq!(engine.price_scale_transformed_base_value(), Some(106.0));
}

#[test]
fn visible_dynamic_base_falls_back_to_all_data_when_window_is_empty() {
    let mut engine = build_engine();
    engine.set_data(vec![
        DataPoint::new(10.0, 111.0),
        DataPoint::new(20.0, 122.0),
    ]);
    engine
        .set_price_scale_transformed_base_behavior(PriceScaleTransformedBaseBehavior {
            explicit_base_price: None,
            dynamic_source: PriceScaleTransformedBaseSource::FirstVisibleData,
        })
        .expect("set transformed base behavior");
    engine
        .set_price_scale_mode(PriceScaleMode::IndexedTo100)
        .expect("set indexed mode");

    engine
        .set_time_visible_range(200.0, 300.0)
        .expect("set off-window range");
    assert_eq!(engine.price_scale_transformed_base_value(), Some(111.0));
}
