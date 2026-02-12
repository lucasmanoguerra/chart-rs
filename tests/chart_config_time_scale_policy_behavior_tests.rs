use chart_rs::api::{
    ChartEngine, ChartEngineConfig, TimeScaleEdgeBehavior, TimeScaleNavigationBehavior,
    TimeScaleRealtimeAppendBehavior, TimeScaleResizeAnchor, TimeScaleResizeBehavior,
};
use chart_rs::core::Viewport;
use chart_rs::render::NullRenderer;

#[test]
fn chart_engine_config_defaults_time_scale_policy_fields() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);
    assert_eq!(
        config.time_scale_edge_behavior,
        TimeScaleEdgeBehavior::default()
    );
    assert_eq!(
        config.time_scale_resize_behavior,
        TimeScaleResizeBehavior::default()
    );
    assert_eq!(
        config.time_scale_realtime_append_behavior,
        TimeScaleRealtimeAppendBehavior::default()
    );
}

#[test]
fn chart_engine_config_applies_time_scale_policy_fields() {
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: true,
            fix_right_edge: true,
        })
        .with_time_scale_resize_behavior(TimeScaleResizeBehavior {
            lock_visible_range_on_resize: false,
            anchor: TimeScaleResizeAnchor::Left,
        })
        .with_time_scale_realtime_append_behavior(TimeScaleRealtimeAppendBehavior {
            preserve_right_edge_on_append: false,
            right_edge_tolerance_bars: 0.25,
        });
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    assert_eq!(
        engine.time_scale_edge_behavior(),
        TimeScaleEdgeBehavior {
            fix_left_edge: true,
            fix_right_edge: true,
        }
    );
    assert_eq!(
        engine.time_scale_resize_behavior(),
        TimeScaleResizeBehavior {
            lock_visible_range_on_resize: false,
            anchor: TimeScaleResizeAnchor::Left,
        }
    );
    assert_eq!(
        engine.time_scale_realtime_append_behavior(),
        TimeScaleRealtimeAppendBehavior {
            preserve_right_edge_on_append: false,
            right_edge_tolerance_bars: 0.25,
        }
    );
}

#[test]
fn chart_engine_config_time_scale_policy_composes_with_navigation_on_init() {
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
            right_offset_bars: 2.0,
            bar_spacing_px: None,
        })
        .with_time_scale_edge_behavior(TimeScaleEdgeBehavior {
            fix_left_edge: false,
            fix_right_edge: true,
        });
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    let (_, full_end) = engine.time_full_range();
    let (_, visible_end) = engine.time_visible_range();
    assert!((visible_end - full_end).abs() <= 1e-9);
}

#[test]
fn chart_engine_config_rejects_invalid_time_scale_realtime_append_policy() {
    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_time_scale_realtime_append_behavior(TimeScaleRealtimeAppendBehavior {
            preserve_right_edge_on_append: true,
            right_edge_tolerance_bars: -1.0,
        });
    let renderer = NullRenderer::default();
    match ChartEngine::new(renderer, config) {
        Ok(_) => panic!("invalid realtime append policy must fail"),
        Err(err) => assert!(matches!(err, chart_rs::ChartError::InvalidData(_))),
    }
}
