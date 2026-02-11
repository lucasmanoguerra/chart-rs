use chart_rs::core::{DataPoint, LinearScale, PriceScale, TimeScale, Viewport};

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

#[test]
fn time_scale_round_trip_within_tolerance() {
    let viewport = Viewport::new(1200, 600);
    let scale = TimeScale::new(1_700_000_000.0, 1_700_000_600.0).expect("valid scale");

    let original = 1_700_000_123.0;
    let px = scale.time_to_pixel(original, viewport).expect("to pixel");
    let recovered = scale.pixel_to_time(px, viewport).expect("from pixel");

    assert!((recovered - original).abs() <= 1e-9);
}

#[test]
fn price_scale_uses_inverted_y_axis() {
    let viewport = Viewport::new(800, 600);
    let scale = PriceScale::new(10.0, 110.0).expect("valid scale");

    let top = scale.price_to_pixel(110.0, viewport).expect("top pixel");
    let bottom = scale.price_to_pixel(10.0, viewport).expect("bottom pixel");

    assert_eq!(top, 0.0);
    assert_eq!(bottom, 600.0);
}

#[test]
fn price_scale_from_flat_data_adds_padding() {
    let points = vec![
        DataPoint::new(1.0, 42.0),
        DataPoint::new(2.0, 42.0),
        DataPoint::new(3.0, 42.0),
    ];

    let scale = PriceScale::from_data(&points).expect("autoscale from flat data");
    let (min, max) = scale.domain();
    assert!(min < 42.0);
    assert!(max > 42.0);
}
