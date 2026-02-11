use chart_rs::core::{OhlcBar, PriceScale, TimeScale, Viewport, project_bars};
use chart_rs::{
    api::{ChartEngine, ChartEngineConfig},
    render::NullRenderer,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn projected_bar_keeps_open_close_inside_high_low(
        time in -1_000_000.0f64..1_000_000.0,
        base in -1_000.0f64..1_000.0,
        span in 0.01f64..1_000.0,
        open_factor in 0.0f64..1.0,
        close_factor in 0.0f64..1.0,
        tick_width in 1.0f64..20.0
    ) {
        let low = base;
        let high = base + span;
        let open = low + open_factor * span;
        let close = low + close_factor * span;

        let bar = OhlcBar::new(time, open, high, low, close).expect("valid bar");
        let viewport = Viewport::new(1200, 800);
        let time_scale = TimeScale::new(time - 10.0, time + 10.0).expect("time scale");
        let price_scale = PriceScale::new(low, high).expect("price scale");

        let projected =
            project_bars(&[bar], time_scale, price_scale, viewport, tick_width).expect("projection");
        let b = projected[0];

        prop_assert!(b.open_x < b.center_x);
        prop_assert!(b.center_x < b.close_x);
        prop_assert!(b.high_y <= b.open_y);
        prop_assert!(b.high_y <= b.close_y);
        prop_assert!(b.open_y <= b.low_y);
        prop_assert!(b.close_y <= b.low_y);
    }

    #[test]
    fn visible_bar_projection_count_matches_visible_filter(
        candle_count in 2usize..64,
        start_ratio in 0.0f64..0.6,
        width_ratio in 0.1f64..0.7,
        tick_width in 1.0f64..20.0
    ) {
        let renderer = NullRenderer::default();
        let right = (candle_count - 1) as f64;
        let config = ChartEngineConfig::new(
            Viewport::new(1200, 800),
            -1.0,
            right + 1.0,
        ).with_price_domain(0.0, 500.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        let mut candles = Vec::with_capacity(candle_count);
        for i in 0..candle_count {
            let time = i as f64;
            let open = 100.0 + time;
            let close = if i % 2 == 0 { open + 2.0 } else { open - 2.0 };
            let low = open.min(close) - 1.0;
            let high = open.max(close) + 1.0;
            candles.push(OhlcBar::new(time, open, high, low, close).expect("valid candle"));
        }
        engine.set_candles(candles);

        let visible_start = right * start_ratio;
        let visible_end = (visible_start + right * width_ratio).min(right);
        prop_assume!(visible_end > visible_start);
        engine
            .set_time_visible_range(visible_start, visible_end)
            .expect("set visible");

        let expected = engine.visible_candles().len();
        let projected = engine
            .project_visible_bars(tick_width)
            .expect("project visible");
        prop_assert_eq!(projected.len(), expected);
    }
}
