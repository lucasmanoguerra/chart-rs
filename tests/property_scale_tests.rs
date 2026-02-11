use chart_rs::core::{
    DataPoint, PriceScale, PriceScaleTuning, TimeScale, TimeScaleTuning, Viewport,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn time_scale_round_trip_property(
        time_start in -1_000_000.0f64..1_000_000.0,
        time_span in 0.001f64..1_000_000.0,
        value_factor in 0.0f64..1.0
    ) {
        let time_end = time_start + time_span;
        let value = time_start + value_factor * time_span;

        let viewport = Viewport::new(2048, 1024);
        let scale = TimeScale::new(time_start, time_end).expect("valid scale");

        let px = scale.time_to_pixel(value, viewport).expect("to pixel");
        let recovered = scale.pixel_to_time(px, viewport).expect("from pixel");

        prop_assert!((recovered - value).abs() <= 1e-7);
    }

    #[test]
    fn price_scale_round_trip_property(
        price_min in -1_000_000.0f64..1_000_000.0,
        price_span in 0.001f64..1_000_000.0,
        value_factor in 0.0f64..1.0
    ) {
        let price_max = price_min + price_span;
        let value = price_min + value_factor * price_span;

        let viewport = Viewport::new(2048, 1024);
        let scale = PriceScale::new(price_min, price_max).expect("valid scale");

        let px = scale.price_to_pixel(value, viewport).expect("to pixel");
        let recovered = scale.pixel_to_price(px, viewport).expect("from pixel");

        prop_assert!((recovered - value).abs() <= 1e-7);
    }

    #[test]
    fn time_scale_tuned_visible_range_contains_data(
        a in -1_000_000.0f64..1_000_000.0,
        span in 0.001f64..1_000_000.0,
        left_pad in 0.0f64..0.5,
        right_pad in 0.0f64..0.5
    ) {
        let b = a + span;
        let points = vec![DataPoint::new(a, 1.0), DataPoint::new(b, 2.0)];
        let tuning = TimeScaleTuning {
            left_padding_ratio: left_pad,
            right_padding_ratio: right_pad,
            min_span_absolute: 1.0,
        };

        let scale = TimeScale::from_data_tuned(&points, tuning).expect("fit");
        let (visible_start, visible_end) = scale.visible_range();
        prop_assert!(visible_start <= a);
        prop_assert!(visible_end >= b);
    }

    #[test]
    fn price_scale_tuned_domain_contains_data(
        min_price in -1_000_000.0f64..1_000_000.0,
        span in 0.001f64..1_000_000.0,
        top_pad in 0.0f64..0.5,
        bottom_pad in 0.0f64..0.5
    ) {
        let max_price = min_price + span;
        let points = vec![
            DataPoint::new(0.0, min_price),
            DataPoint::new(1.0, max_price),
        ];
        let tuning = PriceScaleTuning {
            top_padding_ratio: top_pad,
            bottom_padding_ratio: bottom_pad,
            min_span_absolute: 0.000_001,
        };

        let scale = PriceScale::from_data_tuned(&points, tuning).expect("fit");
        let (domain_min, domain_max) = scale.domain();
        prop_assert!(domain_min <= min_price);
        prop_assert!(domain_max >= max_price);
    }
}
