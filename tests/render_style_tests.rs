use chart_rs::ChartError;
use chart_rs::api::{
    AxisLabelLocale, ChartEngine, ChartEngineConfig, LastPriceLabelBoxWidthMode,
    LastPriceSourceMode, RenderStyle, TimeAxisLabelConfig, TimeAxisLabelPolicy,
    TimeAxisSessionConfig, TimeAxisTimeZone,
};
use chart_rs::core::Viewport;
use chart_rs::render::{Color, NullRenderer};

#[test]
fn default_render_style_produces_grid_and_axis_lines() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");

    let style = engine.render_style();
    let frame = engine.build_render_frame().expect("frame");

    let grid_lines = frame
        .lines
        .iter()
        .filter(|line| line.color == style.grid_line_color)
        .count();
    let axis_lines = frame
        .lines
        .iter()
        .filter(|line| line.color == style.axis_border_color)
        .count();

    assert!(grid_lines >= 4);
    assert!(axis_lines >= 4);
}

#[test]
fn custom_render_style_is_applied_to_frame() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 86_400.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let custom_style = RenderStyle {
        series_line_color: Color::rgb(0.9, 0.2, 0.2),
        grid_line_color: Color::rgb(0.1, 0.7, 0.4),
        major_grid_line_color: Color::rgb(0.8, 0.4, 0.1),
        axis_border_color: Color::rgb(0.2, 0.2, 0.2),
        price_axis_tick_mark_color: Color::rgb(0.7, 0.2, 0.5),
        axis_label_color: Color::rgb(0.0, 0.0, 0.0),
        last_price_line_color: Color::rgb(0.2, 0.2, 0.8),
        last_price_label_color: Color::rgb(0.2, 0.2, 0.8),
        last_price_up_color: Color::rgb(0.1, 0.7, 0.3),
        last_price_down_color: Color::rgb(0.9, 0.2, 0.2),
        last_price_neutral_color: Color::rgb(0.2, 0.2, 0.8),
        grid_line_width: 2.0,
        major_grid_line_width: 3.0,
        axis_line_width: 1.5,
        price_axis_tick_mark_width: 1.25,
        last_price_line_width: 1.75,
        major_time_label_font_size_px: 13.0,
        last_price_label_font_size_px: 12.0,
        price_axis_width_px: 84.0,
        time_axis_height_px: 28.0,
        price_axis_label_padding_right_px: 7.0,
        price_axis_tick_mark_length_px: 8.0,
        show_last_price_line: true,
        show_last_price_label: true,
        last_price_use_trend_color: true,
        last_price_source_mode: LastPriceSourceMode::LatestData,
        show_last_price_label_box: true,
        last_price_label_box_use_marker_color: false,
        last_price_label_box_color: Color::rgb(0.1, 0.1, 0.1),
        last_price_label_box_text_color: Color::rgb(0.95, 0.95, 0.95),
        last_price_label_box_auto_text_contrast: false,
        last_price_label_box_width_mode: LastPriceLabelBoxWidthMode::FitText,
        last_price_label_box_padding_x_px: 8.0,
        last_price_label_box_padding_y_px: 3.5,
        last_price_label_box_min_width_px: 56.0,
        last_price_label_box_border_width_px: 1.5,
        last_price_label_box_border_color: Color::rgb(0.85, 0.85, 0.85),
        last_price_label_box_corner_radius_px: 4.0,
        last_price_label_exclusion_px: 24.0,
    };
    engine
        .set_render_style(custom_style)
        .expect("set render style");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: TimeAxisLabelPolicy::UtcDateTime {
                show_seconds: false,
            },
            ..TimeAxisLabelConfig::default()
        })
        .expect("set time axis policy");

    let frame = engine.build_render_frame().expect("frame");
    assert!(
        frame
            .lines
            .iter()
            .any(|line| line.color == custom_style.grid_line_color && line.stroke_width == 2.0)
    );
    assert!(
        frame
            .lines
            .iter()
            .any(|line| line.color == custom_style.axis_border_color && line.stroke_width == 1.5)
    );
    assert!(
        frame.lines.iter().any(
            |line| line.color == custom_style.major_grid_line_color && line.stroke_width == 3.0
        )
    );
    assert!(frame.lines.iter().any(|line| {
        line.color == custom_style.price_axis_tick_mark_color
            && line.stroke_width == custom_style.price_axis_tick_mark_width
    }));
}

#[test]
fn invalid_render_style_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.grid_line_width = 0.0;

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_last_price_style_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.last_price_line_width = 0.0;

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_price_axis_tick_mark_width_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.price_axis_tick_mark_width = 0.0;

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_price_axis_tick_mark_color_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.price_axis_tick_mark_color = Color::rgb(1.1, 0.2, 0.2);

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_last_price_label_exclusion_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.last_price_label_exclusion_px = -1.0;

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_price_axis_label_padding_right_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.price_axis_label_padding_right_px = -1.0;

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_price_axis_tick_mark_length_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.price_axis_tick_mark_length_px = -1.0;

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_last_price_trend_color_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.last_price_up_color = Color::rgb(1.2, 0.2, 0.2);

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_last_price_label_box_padding_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.last_price_label_box_padding_y_px = -1.0;

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_last_price_label_box_color_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.last_price_label_box_color = Color::rgb(-0.1, 0.2, 0.2);

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_last_price_label_box_border_width_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.last_price_label_box_border_width_px = -0.5;

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_last_price_label_box_corner_radius_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.last_price_label_box_corner_radius_px = -1.0;

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_last_price_label_box_padding_x_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.last_price_label_box_padding_x_px = -1.0;

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_last_price_label_box_min_width_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let mut style = engine.render_style();
    style.last_price_label_box_min_width_px = 0.0;

    let err = engine
        .set_render_style(style)
        .expect_err("invalid style should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn session_boundary_uses_major_tick_styling() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(900, 420), 1_704_205_800.0, 1_704_206_100.0)
        .with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let custom_style = RenderStyle {
        major_grid_line_color: Color::rgb(0.75, 0.35, 0.12),
        major_grid_line_width: 2.5,
        major_time_label_font_size_px: 14.0,
        ..engine.render_style()
    };
    engine
        .set_render_style(custom_style)
        .expect("set custom render style");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: TimeAxisLabelPolicy::UtcDateTime {
                show_seconds: false,
            },
            timezone: TimeAxisTimeZone::FixedOffsetMinutes { minutes: -300 },
            session: Some(TimeAxisSessionConfig {
                start_hour: 9,
                start_minute: 30,
                end_hour: 16,
                end_minute: 0,
            }),
        })
        .expect("set session/time-axis config");

    let frame = engine.build_render_frame().expect("frame");
    assert!(
        frame
            .lines
            .iter()
            .any(|line| line.color == custom_style.major_grid_line_color
                && line.stroke_width == custom_style.major_grid_line_width)
    );
    assert!(
        frame
            .texts
            .iter()
            .any(|text| text.text == "2024-01-02 09:30"
                && text.font_size_px == custom_style.major_time_label_font_size_px)
    );
}
