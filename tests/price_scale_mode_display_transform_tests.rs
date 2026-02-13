use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{PriceScale, PriceScaleMode, Viewport};
use chart_rs::render::NullRenderer;

#[test]
fn percentage_mode_uses_base_relative_transform_roundtrip() {
    let viewport = Viewport::new(800, 600);
    let scale = PriceScale::new_with_mode(100.0, 120.0, PriceScaleMode::Percentage)
        .expect("percentage scale");
    let space = scale.coordinate_space(viewport).expect("coordinate space");

    assert_eq!(scale.base_value(), Some(100.0));

    let y = scale
        .price_to_pixel(110.0, viewport)
        .expect("price to pixel");
    let transformed = space.pixel_to_transformed(y).expect("pixel to transformed");
    let recovered = scale.pixel_to_price(y, viewport).expect("pixel to price");

    assert!((transformed - 10.0).abs() <= 1e-9);
    assert!((recovered - 110.0).abs() <= 1e-9);
}

#[test]
fn indexed_to_100_mode_uses_base_relative_transform_roundtrip() {
    let viewport = Viewport::new(800, 600);
    let scale = PriceScale::new_with_mode(50.0, 150.0, PriceScaleMode::IndexedTo100)
        .expect("indexed scale");
    let space = scale.coordinate_space(viewport).expect("coordinate space");

    assert_eq!(scale.base_value(), Some(50.0));

    let y = scale
        .price_to_pixel(75.0, viewport)
        .expect("price to pixel");
    let transformed = space.pixel_to_transformed(y).expect("pixel to transformed");
    let recovered = scale.pixel_to_price(y, viewport).expect("pixel to price");

    assert!((transformed - 150.0).abs() <= 1e-9);
    assert!((recovered - 75.0).abs() <= 1e-9);
}

#[test]
fn axis_drag_pan_price_matches_linear_domain_shift_for_percentage_and_indexed_modes() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 100.0).with_price_domain(50.0, 150.0);

    let mut linear = ChartEngine::new(renderer, config).expect("linear engine");
    let mut percentage =
        ChartEngine::new(NullRenderer::default(), config).expect("percentage engine");
    let mut indexed = ChartEngine::new(NullRenderer::default(), config).expect("indexed engine");

    percentage
        .set_price_scale_mode(PriceScaleMode::Percentage)
        .expect("set percentage mode");
    indexed
        .set_price_scale_mode(PriceScaleMode::IndexedTo100)
        .expect("set indexed mode");

    let changed_linear = linear
        .axis_drag_pan_price(36.0, 120.0)
        .expect("linear axis pan");
    let changed_percentage = percentage
        .axis_drag_pan_price(36.0, 120.0)
        .expect("percentage axis pan");
    let changed_indexed = indexed
        .axis_drag_pan_price(36.0, 120.0)
        .expect("indexed axis pan");

    assert!(changed_linear);
    assert_eq!(changed_percentage, changed_linear);
    assert_eq!(changed_indexed, changed_linear);

    let linear_domain = linear.price_domain();
    let percentage_domain = percentage.price_domain();
    let indexed_domain = indexed.price_domain();
    assert!((percentage_domain.0 - linear_domain.0).abs() <= 1e-9);
    assert!((percentage_domain.1 - linear_domain.1).abs() <= 1e-9);
    assert!((indexed_domain.0 - linear_domain.0).abs() <= 1e-9);
    assert!((indexed_domain.1 - linear_domain.1).abs() <= 1e-9);
}
