use chart_rs::core::{LinearScale, Viewport};
use proptest::prelude::*;

proptest! {
    #[test]
    fn linear_scale_round_trip_property(
        domain_start in -1_000_000.0f64..1_000_000.0,
        domain_span in 0.001f64..1_000_000.0,
        value_factor in 0.0f64..1.0
    ) {
        let domain_end = domain_start + domain_span;
        let value = domain_start + value_factor * domain_span;

        let viewport = Viewport::new(2048, 1024);
        let scale = LinearScale::new(domain_start, domain_end).expect("valid scale");

        let px = scale.domain_to_pixel(value, viewport).expect("to pixel");
        let recovered = scale.pixel_to_domain(px, viewport).expect("from pixel");

        prop_assert!((recovered - value).abs() <= 1e-7);
    }
}
