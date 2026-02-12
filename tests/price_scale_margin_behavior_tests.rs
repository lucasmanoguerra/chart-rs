use chart_rs::api::{ChartEngine, ChartEngineConfig, PriceScaleMarginBehavior};
use chart_rs::core::{DataPoint, PriceScaleMode, Viewport};
use chart_rs::render::NullRenderer;

fn build_engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 100.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

#[test]
fn price_scale_margins_default_to_lightweight_values() {
    let engine = build_engine();
    let margins = engine.price_scale_margin_behavior();
    assert!((margins.top_margin_ratio - 0.2).abs() <= 1e-12);
    assert!((margins.bottom_margin_ratio - 0.1).abs() <= 1e-12);
}

#[test]
fn setting_margins_adds_top_bottom_whitespace() {
    let mut engine = build_engine();
    engine
        .set_price_scale_margin_behavior(PriceScaleMarginBehavior {
            top_margin_ratio: 0.1,
            bottom_margin_ratio: 0.2,
        })
        .expect("set margins");

    let top_price_y = engine.map_price_to_pixel(100.0).expect("map top price");
    let bottom_price_y = engine.map_price_to_pixel(0.0).expect("map bottom price");
    assert!((top_price_y - 50.0).abs() <= 1e-9);
    assert!((bottom_price_y - 400.0).abs() <= 1e-9);
}

#[test]
fn margins_preserve_roundtrip_mapping() {
    let mut engine = build_engine();
    engine
        .set_price_scale_margin_behavior(PriceScaleMarginBehavior {
            top_margin_ratio: 0.12,
            bottom_margin_ratio: 0.18,
        })
        .expect("set margins");

    for value in [0.0, 10.0, 55.0, 100.0] {
        let px = engine.map_price_to_pixel(value).expect("map price");
        let back = engine.map_pixel_to_price(px).expect("map pixel");
        assert!((back - value).abs() <= 1e-9);
    }
}

#[test]
fn invalid_margins_are_rejected() {
    let mut engine = build_engine();
    let err = engine
        .set_price_scale_margin_behavior(PriceScaleMarginBehavior {
            top_margin_ratio: -0.1,
            bottom_margin_ratio: 0.1,
        })
        .expect_err("negative margin must fail");
    assert!(matches!(err, chart_rs::ChartError::InvalidData(_)));

    let err = engine
        .set_price_scale_margin_behavior(PriceScaleMarginBehavior {
            top_margin_ratio: 0.6,
            bottom_margin_ratio: 0.4,
        })
        .expect_err("sum >= 1 must fail");
    assert!(matches!(err, chart_rs::ChartError::InvalidData(_)));
}

#[test]
fn margins_are_preserved_across_mode_switch_and_autoscale() {
    let mut engine = build_engine();
    engine
        .set_price_scale_margin_behavior(PriceScaleMarginBehavior {
            top_margin_ratio: 0.08,
            bottom_margin_ratio: 0.15,
        })
        .expect("set margins");

    engine
        .set_price_scale_mode(PriceScaleMode::Linear)
        .expect("mode switch");
    assert_eq!(
        engine.price_scale_margin_behavior(),
        PriceScaleMarginBehavior {
            top_margin_ratio: 0.08,
            bottom_margin_ratio: 0.15,
        }
    );

    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(1.0, 25.0),
        DataPoint::new(2.0, 40.0),
    ]);
    engine
        .autoscale_price_from_data()
        .expect("autoscale from points");
    assert_eq!(
        engine.price_scale_margin_behavior(),
        PriceScaleMarginBehavior {
            top_margin_ratio: 0.08,
            bottom_margin_ratio: 0.15,
        }
    );
}
