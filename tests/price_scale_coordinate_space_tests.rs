use chart_rs::core::{PriceScale, PriceScaleMode, Viewport};

#[test]
fn price_coordinate_space_roundtrip_matches_price_scale_linear() {
    let viewport = Viewport::new(800, 600);
    let scale = PriceScale::new_with_mode(10.0, 110.0, PriceScaleMode::Linear)
        .expect("linear scale")
        .with_margins(0.2, 0.1)
        .expect("margins");
    let space = scale.coordinate_space(viewport).expect("coordinate space");

    let price = 42.5;
    let y = scale
        .price_to_pixel(price, viewport)
        .expect("price to pixel");
    let transformed = space.pixel_to_transformed(y).expect("pixel to transformed");
    let recovered = scale.pixel_to_price(y, viewport).expect("pixel to price");

    assert!((recovered - price).abs() <= 1e-9);
    assert!((transformed - price).abs() <= 1e-9);
}

#[test]
fn price_coordinate_space_roundtrip_matches_price_scale_log() {
    let viewport = Viewport::new(800, 600);
    let scale = PriceScale::new_with_mode(1.0, 1000.0, PriceScaleMode::Log)
        .expect("log scale")
        .with_margins(0.15, 0.05)
        .expect("margins");
    let space = scale.coordinate_space(viewport).expect("coordinate space");

    let price = 25.0;
    let y = scale
        .price_to_pixel(price, viewport)
        .expect("price to pixel");
    let transformed = space.pixel_to_transformed(y).expect("pixel to transformed");
    let recovered = scale.pixel_to_price(y, viewport).expect("pixel to price");

    assert!((recovered - price).abs() <= 1e-9);
    assert!((transformed - price.ln()).abs() <= 1e-9);
}

#[test]
fn price_coordinate_space_respects_inverted_orientation() {
    let viewport = Viewport::new(800, 600);
    let normal = PriceScale::new(10.0, 110.0)
        .expect("scale")
        .with_margins(0.1, 0.1)
        .expect("margins");
    let inverted = normal.with_inverted(true);

    let high_normal = normal.price_to_pixel(110.0, viewport).expect("normal high");
    let low_normal = normal.price_to_pixel(10.0, viewport).expect("normal low");
    let high_inverted = inverted
        .price_to_pixel(110.0, viewport)
        .expect("inverted high");
    let low_inverted = inverted
        .price_to_pixel(10.0, viewport)
        .expect("inverted low");

    assert!(high_normal < low_normal);
    assert!(high_inverted > low_inverted);
}

#[test]
fn price_coordinate_space_reports_internal_height_from_margins() {
    let viewport = Viewport::new(800, 600);
    let scale = PriceScale::new(10.0, 110.0)
        .expect("scale")
        .with_margins(0.2, 0.1)
        .expect("margins");
    let space = scale.coordinate_space(viewport).expect("coordinate space");

    assert!((space.internal_height_px() - 420.0).abs() <= 1e-9);
}
