use chart_rs::api::{ChartEngine, ChartEngineConfig, EngineSnapshot};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::render::NullRenderer;
use proptest::prelude::*;

proptest! {
    #[test]
    fn crosshair_snap_prefers_nearest_data_point(
        time_start in -10_000.0f64..10_000.0,
        gap in 1.0f64..2_000.0,
        offset_factor in 0.0f64..0.24,
        y0 in -1_000.0f64..1_000.0,
        y1 in -1_000.0f64..1_000.0
    ) {
        let t0 = time_start;
        let t1 = time_start + gap;
        let pointer_time = t0 + gap * offset_factor;

        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1200, 700), t0 - 1.0, t1 + 1.0)
            .with_price_domain(-2_000.0, 2_000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        engine.set_data(vec![DataPoint::new(t0, y0), DataPoint::new(t1, y1)]);
        let pointer_x = engine.map_x_to_pixel(pointer_time).expect("map pointer time");
        engine.pointer_move(pointer_x, 250.0);

        let crosshair = engine.crosshair_state();
        let snapped_x = crosshair.snapped_x.expect("snapped x");
        let snapped_time = crosshair.snapped_time.expect("snapped time");
        let snapped_price = crosshair.snapped_price.expect("snapped price");
        let expected_x = engine.map_x_to_pixel(t0).expect("expected x");

        prop_assert!((snapped_x - expected_x).abs() <= 1e-7);
        prop_assert!((snapped_time - t0).abs() <= 1e-7);
        prop_assert!((snapped_price - y0).abs() <= 1e-7);
    }

    #[test]
    fn crosshair_snap_prefers_nearest_candle_close(
        time_start in -10_000.0f64..10_000.0,
        gap in 1.0f64..2_000.0,
        offset_factor in 0.76f64..1.0,
        close0 in 10.0f64..100.0,
        close1 in 101.0f64..200.0
    ) {
        let t0 = time_start;
        let t1 = time_start + gap;
        let pointer_time = t0 + gap * offset_factor;

        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1200, 700), t0 - 1.0, t1 + 1.0)
            .with_price_domain(0.0, 300.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        let bar0 = OhlcBar::new(t0, close0, close0 + 5.0, close0 - 5.0, close0).expect("valid bar0");
        let bar1 = OhlcBar::new(t1, close1, close1 + 5.0, close1 - 5.0, close1).expect("valid bar1");
        engine.set_candles(vec![bar0, bar1]);

        let pointer_x = engine.map_x_to_pixel(pointer_time).expect("map pointer time");
        engine.pointer_move(pointer_x, 260.0);

        let crosshair = engine.crosshair_state();
        let snapped_x = crosshair.snapped_x.expect("snapped x");
        let snapped_time = crosshair.snapped_time.expect("snapped time");
        let snapped_price = crosshair.snapped_price.expect("snapped price");
        let expected_x = engine.map_x_to_pixel(t1).expect("expected x");

        prop_assert!((snapped_x - expected_x).abs() <= 1e-7);
        prop_assert!((snapped_time - t1).abs() <= 1e-7);
        prop_assert!((snapped_price - close1).abs() <= 1e-7);
    }

    #[test]
    fn snapshot_keeps_geometry_count_and_roundtrips_json(
        candle_count in 1usize..64,
        body_width in 0.5f64..20.0
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(
            Viewport::new(1280, 720),
            -1.0,
            candle_count as f64 + 1.0
        ).with_price_domain(0.0, 200.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        let mut candles = Vec::with_capacity(candle_count);
        for i in 0..candle_count {
            let time = i as f64;
            let base = 50.0 + i as f64 * 0.5;
            let delta = if i % 2 == 0 { 2.0 } else { -2.0 };
            let open = base;
            let close = base + delta;
            let low = open.min(close) - 1.0;
            let high = open.max(close) + 1.0;
            candles.push(
                OhlcBar::new(time, open, high, low, close).expect("generated candle must be valid")
            );
        }

        engine.set_series_metadata("series-id", "candles-main");
        engine.set_series_metadata("series-type", "candlestick");
        engine.set_candles(candles);

        let snapshot = engine.snapshot(body_width).expect("snapshot");
        prop_assert_eq!(snapshot.candle_geometry.len(), candle_count);
        prop_assert_eq!(snapshot.series_metadata.len(), 2);
        for geometry in &snapshot.candle_geometry {
            prop_assert!(geometry.center_x.is_finite());
            prop_assert!(geometry.body_left.is_finite());
            prop_assert!(geometry.body_right.is_finite());
            prop_assert!(geometry.wick_top.is_finite());
            prop_assert!(geometry.wick_bottom.is_finite());
        }

        let json = engine.snapshot_json_pretty(body_width).expect("snapshot json");
        let restored: EngineSnapshot = serde_json::from_str(&json).expect("snapshot roundtrip");
        prop_assert_eq!(restored.candle_geometry.len(), candle_count);
        prop_assert_eq!(restored.series_metadata.len(), 2);
    }

    #[test]
    fn pan_by_pixels_preserves_span(
        time_start in -10_000.0f64..10_000.0,
        time_span in 1.0f64..5_000.0,
        delta_px in -500.0f64..500.0
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(
            Viewport::new(1000, 700),
            time_start,
            time_start + time_span,
        ).with_price_domain(0.0, 1.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        let (start_before, end_before) = engine.time_visible_range();
        let span_before = end_before - start_before;

        engine
            .pan_time_visible_by_pixels(delta_px)
            .expect("pan should work");
        let (start_after, end_after) = engine.time_visible_range();
        let span_after = end_after - start_after;

        let expected_delta = -(delta_px / 1000.0) * span_before;
        prop_assert!((span_after - span_before).abs() <= 1e-7);
        prop_assert!((start_after - (start_before + expected_delta)).abs() <= 1e-7);
        prop_assert!((end_after - (end_before + expected_delta)).abs() <= 1e-7);
    }
}
