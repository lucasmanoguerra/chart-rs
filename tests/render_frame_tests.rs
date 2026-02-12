use chart_rs::api::{
    ChartEngine, ChartEngineConfig, LastPriceLabelBoxWidthMode, LastPriceSourceMode, RenderStyle,
};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::{Color, NullRenderer, TextHAlign};

#[test]
fn build_render_frame_includes_series_and_axis_primitives() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(10.0, 10.0),
        DataPoint::new(20.0, 25.0),
        DataPoint::new(40.0, 15.0),
    ]);

    let frame = engine.build_render_frame().expect("build frame");
    frame.validate().expect("valid frame");

    let time_label_count = frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .count();
    let price_label_count = frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Right)
        .count();

    assert!(frame.lines.len() >= 18, "expected series + axis lines");
    assert!(time_label_count >= 2, "time labels must be present");
    assert!(price_label_count >= 2, "price labels must be present");
}

#[test]
fn null_renderer_receives_computed_frame_counts() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 450), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(5.0, 10.0),
        DataPoint::new(15.0, 20.0),
        DataPoint::new(30.0, 15.0),
    ]);
    let frame = engine.build_render_frame().expect("build frame");

    engine.render().expect("render");
    let renderer = engine.into_renderer();

    assert_eq!(renderer.last_line_count, frame.lines.len());
    assert_eq!(renderer.last_rect_count, frame.rects.len());
    assert_eq!(renderer.last_text_count, frame.texts.len());
}

#[test]
fn last_price_marker_uses_latest_sample_value() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(1.0, 12.0),
        DataPoint::new(2.0, 16.0),
        DataPoint::new(3.0, 15.0),
    ]);

    let frame = engine.build_render_frame().expect("build frame");
    let style = engine.render_style();
    let expected_y = engine.map_price_to_pixel(15.0).expect("map").clamp(
        0.0,
        f64::from(engine.viewport().height) - style.time_axis_height_px,
    );

    assert!(frame.lines.iter().any(|line| {
        line.color == style.last_price_line_color
            && line.stroke_width == style.last_price_line_width
            && (line.y1 - expected_y).abs() <= 1e-9
            && (line.y2 - expected_y).abs() <= 1e-9
    }));
    assert!(frame.texts.iter().any(|text| {
        text.color == style.last_price_label_color
            && text.h_align == TextHAlign::Right
            && text.text == "15.00"
    }));
}

#[test]
fn last_price_label_uses_configured_vertical_offset() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(1.0, 12.0),
        DataPoint::new(2.0, 16.0),
        DataPoint::new(3.0, 15.0),
    ]);

    let style = RenderStyle {
        last_price_label_color: Color::rgb(1.0, 0.2, 0.2),
        last_price_label_offset_y_px: 13.0,
        show_last_price_label_box: false,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");

    let frame = engine.build_render_frame().expect("build frame");
    let marker_y = engine.map_price_to_pixel(15.0).expect("map").clamp(
        0.0,
        f64::from(engine.viewport().height) - style.time_axis_height_px,
    );
    let expected_text_y = (marker_y - style.last_price_label_offset_y_px).max(0.0);

    assert!(frame.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right
            && text.text == "15.00"
            && text.color == style.last_price_label_color
            && (text.y - expected_text_y).abs() <= 1e-9
    }));
}

#[test]
fn last_price_marker_can_be_disabled() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![DataPoint::new(1.0, 12.0), DataPoint::new(2.0, 16.0)]);

    let custom_style = RenderStyle {
        last_price_line_color: Color::rgb(1.0, 0.0, 0.0),
        last_price_label_color: Color::rgb(1.0, 0.0, 0.0),
        show_last_price_line: false,
        show_last_price_label: false,
        ..engine.render_style()
    };
    engine
        .set_render_style(custom_style)
        .expect("set render style");

    let frame = engine.build_render_frame().expect("build frame");
    assert!(!frame.lines.iter().any(|line| {
        line.color == custom_style.last_price_line_color
            && line.stroke_width == custom_style.last_price_line_width
    }));
    assert!(!frame.texts.iter().any(|text| {
        text.color == custom_style.last_price_label_color && text.h_align == TextHAlign::Right
    }));
}

#[test]
fn price_axis_insets_apply_to_labels_and_tick_marks() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![DataPoint::new(1.0, 10.0), DataPoint::new(2.0, 20.0)]);

    let style = RenderStyle {
        price_axis_label_padding_right_px: 14.0,
        price_axis_tick_mark_length_px: 9.0,
        price_axis_tick_mark_color: Color::rgb(0.9, 0.3, 0.2),
        price_axis_tick_mark_width: 2.25,
        show_last_price_label_box: false,
        last_price_label_color: Color::rgb(0.0, 1.0, 0.0),
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");

    let frame = engine.build_render_frame().expect("build frame");
    let viewport_width = f64::from(engine.viewport().width);
    let expected_label_x = viewport_width - style.price_axis_label_padding_right_px;
    let plot_right = (viewport_width - style.price_axis_width_px).clamp(0.0, viewport_width);
    let expected_tick_mark_end_x =
        (plot_right + style.price_axis_tick_mark_length_px).min(viewport_width);

    assert!(frame.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right
            && text.color == style.axis_label_color
            && (text.x - expected_label_x).abs() <= 1e-9
    }));
    assert!(frame.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right
            && text.color == style.last_price_label_color
            && text.text == "20.00"
            && (text.x - expected_label_x).abs() <= 1e-9
    }));
    assert!(frame.lines.iter().any(|line| {
        line.color == style.price_axis_tick_mark_color
            && line.stroke_width == style.price_axis_tick_mark_width
            && (line.y1 - line.y2).abs() <= 1e-9
            && (line.x1 - plot_right).abs() <= 1e-9
            && (line.x2 - expected_tick_mark_end_x).abs() <= 1e-9
    }));
}

#[test]
fn price_axis_labels_use_configured_font_size_and_offset() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![DataPoint::new(1.0, 10.0), DataPoint::new(2.0, 20.0)]);

    let style = RenderStyle {
        price_axis_label_font_size_px: 13.5,
        price_axis_label_offset_y_px: 11.0,
        show_last_price_label: false,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");

    let frame = engine.build_render_frame().expect("build frame");
    let axis_labels: Vec<_> = frame
        .texts
        .iter()
        .filter(|text| text.h_align == TextHAlign::Right && text.color == style.axis_label_color)
        .collect();
    assert!(!axis_labels.is_empty(), "expected price-axis labels");
    assert!(
        axis_labels
            .iter()
            .all(|text| (text.font_size_px - style.price_axis_label_font_size_px).abs() <= 1e-9)
    );
    assert!(axis_labels.iter().any(|text| {
        frame.lines.iter().any(|line| {
            line.color == style.grid_line_color
                && (line.y1 - line.y2).abs() <= 1e-9
                && ((line.y1 - text.y) - style.price_axis_label_offset_y_px).abs() <= 1e-9
        })
    }));
    assert!(axis_labels.iter().all(|text| {
        frame.lines.iter().any(|line| {
            line.color == style.grid_line_color && (line.y1 - line.y2).abs() <= 1e-9 && {
                let delta_y = line.y1 - text.y;
                delta_y >= -1e-9 && delta_y <= style.price_axis_label_offset_y_px + 1e-9
            }
        })
    }));
}

#[test]
fn last_price_label_exclusion_filters_overlapping_axis_labels() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(320, 240), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(1.0, 12.0),
        DataPoint::new(2.0, 16.0),
        DataPoint::new(3.0, 18.0),
        DataPoint::new(4.0, 17.0),
    ]);

    let no_exclusion_style = RenderStyle {
        last_price_line_color: Color::rgb(1.0, 0.0, 0.0),
        last_price_label_color: Color::rgb(1.0, 0.0, 0.0),
        show_last_price_line: false,
        show_last_price_label: true,
        last_price_label_exclusion_px: 0.0,
        ..engine.render_style()
    };
    engine
        .set_render_style(no_exclusion_style)
        .expect("set style no exclusion");
    let frame_no_exclusion = engine.build_render_frame().expect("frame no exclusion");
    let axis_labels_no_exclusion = frame_no_exclusion
        .texts
        .iter()
        .filter(|text| {
            text.h_align == TextHAlign::Right && text.color == no_exclusion_style.axis_label_color
        })
        .count();

    let strong_exclusion_style = RenderStyle {
        last_price_label_exclusion_px: 10_000.0,
        ..no_exclusion_style
    };
    engine
        .set_render_style(strong_exclusion_style)
        .expect("set style strong exclusion");
    let frame_strong_exclusion = engine.build_render_frame().expect("frame strong exclusion");
    let axis_labels_strong_exclusion = frame_strong_exclusion
        .texts
        .iter()
        .filter(|text| {
            text.h_align == TextHAlign::Right
                && text.color == strong_exclusion_style.axis_label_color
        })
        .count();

    assert!(axis_labels_strong_exclusion < axis_labels_no_exclusion);
    assert!(frame_strong_exclusion.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right
            && text.color == strong_exclusion_style.last_price_label_color
    }));
}

#[test]
fn last_price_trend_color_uses_up_color() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![DataPoint::new(1.0, 12.0), DataPoint::new(2.0, 15.0)]);
    let trend_style = RenderStyle {
        last_price_line_color: Color::rgb(0.0, 0.0, 0.0),
        last_price_label_color: Color::rgb(0.0, 0.0, 0.0),
        last_price_up_color: Color::rgb(0.0, 0.8, 0.0),
        last_price_down_color: Color::rgb(0.8, 0.0, 0.0),
        last_price_neutral_color: Color::rgb(0.0, 0.0, 0.8),
        last_price_use_trend_color: true,
        ..engine.render_style()
    };
    engine
        .set_render_style(trend_style)
        .expect("set trend style");

    let frame = engine.build_render_frame().expect("build frame");
    let expected_y = engine.map_price_to_pixel(15.0).expect("map").clamp(
        0.0,
        f64::from(engine.viewport().height) - trend_style.time_axis_height_px,
    );

    assert!(frame.lines.iter().any(|line| {
        line.color == trend_style.last_price_up_color
            && line.stroke_width == trend_style.last_price_line_width
            && (line.y1 - expected_y).abs() <= 1e-9
            && (line.y2 - expected_y).abs() <= 1e-9
    }));
    assert!(frame.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right
            && text.text == "15.00"
            && text.color == trend_style.last_price_up_color
    }));
}

#[test]
fn last_price_trend_color_uses_down_color() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![DataPoint::new(1.0, 15.0), DataPoint::new(2.0, 12.0)]);
    let trend_style = RenderStyle {
        last_price_line_color: Color::rgb(0.0, 0.0, 0.0),
        last_price_label_color: Color::rgb(0.0, 0.0, 0.0),
        last_price_up_color: Color::rgb(0.0, 0.8, 0.0),
        last_price_down_color: Color::rgb(0.8, 0.0, 0.0),
        last_price_neutral_color: Color::rgb(0.0, 0.0, 0.8),
        last_price_use_trend_color: true,
        ..engine.render_style()
    };
    engine
        .set_render_style(trend_style)
        .expect("set trend style");

    let frame = engine.build_render_frame().expect("build frame");
    let expected_y = engine.map_price_to_pixel(12.0).expect("map").clamp(
        0.0,
        f64::from(engine.viewport().height) - trend_style.time_axis_height_px,
    );

    assert!(frame.lines.iter().any(|line| {
        line.color == trend_style.last_price_down_color
            && line.stroke_width == trend_style.last_price_line_width
            && (line.y1 - expected_y).abs() <= 1e-9
            && (line.y2 - expected_y).abs() <= 1e-9
    }));
    assert!(frame.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right
            && text.text == "12.00"
            && text.color == trend_style.last_price_down_color
    }));
}

#[test]
fn last_price_trend_color_uses_neutral_without_previous_sample() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![DataPoint::new(1.0, 14.0)]);
    let trend_style = RenderStyle {
        last_price_line_color: Color::rgb(0.0, 0.0, 0.0),
        last_price_label_color: Color::rgb(0.0, 0.0, 0.0),
        last_price_up_color: Color::rgb(0.0, 0.8, 0.0),
        last_price_down_color: Color::rgb(0.8, 0.0, 0.0),
        last_price_neutral_color: Color::rgb(0.0, 0.0, 0.8),
        last_price_use_trend_color: true,
        ..engine.render_style()
    };
    engine
        .set_render_style(trend_style)
        .expect("set trend style");

    let frame = engine.build_render_frame().expect("build frame");
    let expected_y = engine.map_price_to_pixel(14.0).expect("map").clamp(
        0.0,
        f64::from(engine.viewport().height) - trend_style.time_axis_height_px,
    );

    assert!(frame.lines.iter().any(|line| {
        line.color == trend_style.last_price_neutral_color
            && line.stroke_width == trend_style.last_price_line_width
            && (line.y1 - expected_y).abs() <= 1e-9
            && (line.y2 - expected_y).abs() <= 1e-9
    }));
    assert!(frame.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right
            && text.text == "14.00"
            && text.color == trend_style.last_price_neutral_color
    }));
}

#[test]
fn last_price_source_mode_latest_visible_uses_latest_visible_sample() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 10.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(1.0, 10.0),
        DataPoint::new(2.0, 20.0),
        DataPoint::new(3.0, 30.0),
        DataPoint::new(4.0, 40.0),
    ]);
    engine
        .set_time_visible_range(1.0, 2.2)
        .expect("set visible range");

    let style = RenderStyle {
        last_price_line_color: Color::rgb(0.0, 1.0, 0.0),
        last_price_label_color: Color::rgb(0.0, 1.0, 0.0),
        last_price_source_mode: LastPriceSourceMode::LatestVisible,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");

    let frame = engine.build_render_frame().expect("build frame");
    let expected_y = engine.map_price_to_pixel(20.0).expect("map").clamp(
        0.0,
        f64::from(engine.viewport().height) - style.time_axis_height_px,
    );
    assert!(frame.lines.iter().any(|line| {
        line.color == style.last_price_line_color
            && line.stroke_width == style.last_price_line_width
            && (line.y1 - expected_y).abs() <= 1e-9
            && (line.y2 - expected_y).abs() <= 1e-9
    }));
    assert!(frame.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right
            && text.text == "20.00"
            && text.color == style.last_price_label_color
    }));
}

#[test]
fn last_price_source_mode_latest_visible_hides_marker_for_empty_visible_window() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 10.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(1.0, 10.0),
        DataPoint::new(2.0, 20.0),
        DataPoint::new(3.0, 30.0),
        DataPoint::new(4.0, 40.0),
    ]);
    engine
        .set_time_visible_range(8.0, 9.0)
        .expect("set visible range");

    let style = RenderStyle {
        last_price_line_color: Color::rgb(0.0, 1.0, 0.0),
        last_price_label_color: Color::rgb(0.0, 1.0, 0.0),
        last_price_source_mode: LastPriceSourceMode::LatestVisible,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");

    let frame = engine.build_render_frame().expect("build frame");
    assert!(!frame.lines.iter().any(|line| {
        line.color == style.last_price_line_color
            && line.stroke_width == style.last_price_line_width
    }));
    assert!(!frame.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right && text.color == style.last_price_label_color
    }));
}

#[test]
fn last_price_label_box_draws_axis_rect_and_uses_box_text_color() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![DataPoint::new(1.0, 10.0), DataPoint::new(2.0, 20.0)]);

    let style = RenderStyle {
        last_price_line_color: Color::rgb(0.1, 0.4, 1.0),
        last_price_label_color: Color::rgb(0.1, 0.4, 1.0),
        show_last_price_label_box: true,
        last_price_label_box_use_marker_color: true,
        last_price_label_box_border_width_px: 1.25,
        last_price_label_box_border_color: Color::rgb(0.8, 0.8, 0.8),
        last_price_label_box_corner_radius_px: 3.0,
        last_price_label_box_text_color: Color::rgb(1.0, 1.0, 1.0),
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");
    let frame = engine.build_render_frame().expect("build frame");
    let plot_right = (f64::from(engine.viewport().width) - style.price_axis_width_px)
        .clamp(0.0, f64::from(engine.viewport().width));

    assert!(frame.rects.iter().any(|rect| {
        rect.fill_color == style.last_price_label_color
            && (rect.x - plot_right).abs() <= 1e-9
            && rect.width > 0.0
            && rect.height > 0.0
            && rect.border_width == style.last_price_label_box_border_width_px
            && rect.border_color == style.last_price_label_box_border_color
            && rect.corner_radius == style.last_price_label_box_corner_radius_px
    }));
    assert!(frame.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right
            && text.text == "20.00"
            && text.color == style.last_price_label_box_text_color
    }));
}

#[test]
fn last_price_label_box_can_use_custom_fill_color() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![DataPoint::new(1.0, 10.0), DataPoint::new(2.0, 20.0)]);

    let style = RenderStyle {
        last_price_label_color: Color::rgb(0.0, 0.9, 0.0),
        show_last_price_label_box: true,
        last_price_label_box_use_marker_color: false,
        last_price_label_box_color: Color::rgb(0.15, 0.15, 0.15),
        last_price_label_box_text_color: Color::rgb(0.95, 0.95, 0.95),
        last_price_label_box_auto_text_contrast: false,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");
    let frame = engine.build_render_frame().expect("build frame");

    assert!(
        frame
            .rects
            .iter()
            .any(|rect| rect.fill_color == style.last_price_label_box_color)
    );
    assert!(frame.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right
            && text.text == "20.00"
            && text.color == style.last_price_label_box_text_color
    }));
}

#[test]
fn last_price_label_box_auto_text_contrast_switches_to_dark_text_on_bright_fill() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![DataPoint::new(1.0, 10.0), DataPoint::new(2.0, 20.0)]);

    let style = RenderStyle {
        show_last_price_label_box: true,
        last_price_label_box_use_marker_color: false,
        last_price_label_box_color: Color::rgb(0.95, 0.95, 0.95),
        last_price_label_box_text_color: Color::rgb(1.0, 0.0, 0.0),
        last_price_label_box_auto_text_contrast: true,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");
    let frame = engine.build_render_frame().expect("build frame");

    assert!(frame.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right
            && text.text == "20.00"
            && text.color == Color::rgb(0.06, 0.08, 0.11)
    }));
}

#[test]
fn last_price_label_box_corner_radius_is_clamped_to_box_size() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(240, 160), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![DataPoint::new(1.0, 10.0), DataPoint::new(2.0, 20.0)]);

    let style = RenderStyle {
        show_last_price_label_box: true,
        last_price_label_box_use_marker_color: false,
        last_price_label_box_color: Color::rgb(0.1, 0.1, 0.1),
        last_price_label_box_corner_radius_px: 10_000.0,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");
    let frame = engine.build_render_frame().expect("build frame");

    assert!(frame.rects.iter().any(|rect| {
        rect.corner_radius <= (rect.width.min(rect.height)) * 0.5 + 1e-9
            && rect.corner_radius >= 0.0
    }));
}

#[test]
fn last_price_label_box_fit_text_respects_min_width_and_padding() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![DataPoint::new(1.0, 10.0), DataPoint::new(2.0, 20.0)]);

    let style = RenderStyle {
        price_axis_width_px: 120.0,
        show_last_price_label_box: true,
        last_price_label_box_width_mode: LastPriceLabelBoxWidthMode::FitText,
        last_price_label_box_padding_x_px: 10.0,
        last_price_label_box_min_width_px: 80.0,
        last_price_label_box_use_marker_color: false,
        last_price_label_box_color: Color::rgb(0.12, 0.12, 0.12),
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set style");
    let frame = engine.build_render_frame().expect("build frame");

    let expected_box_width = 80.0;
    let expected_text_x =
        f64::from(engine.viewport().width) - style.last_price_label_box_padding_x_px;
    assert!(frame.rects.iter().any(|rect| {
        (rect.width - expected_box_width).abs() <= 1e-9
            && rect.width < style.price_axis_width_px
            && rect.fill_color == style.last_price_label_box_color
    }));
    assert!(frame.texts.iter().any(|text| {
        text.h_align == TextHAlign::Right
            && text.text == "20.00"
            && (text.x - expected_text_x).abs() <= 1e-9
    }));
}
