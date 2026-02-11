use chart_rs::api::{ChartEngine, ChartEngineConfig, RenderStyle};
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
