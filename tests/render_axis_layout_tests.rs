use chart_rs::api::{
    ChartEngine, ChartEngineConfig, CrosshairMode, RenderStyle, TimeAxisLabelConfig,
    TimeAxisLabelPolicy,
};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::{Color, NullRenderer, TextHAlign};

#[test]
fn narrow_viewport_uses_collision_safe_axis_labels() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(180, 120), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            policy: TimeAxisLabelPolicy::LogicalDecimal { precision: 0 },
            ..TimeAxisLabelConfig::default()
        })
        .expect("set time-axis config");
    let frame = engine.build_render_frame().expect("build frame");

    let mut time_xs: Vec<f64> = frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .map(|label| label.x)
        .collect();
    let mut price_ys: Vec<f64> = frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Right)
        .map(|label| label.y + 8.0)
        .collect();

    time_xs.sort_by(f64::total_cmp);
    price_ys.sort_by(f64::total_cmp);

    assert!(time_xs.windows(2).all(|pair| pair[1] - pair[0] >= 56.0));
    assert!(price_ys.windows(2).all(|pair| pair[1] - pair[0] >= 22.0));
    assert!(
        time_xs.len() <= 4,
        "time labels should be compact on narrow views"
    );
    assert!(
        price_ys.len() <= 5,
        "price labels should be compact on narrow views"
    );
}

#[test]
fn wide_viewport_produces_more_labels_than_narrow_viewport() {
    let narrow = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(180, 120), 0.0, 100.0).with_price_domain(0.0, 50.0),
    )
    .expect("narrow engine");
    let wide = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(1200, 900), 0.0, 100.0).with_price_domain(0.0, 50.0),
    )
    .expect("wide engine");

    let narrow_frame = narrow.build_render_frame().expect("narrow frame");
    let wide_frame = wide.build_render_frame().expect("wide frame");

    let narrow_time = narrow_frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .count();
    let wide_time = wide_frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .count();

    let narrow_price = narrow_frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Right)
        .count();
    let wide_price = wide_frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Right)
        .count();

    assert!(wide_time > narrow_time);
    assert!(wide_price > narrow_price);
}

#[test]
fn price_axis_and_last_price_labels_stay_inside_plot_section() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(10.0, 10.0),
        DataPoint::new(20.0, 25.0),
        DataPoint::new(40.0, 15.0),
    ]);

    let frame = engine.build_render_frame().expect("build frame");
    let style = engine.render_style();
    let viewport_height = f64::from(engine.viewport().height);
    let plot_bottom = (viewport_height - style.time_axis_height_px).clamp(0.0, viewport_height);

    let right_aligned_labels: Vec<_> = frame
        .texts
        .iter()
        .filter(|text| text.h_align == TextHAlign::Right)
        .collect();
    assert!(
        !right_aligned_labels.is_empty(),
        "expected price-axis labels"
    );
    assert!(
        right_aligned_labels
            .iter()
            .all(|text| { text.y >= 0.0 && text.y + text.font_size_px <= plot_bottom + 1e-9 })
    );
}

#[test]
fn crosshair_label_boxes_respect_section_bounds_even_with_allow_overflow() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(10.0, 10.0),
        DataPoint::new(20.0, 25.0),
        DataPoint::new(40.0, 15.0),
    ]);
    engine.set_crosshair_mode(CrosshairMode::Normal);
    engine.pointer_move(860.0, 395.0);

    let style = RenderStyle {
        crosshair_time_label_box_color: Some(Color::rgb(0.89, 0.28, 0.20)),
        crosshair_price_label_box_color: Some(Color::rgb(0.21, 0.44, 0.89)),
        crosshair_time_label_box_overflow_policy: Some(
            chart_rs::api::CrosshairLabelBoxOverflowPolicy::AllowOverflow,
        ),
        crosshair_price_label_box_overflow_policy: Some(
            chart_rs::api::CrosshairLabelBoxOverflowPolicy::AllowOverflow,
        ),
        crosshair_time_label_box_padding_y_px: 18.0,
        crosshair_price_label_box_padding_y_px: 18.0,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set render style");

    let frame = engine.build_render_frame().expect("build frame");
    let viewport_height = f64::from(engine.viewport().height);
    let plot_bottom = (viewport_height - style.time_axis_height_px).clamp(0.0, viewport_height);

    let time_rect = frame
        .rects
        .iter()
        .find(|rect| rect.fill_color == style.crosshair_time_label_box_color.expect("time color"))
        .expect("time crosshair label box");
    let price_rect = frame
        .rects
        .iter()
        .find(|rect| rect.fill_color == style.crosshair_price_label_box_color.expect("price color"))
        .expect("price crosshair label box");

    assert!(time_rect.y >= plot_bottom - 1e-9);
    assert!(time_rect.y + time_rect.height <= viewport_height + 1e-9);
    assert!(price_rect.y >= -1e-9);
    assert!(price_rect.y + price_rect.height <= plot_bottom + 1e-9);
}

#[test]
fn oversized_axis_dimensions_preserve_minimum_plot_area() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(180, 120), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let style = RenderStyle {
        price_axis_width_px: 170.0,
        time_axis_height_px: 100.0,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set render style");

    let frame = engine.build_render_frame().expect("build frame");
    let viewport_width = f64::from(engine.viewport().width);
    let viewport_height = f64::from(engine.viewport().height);
    let expected_plot_right = 80.0;
    let expected_plot_bottom = 56.0;

    let time_axis_border = frame
        .lines
        .iter()
        .find(|line| {
            line.color == style.axis_border_color
                && (line.y1 - expected_plot_bottom).abs() <= 1e-9
                && (line.y2 - expected_plot_bottom).abs() <= 1e-9
        })
        .expect("time axis border line");
    assert!((time_axis_border.x1 - 0.0).abs() <= 1e-9);
    assert!((time_axis_border.x2 - viewport_width).abs() <= 1e-9);

    let price_axis_border = frame
        .lines
        .iter()
        .find(|line| {
            line.color == style.axis_border_color
                && (line.x1 - expected_plot_right).abs() <= 1e-9
                && (line.x2 - expected_plot_right).abs() <= 1e-9
        })
        .expect("price axis border line");
    assert!((price_axis_border.y1 - 0.0).abs() <= 1e-9);
    assert!((price_axis_border.y2 - viewport_height).abs() <= 1e-9);

    let time_labels: Vec<_> = frame
        .texts
        .iter()
        .filter(|text| text.h_align == TextHAlign::Center)
        .collect();
    assert!(
        time_labels
            .iter()
            .all(|text| text.x <= expected_plot_right + 1e-9)
    );

    let price_labels: Vec<_> = frame
        .texts
        .iter()
        .filter(|text| text.h_align == TextHAlign::Right)
        .collect();
    assert!(!price_labels.is_empty(), "expected price-axis labels");
    assert!(
        price_labels
            .iter()
            .all(|text| text.y + text.font_size_px <= expected_plot_bottom + 1e-9)
    );
}

#[test]
fn tiny_viewport_clamps_axis_sections_without_negative_plot_space() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(60, 40), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let style = RenderStyle {
        price_axis_width_px: 200.0,
        time_axis_height_px: 200.0,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set render style");

    let frame = engine.build_render_frame().expect("build frame");
    let viewport_width = f64::from(engine.viewport().width);
    let viewport_height = f64::from(engine.viewport().height);

    let time_axis_border = frame
        .lines
        .iter()
        .find(|line| {
            line.color == style.axis_border_color
                && (line.y1 - viewport_height).abs() <= 1e-9
                && (line.y2 - viewport_height).abs() <= 1e-9
        })
        .expect("time axis border line");
    assert!((time_axis_border.x1 - 0.0).abs() <= 1e-9);
    assert!((time_axis_border.x2 - viewport_width).abs() <= 1e-9);

    let price_axis_border = frame
        .lines
        .iter()
        .find(|line| {
            line.color == style.axis_border_color
                && (line.x1 - viewport_width).abs() <= 1e-9
                && (line.x2 - viewport_width).abs() <= 1e-9
        })
        .expect("price axis border line");
    assert!((price_axis_border.y1 - 0.0).abs() <= 1e-9);
    assert!((price_axis_border.y2 - viewport_height).abs() <= 1e-9);
}
