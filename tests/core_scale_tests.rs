use chart_rs::core::{
    DataPoint, LinearScale, PriceScale, PriceScaleMode, PriceScaleTuning, TimeScale,
    TimeScaleTuning, Viewport,
};

fn is_log_125_ladder(value: f64) -> bool {
    if !value.is_finite() || value <= 0.0 {
        return false;
    }
    let exponent = value.log10().floor();
    let base = 10_f64.powf(exponent);
    let mantissa = value / base;
    let scale = value.abs().max(1.0);
    (mantissa - 1.0).abs() <= scale * 1e-12
        || (mantissa - 2.0).abs() <= scale * 1e-12
        || (mantissa - 5.0).abs() <= scale * 1e-12
}

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
fn time_scale_visible_range_controls_mapping() {
    let viewport = Viewport::new(1000, 600);
    let mut scale = TimeScale::new(0.0, 10.0).expect("valid scale");
    scale
        .set_visible_range(2.0, 6.0)
        .expect("set visible range");

    let left = scale.time_to_pixel(2.0, viewport).expect("left");
    let right = scale.time_to_pixel(6.0, viewport).expect("right");
    assert_eq!(left, 0.0);
    assert_eq!(right, 1000.0);
}

#[test]
fn time_scale_from_data_tuned_applies_padding() {
    let points = vec![DataPoint::new(10.0, 1.0), DataPoint::new(20.0, 2.0)];
    let tuning = TimeScaleTuning {
        left_padding_ratio: 0.1,
        right_padding_ratio: 0.2,
        min_span_absolute: 1.0,
    };

    let scale = TimeScale::from_data_tuned(&points, tuning).expect("time fit");
    let (visible_start, visible_end) = scale.visible_range();
    assert!((visible_start - 9.0).abs() <= 1e-9);
    assert!((visible_end - 22.0).abs() <= 1e-9);
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

#[test]
fn price_scale_tuned_padding_is_applied() {
    let points = vec![DataPoint::new(1.0, 10.0), DataPoint::new(2.0, 20.0)];
    let tuning = PriceScaleTuning {
        top_padding_ratio: 0.2,
        bottom_padding_ratio: 0.1,
        min_span_absolute: 0.000_001,
    };

    let scale = PriceScale::from_data_tuned(&points, tuning).expect("price fit");
    let (min, max) = scale.domain();
    assert!((min - 9.0).abs() <= 1e-9);
    assert!((max - 22.0).abs() <= 1e-9);
}

#[test]
fn price_scale_log_mode_keeps_ratio_spacing() {
    let viewport = Viewport::new(800, 600);
    let scale = PriceScale::new_with_mode(1.0, 1_000.0, PriceScaleMode::Log).expect("log scale");

    let y1 = scale.price_to_pixel(1.0, viewport).expect("y1");
    let y10 = scale.price_to_pixel(10.0, viewport).expect("y10");
    let y100 = scale.price_to_pixel(100.0, viewport).expect("y100");
    let y1000 = scale.price_to_pixel(1_000.0, viewport).expect("y1000");

    let d1 = y1 - y10;
    let d2 = y10 - y100;
    let d3 = y100 - y1000;
    assert!((d1 - d2).abs() <= 1e-6);
    assert!((d2 - d3).abs() <= 1e-6);

    let price = 25.0;
    let px = scale.price_to_pixel(price, viewport).expect("to pixel");
    let recovered = scale.pixel_to_price(px, viewport).expect("from pixel");
    assert!((recovered - price).abs() <= 1e-9);
}

#[test]
fn price_scale_log_mode_rejects_non_positive_prices() {
    let viewport = Viewport::new(800, 600);
    assert!(PriceScale::new_with_mode(0.0, 100.0, PriceScaleMode::Log).is_err());

    let scale = PriceScale::new_with_mode(1.0, 100.0, PriceScaleMode::Log).expect("log scale");
    assert!(scale.price_to_pixel(0.0, viewport).is_err());
}

#[test]
fn price_scale_log_mode_ticks_follow_125_ladder() {
    let scale = PriceScale::new_with_mode(1.0, 1_000.0, PriceScaleMode::Log).expect("log scale");
    let ticks = scale.ticks(10).expect("ticks");

    assert!(!ticks.is_empty());
    assert!(ticks.len() <= 10);
    assert!(ticks.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(ticks.iter().all(|value| is_log_125_ladder(*value)));
}

#[test]
fn price_scale_log_mode_ticks_preserve_domain_direction() {
    let scale = PriceScale::new_with_mode(1_000.0, 1.0, PriceScaleMode::Log).expect("log scale");
    let ticks = scale.ticks(8).expect("ticks");

    assert!(!ticks.is_empty());
    assert!(ticks.windows(2).all(|pair| pair[0] > pair[1]));
    assert!(ticks.iter().all(|value| is_log_125_ladder(*value)));
}
