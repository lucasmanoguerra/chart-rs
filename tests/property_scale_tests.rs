use chart_rs::core::{PriceScale, TimeScale, Viewport};
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
}
