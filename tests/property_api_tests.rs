use chart_rs::api::{ChartEngine, ChartEngineConfig, EngineSnapshot};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::NullRenderer;
use proptest::prelude::*;
use std::sync::Arc;

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

    #[test]
    fn visible_points_are_inside_visible_window(
        time_start in -10_000.0f64..10_000.0,
        time_span in 10.0f64..5_000.0,
        visible_offset in 0.0f64..0.5,
        visible_width_ratio in 0.2f64..0.8
    ) {
        let renderer = NullRenderer::default();
        let time_end = time_start + time_span;
        let config = ChartEngineConfig::new(
            Viewport::new(1200, 700),
            time_start,
            time_end,
        ).with_price_domain(0.0, 1000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        let points = vec![
            DataPoint::new(time_start, 10.0),
            DataPoint::new(time_start + time_span * 0.25, 20.0),
            DataPoint::new(time_start + time_span * 0.5, 30.0),
            DataPoint::new(time_start + time_span * 0.75, 40.0),
            DataPoint::new(time_end, 50.0),
        ];
        engine.set_data(points);

        let visible_start = time_start + time_span * visible_offset;
        let visible_end = visible_start + time_span * visible_width_ratio;
        prop_assume!(visible_end <= time_end);
        engine
            .set_time_visible_range(visible_start, visible_end)
            .expect("set visible range");

        let visible = engine.visible_points();
        for point in &visible {
            prop_assert!(point.x >= visible_start);
            prop_assert!(point.x <= visible_end);
        }
    }

    #[test]
    fn normal_crosshair_mode_never_snaps(
        time_start in -10_000.0f64..10_000.0,
        gap in 1.0f64..2_000.0,
        offset_factor in 0.0f64..1.0,
        y0 in -1_000.0f64..1_000.0,
        y1 in -1_000.0f64..1_000.0,
        pointer_y in -10_000.0f64..10_000.0
    ) {
        let t0 = time_start;
        let t1 = time_start + gap;
        let pointer_time = t0 + gap * offset_factor;

        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1200, 700), t0 - 1.0, t1 + 1.0)
            .with_price_domain(-2_000.0, 2_000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        engine.set_data(vec![DataPoint::new(t0, y0), DataPoint::new(t1, y1)]);
        engine.set_crosshair_mode(CrosshairMode::Normal);

        let pointer_x = engine.map_x_to_pixel(pointer_time).expect("map pointer time");
        engine.pointer_move(pointer_x, pointer_y);

        let crosshair = engine.crosshair_state();
        prop_assert!(crosshair.visible);
        prop_assert!((crosshair.x - pointer_x).abs() <= 1e-7);
        prop_assert!((crosshair.y - pointer_y).abs() <= 1e-7);
        prop_assert!(crosshair.snapped_x.is_none());
        prop_assert!(crosshair.snapped_y.is_none());
        prop_assert!(crosshair.snapped_time.is_none());
        prop_assert!(crosshair.snapped_price.is_none());
    }

    #[test]
    fn wheel_zoom_keeps_anchor_stable(
        time_start in -10_000.0f64..10_000.0,
        time_span in 10.0f64..5_000.0,
        anchor_ratio in 0.05f64..0.95,
        delta_steps in -3i32..4i32,
        zoom_step_ratio in 0.01f64..0.5
    ) {
        prop_assume!(delta_steps != 0);

        let renderer = NullRenderer::default();
        let time_end = time_start + time_span;
        let config = ChartEngineConfig::new(
            Viewport::new(1200, 700),
            time_start,
            time_end,
        ).with_price_domain(0.0, 1000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        let anchor_px = 1200.0 * anchor_ratio;
        let anchor_time_before = engine.map_pixel_to_x(anchor_px).expect("anchor before");

        let wheel_delta_y = 120.0 * f64::from(delta_steps);
        let _ = engine
            .wheel_zoom_time_visible(wheel_delta_y, anchor_px, zoom_step_ratio, 1e-6)
            .expect("wheel zoom");

        let anchor_time_after = engine.map_pixel_to_x(anchor_px).expect("anchor after");
        prop_assert!((anchor_time_after - anchor_time_before).abs() <= 1e-6);
    }

    #[test]
    fn wheel_pan_preserves_span(
        time_start in -10_000.0f64..10_000.0,
        time_span in 10.0f64..5_000.0,
        delta_steps in -5i32..6i32,
        pan_step_ratio in 0.01f64..0.5
    ) {
        let renderer = NullRenderer::default();
        let time_end = time_start + time_span;
        let config = ChartEngineConfig::new(
            Viewport::new(1200, 700),
            time_start,
            time_end,
        ).with_price_domain(0.0, 1000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        let (start_before, end_before) = engine.time_visible_range();
        let span_before = end_before - start_before;

        let wheel_delta = 120.0 * f64::from(delta_steps);
        let delta_time = engine
            .wheel_pan_time_visible(wheel_delta, pan_step_ratio)
            .expect("wheel pan");
        let (start_after, end_after) = engine.time_visible_range();
        let span_after = end_after - start_after;

        let expected_delta = f64::from(delta_steps) * span_before * pan_step_ratio;
        prop_assert!((delta_time - expected_delta).abs() <= 1e-7);
        prop_assert!((span_after - span_before).abs() <= 1e-7);
        prop_assert!((start_after - (start_before + expected_delta)).abs() <= 1e-7);
        prop_assert!((end_after - (end_before + expected_delta)).abs() <= 1e-7);
    }

    #[test]
    fn crosshair_formatter_lifecycle_snapshot_is_deterministic_and_roundtrips(
        set_time_legacy in any::<bool>(),
        set_time_context in any::<bool>(),
        clear_time in any::<bool>(),
        set_price_legacy in any::<bool>(),
        set_price_context in any::<bool>(),
        clear_price in any::<bool>(),
        switch_crosshair_mode in any::<bool>(),
        change_visible_range in any::<bool>(),
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1200, 700), 0.0, 100.0)
            .with_price_domain(-2_000.0, 2_000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        if set_time_legacy {
            engine.set_crosshair_time_label_formatter(Arc::new(|value| format!("TL:{value:.2}")));
        }
        if set_time_context {
            engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, context| {
                format!("TC:{value:.2}:{:.1}", context.visible_span_abs)
            }));
        }
        if clear_time {
            if engine.crosshair_time_label_formatter_override_mode()
                == chart_rs::api::CrosshairFormatterOverrideMode::Context
            {
                engine.clear_crosshair_time_label_formatter_with_context();
            } else {
                engine.clear_crosshair_time_label_formatter();
            }
        }

        if set_price_legacy {
            engine.set_crosshair_price_label_formatter(Arc::new(|value| format!("PL:{value:.2}")));
        }
        if set_price_context {
            engine.set_crosshair_price_label_formatter_with_context(Arc::new(|value, context| {
                format!("PC:{value:.2}:{:.1}", context.visible_span_abs)
            }));
        }
        if clear_price {
            if engine.crosshair_price_label_formatter_override_mode()
                == chart_rs::api::CrosshairFormatterOverrideMode::Context
            {
                engine.clear_crosshair_price_label_formatter_with_context();
            } else {
                engine.clear_crosshair_price_label_formatter();
            }
        }

        if switch_crosshair_mode {
            engine.set_crosshair_mode(CrosshairMode::Normal);
        } else {
            engine.set_crosshair_mode(CrosshairMode::Magnet);
        }
        if change_visible_range {
            engine
                .set_time_visible_range(10.0, 80.0)
                .expect("set visible range");
        }

        let first = engine.snapshot(6.0).expect("first snapshot");
        let second = engine.snapshot(6.0).expect("second snapshot");
        prop_assert_eq!(first, second);

        prop_assert_eq!(
            first.crosshair_formatter.time_override_mode,
            engine.crosshair_time_label_formatter_override_mode()
        );
        prop_assert_eq!(
            first.crosshair_formatter.price_override_mode,
            engine.crosshair_price_label_formatter_override_mode()
        );
        let (time_gen, price_gen) = engine.crosshair_label_formatter_generations();
        prop_assert_eq!(first.crosshair_formatter.time_formatter_generation, time_gen);
        prop_assert_eq!(first.crosshair_formatter.price_formatter_generation, price_gen);

        let json = engine.snapshot_json_pretty(6.0).expect("snapshot json");
        let restored: EngineSnapshot = serde_json::from_str(&json).expect("snapshot roundtrip");
        prop_assert_eq!(restored.crosshair_formatter, first.crosshair_formatter);
    }

    #[test]
    fn crosshair_snapshot_and_diagnostics_contract_stay_coherent(
        set_time_legacy in any::<bool>(),
        set_time_context in any::<bool>(),
        clear_time in any::<bool>(),
        set_price_legacy in any::<bool>(),
        set_price_context in any::<bool>(),
        clear_price in any::<bool>(),
        switch_crosshair_mode in any::<bool>(),
        change_visible_range in any::<bool>(),
        render_once in any::<bool>(),
        clear_caches in any::<bool>(),
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1200, 700), 0.0, 100.0)
            .with_price_domain(-2_000.0, 2_000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        if set_time_legacy {
            engine.set_crosshair_time_label_formatter(Arc::new(|value| format!("TL:{value:.2}")));
        }
        if set_time_context {
            engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, context| {
                format!("TC:{value:.2}:{:.1}", context.visible_span_abs)
            }));
        }
        if clear_time {
            if engine.crosshair_time_label_formatter_override_mode()
                == chart_rs::api::CrosshairFormatterOverrideMode::Context
            {
                engine.clear_crosshair_time_label_formatter_with_context();
            } else {
                engine.clear_crosshair_time_label_formatter();
            }
        }

        if set_price_legacy {
            engine.set_crosshair_price_label_formatter(Arc::new(|value| format!("PL:{value:.2}")));
        }
        if set_price_context {
            engine.set_crosshair_price_label_formatter_with_context(Arc::new(|value, context| {
                format!("PC:{value:.2}:{:.1}", context.visible_span_abs)
            }));
        }
        if clear_price {
            if engine.crosshair_price_label_formatter_override_mode()
                == chart_rs::api::CrosshairFormatterOverrideMode::Context
            {
                engine.clear_crosshair_price_label_formatter_with_context();
            } else {
                engine.clear_crosshair_price_label_formatter();
            }
        }

        if switch_crosshair_mode {
            engine.set_crosshair_mode(CrosshairMode::Normal);
        } else {
            engine.set_crosshair_mode(CrosshairMode::Magnet);
        }
        if change_visible_range {
            engine
                .set_time_visible_range(15.0, 90.0)
                .expect("set visible range");
        }

        if render_once {
            engine.pointer_move(440.0, 260.0);
            let _ = engine.build_render_frame().expect("build frame");
        }
        if clear_caches {
            engine.clear_crosshair_formatter_caches();
        }

        let snapshot = engine.snapshot(6.0).expect("snapshot");
        let diagnostics = engine.crosshair_formatter_diagnostics();
        prop_assert_eq!(
            snapshot.crosshair_formatter.time_override_mode,
            diagnostics.time_override_mode
        );
        prop_assert_eq!(
            snapshot.crosshair_formatter.price_override_mode,
            diagnostics.price_override_mode
        );
        prop_assert_eq!(
            snapshot.crosshair_formatter.time_formatter_generation,
            diagnostics.time_formatter_generation
        );
        prop_assert_eq!(
            snapshot.crosshair_formatter.price_formatter_generation,
            diagnostics.price_formatter_generation
        );

        if clear_caches {
            prop_assert_eq!(diagnostics.time_cache.size, 0);
            prop_assert_eq!(diagnostics.price_cache.size, 0);
        }
    }

    #[test]
    fn snapshot_json_compat_parser_keeps_crosshair_formatter_contract(
        set_time_context in any::<bool>(),
        set_price_legacy in any::<bool>(),
        clear_price in any::<bool>(),
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1200, 700), 0.0, 100.0)
            .with_price_domain(-2_000.0, 2_000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        if set_time_context {
            engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, context| {
                format!("TC:{value:.2}:{:.1}", context.visible_span_abs)
            }));
        }
        if set_price_legacy {
            engine.set_crosshair_price_label_formatter(Arc::new(|value| format!("PL:{value:.2}")));
        }
        if clear_price {
            engine.clear_crosshair_price_label_formatter();
        }

        let raw_json = engine.snapshot_json_pretty(6.0).expect("raw snapshot json");
        let contract_json = engine
            .snapshot_json_contract_v1_pretty(6.0)
            .expect("contract snapshot json");

        let raw_snapshot = EngineSnapshot::from_json_compat_str(&raw_json).expect("parse raw");
        let contract_snapshot =
            EngineSnapshot::from_json_compat_str(&contract_json).expect("parse contract");

        prop_assert_eq!(raw_snapshot.crosshair_formatter, contract_snapshot.crosshair_formatter);
    }
}
