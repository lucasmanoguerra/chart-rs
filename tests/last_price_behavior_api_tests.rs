use chart_rs::api::{ChartEngine, ChartEngineConfig, LastPriceBehavior, LastPriceSourceMode};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::NullRenderer;

#[test]
fn last_price_behavior_defaults_match_render_style_defaults() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 10.0).with_price_domain(0.0, 50.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    assert_eq!(engine.last_price_behavior(), LastPriceBehavior::default());
}

#[test]
fn set_last_price_behavior_updates_render_style_contract() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 10.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let behavior = LastPriceBehavior {
        show_line: false,
        show_label: true,
        use_trend_color: true,
        source_mode: LastPriceSourceMode::LatestVisible,
    };
    engine
        .set_last_price_behavior(behavior)
        .expect("set behavior");

    assert_eq!(engine.last_price_behavior(), behavior);
    let style = engine.render_style();
    assert!(!style.show_last_price_line);
    assert!(style.show_last_price_label);
    assert!(style.last_price_use_trend_color);
    assert_eq!(
        style.last_price_source_mode,
        LastPriceSourceMode::LatestVisible
    );
}

#[test]
fn last_price_behavior_can_hide_line_and_label_in_render_frame() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 10.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(1.0, 10.0),
        DataPoint::new(2.0, 20.0),
        DataPoint::new(3.0, 15.0),
    ]);

    engine
        .set_last_price_behavior(LastPriceBehavior {
            show_line: false,
            show_label: false,
            use_trend_color: false,
            source_mode: LastPriceSourceMode::LatestData,
        })
        .expect("set behavior");

    let style = engine.render_style();
    let frame = engine.build_render_frame().expect("build frame");

    assert!(!frame.lines.iter().any(|line| {
        line.color == style.last_price_line_color
            && line.stroke_width == style.last_price_line_width
    }));
    assert!(!frame.texts.iter().any(|text| {
        text.h_align == chart_rs::render::TextHAlign::Right
            && text.color == style.last_price_label_color
    }));
}
