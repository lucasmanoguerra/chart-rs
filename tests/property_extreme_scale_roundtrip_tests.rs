use chart_rs::api::{
    ChartEngine, ChartEngineConfig, PriceScaleMarginBehavior, PriceScaleTransformedBaseBehavior,
    PriceScaleTransformedBaseSource, TimeCoordinateIndexPolicy, TimeScaleNavigationBehavior,
    TimeScaleZoomLimitBehavior,
};
use chart_rs::core::{DataPoint, PriceScaleMode, Viewport};
use chart_rs::render::NullRenderer;
use proptest::prelude::*;

fn build_engine(viewport: Viewport) -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(viewport, 0.0, 100.0).with_price_domain(50.0, 150.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(0.0, 90.0),
        DataPoint::new(10.0, 94.0),
        DataPoint::new(20.0, 101.0),
        DataPoint::new(30.0, 98.0),
        DataPoint::new(40.0, 110.0),
        DataPoint::new(50.0, 104.0),
        DataPoint::new(60.0, 120.0),
        DataPoint::new(70.0, 126.0),
        DataPoint::new(80.0, 118.0),
        DataPoint::new(90.0, 132.0),
        DataPoint::new(100.0, 140.0),
    ]);
    engine
}

fn transformed_source(code: u8) -> PriceScaleTransformedBaseSource {
    match code % 5 {
        0 => PriceScaleTransformedBaseSource::DomainStart,
        1 => PriceScaleTransformedBaseSource::FirstData,
        2 => PriceScaleTransformedBaseSource::LastData,
        3 => PriceScaleTransformedBaseSource::FirstVisibleData,
        _ => PriceScaleTransformedBaseSource::LastVisibleData,
    }
}

fn select_mode(code: u8) -> PriceScaleMode {
    match code % 4 {
        0 => PriceScaleMode::Linear,
        1 => PriceScaleMode::Log,
        2 => PriceScaleMode::Percentage,
        _ => PriceScaleMode::IndexedTo100,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(56))]

    #[test]
    fn price_roundtrip_stays_stable_under_transformed_modes_margins_and_resize(
        width in 320u32..2200u32,
        height in 180u32..1400u32,
        resized_width in 320u32..2200u32,
        resized_height in 180u32..1400u32,
        top_margin_ratio in 0.0f64..0.45,
        bottom_margin_ratio in 0.0f64..0.45,
        mode_code in 0u8..4u8,
        source_code in 0u8..5u8,
        explicit_base in prop::option::of(60.0f64..140.0),
        visible_start in 0.0f64..60.0,
        visible_span in 10.0f64..100.0,
        probe_price in 60.0f64..140.0,
        probe_pixel_ratio in 0.0f64..1.0,
    ) {
        prop_assume!(top_margin_ratio + bottom_margin_ratio < 0.9);

        let mut engine = build_engine(Viewport::new(width, height));
        engine
            .set_price_scale_margin_behavior(PriceScaleMarginBehavior {
                top_margin_ratio,
                bottom_margin_ratio,
            })
            .expect("set price margin behavior");

        let mode = select_mode(mode_code);
        engine
            .set_price_scale_mode(mode)
            .expect("set price scale mode");
        if matches!(mode, PriceScaleMode::Percentage | PriceScaleMode::IndexedTo100) {
            engine
                .set_price_scale_transformed_base_behavior(PriceScaleTransformedBaseBehavior {
                    explicit_base_price: explicit_base,
                    dynamic_source: transformed_source(source_code),
                })
                .expect("set transformed base behavior");
        }

        let visible_end = (visible_start + visible_span).min(100.0);
        engine
            .set_time_visible_range(visible_start, visible_end)
            .expect("set time visible range");
        engine
            .autoscale_price_from_visible_data()
            .expect("autoscale visible data");
        engine
            .set_viewport(Viewport::new(resized_width, resized_height))
            .expect("set resized viewport");

        let px = engine.map_price_to_pixel(probe_price).expect("price to pixel");
        let recovered_price = engine.map_pixel_to_price(px).expect("pixel to price");
        prop_assert!(px.is_finite());
        prop_assert!(recovered_price.is_finite());

        let price_tolerance = 1e-6 * probe_price.abs().max(1.0) + 1e-6;
        prop_assert!((recovered_price - probe_price).abs() <= price_tolerance);

        let probe_pixel = probe_pixel_ratio * f64::from(resized_height);
        let mapped_price = engine
            .map_pixel_to_price(probe_pixel)
            .expect("pixel to price on arbitrary pixel");
        let mapped_pixel_back = engine
            .map_price_to_pixel(mapped_price)
            .expect("price to pixel roundtrip");
        let pixel_tolerance = 1e-6 * f64::from(resized_height).max(1.0) + 1e-6;
        prop_assert!((mapped_pixel_back - probe_pixel).abs() <= pixel_tolerance);
    }

    #[test]
    fn time_roundtrip_and_logical_mapping_stay_stable_under_right_offset_zoom_resize(
        right_offset_px in 0.0f64..320.0,
        min_spacing in 0.5f64..25.0,
        max_spacing_raw in prop::option::of(0.5f64..80.0),
        operation_steps in prop::collection::vec(
            (0u8..4, -360.0f64..360.0, -360.0f64..360.0, 320u32..2200u32, 180u32..1400u32),
            1..32,
        ),
    ) {
        let mut engine = build_engine(Viewport::new(1000, 500));
        let max_spacing = max_spacing_raw.map(|value| value.max(min_spacing));

        engine
            .set_time_scale_zoom_limit_behavior(TimeScaleZoomLimitBehavior {
                min_bar_spacing_px: min_spacing,
                max_bar_spacing_px: max_spacing,
            })
            .expect("set time zoom limits");
        engine
            .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
                right_offset_bars: 0.0,
                bar_spacing_px: Some(6.0),
            })
            .expect("set navigation");
        engine
            .set_time_scale_right_offset_px(Some(right_offset_px))
            .expect("set right offset px");

        for (kind, v0, v1, width, height) in operation_steps {
            match kind % 4 {
                0 => {
                    let wheel_delta_y = if v0.abs() <= 1e-9 {
                        120.0
                    } else {
                        v0
                    };
                    let anchor_px = (v1.abs() % f64::from(engine.viewport().width)).max(0.0);
                    let _ = engine
                        .wheel_zoom_time_visible(wheel_delta_y, anchor_px, 0.2, 1e-6)
                        .expect("wheel zoom");
                }
                1 => {
                    let factor = (v0.abs() / 240.0).clamp(0.25, 4.0);
                    let anchor_px = (v1.abs() % f64::from(engine.viewport().width)).max(0.0);
                    let _ = engine
                        .pinch_zoom_time_visible(factor, anchor_px, 1e-6)
                        .expect("pinch zoom");
                }
                2 => {
                    engine
                        .set_viewport(Viewport::new(width, height))
                        .expect("resize viewport");
                }
                _ => {
                    let wheel_delta_x = if v0.abs() <= 1e-9 {
                        120.0
                    } else {
                        v0
                    };
                    let _ = engine
                        .wheel_pan_time_visible(wheel_delta_x, 0.1)
                        .expect("wheel pan");
                }
            }

            let (start, end) = engine.time_visible_range();
            prop_assert!(start.is_finite());
            prop_assert!(end.is_finite());
            prop_assert!(end > start);

            let probe_px = (v1.abs() % f64::from(engine.viewport().width)).max(0.0);
            let probe_time = engine.map_pixel_to_x(probe_px).expect("pixel to time");
            let probe_px_back = engine.map_x_to_pixel(probe_time).expect("time to pixel");
            let width_tolerance = 1e-6 * f64::from(engine.viewport().width).max(1.0) + 1e-6;
            prop_assert!((probe_px_back - probe_px).abs() <= width_tolerance);

            let logical = engine
                .map_pixel_to_logical_index(probe_px, TimeCoordinateIndexPolicy::AllowWhitespace)
                .expect("pixel to logical index")
                .expect("logical index should resolve");
            let logical_px = engine
                .map_logical_index_to_pixel(logical)
                .expect("logical index to pixel")
                .expect("logical pixel should resolve");
            prop_assert!((logical_px - probe_px).abs() <= width_tolerance);
        }
    }
}
