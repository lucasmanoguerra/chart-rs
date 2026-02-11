use chart_rs::core::{LinearScale, Viewport};

#[test]
fn scale_round_trip_within_tolerance() {
    let viewport = Viewport::new(1000, 600);
    let scale = LinearScale::new(10.0, 110.0).expect("valid scale");

    let original = 42.5;
    let px = scale.domain_to_pixel(original, viewport).expect("to pixel");
    let recovered = scale.pixel_to_domain(px, viewport).expect("from pixel");

    let epsilon = 1e-9;
    assert!((recovered - original).abs() <= epsilon);
}

#[test]
fn invalid_viewport_is_rejected() {
    let viewport = Viewport::new(0, 0);
    let scale = LinearScale::new(0.0, 1.0).expect("valid scale");

    let result = scale.domain_to_pixel(0.5, viewport);
    assert!(result.is_err());
}
