use chart_rs::core::{OhlcBar, PriceScale, TimeScale, Viewport, project_candles};
use proptest::prelude::*;

proptest! {
    #[test]
    fn projected_candle_keeps_body_inside_wick(
        time in -1_000_000.0f64..1_000_000.0,
        base in -1_000.0f64..1_000.0,
        span in 0.01f64..1_000.0,
        open_factor in 0.0f64..1.0,
        close_factor in 0.0f64..1.0,
        body_width in 1.0f64..20.0
    ) {
        let low = base;
        let high = base + span;
        let open = low + open_factor * span;
        let close = low + close_factor * span;

        let bar = OhlcBar::new(time, open, high, low, close).expect("valid bar");
        let viewport = Viewport::new(1200, 800);
        let time_scale = TimeScale::new(time - 10.0, time + 10.0).expect("time scale");
        let price_scale = PriceScale::new(low, high).expect("price scale");

        let projected = project_candles(&[bar], time_scale, price_scale, viewport, body_width)
            .expect("projection");

        let c = projected[0];
        prop_assert!(c.body_left < c.body_right);
        prop_assert!(c.wick_top <= c.body_top);
        prop_assert!(c.body_bottom <= c.wick_bottom);
        prop_assert!(c.body_top <= c.body_bottom);
    }
}
