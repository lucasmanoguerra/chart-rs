use chart_rs::api::{
    ChartEngine, ChartEngineConfig, CrosshairMode, RenderStyle, TimeAxisLabelConfig,
    TimeAxisLabelPolicy, TimeAxisSessionConfig, TimeAxisTimeZone,
};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::{Color, NullRenderer, TextHAlign};

fn sorted_time_label_xs(frame: &chart_rs::render::RenderFrame) -> Vec<f64> {
    let mut xs: Vec<f64> = frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .map(|label| label.x)
        .collect();
    xs.sort_by(f64::total_cmp);
    xs
}

fn sorted_price_label_ys(frame: &chart_rs::render::RenderFrame) -> Vec<f64> {
    let mut ys: Vec<f64> = frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Right)
        .map(|label| label.y + 8.0)
        .collect();
    ys.sort_by(f64::total_cmp);
    ys
}

fn time_label_count(frame: &chart_rs::render::RenderFrame) -> usize {
    frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .count()
}

fn price_label_count(frame: &chart_rs::render::RenderFrame) -> usize {
    frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Right)
        .count()
}

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

    let time_xs = sorted_time_label_xs(&frame);
    let price_ys = sorted_price_label_ys(&frame);

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

#[test]
fn adaptive_price_axis_width_expands_for_large_price_labels() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(420, 220), 0.0, 10.0)
        .with_price_domain(0.0, 1_000_000.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(0.0, 1_100_000.0),
        DataPoint::new(1.0, 1_200_000.0),
        DataPoint::new(2.0, 1_300_000.0),
    ]);

    let style = RenderStyle {
        price_axis_width_px: 28.0,
        price_axis_label_font_size_px: 18.0,
        last_price_label_font_size_px: 18.0,
        ..engine.render_style()
    };
    engine.set_render_style(style).expect("set render style");

    let frame = engine.build_render_frame().expect("build frame");
    let viewport_width = f64::from(engine.viewport().width);
    let viewport_height = f64::from(engine.viewport().height);
    let price_axis_border = frame
        .lines
        .iter()
        .find(|line| {
            line.color == style.axis_border_color
                && (line.x1 - line.x2).abs() <= 1e-9
                && (line.y1 - 0.0).abs() <= 1e-9
                && (line.y2 - viewport_height).abs() <= 1e-9
        })
        .expect("price axis border line");

    let effective_width = viewport_width - price_axis_border.x1;
    assert!(effective_width > style.price_axis_width_px);
}

#[test]
fn adaptive_time_axis_height_expands_for_large_time_axis_typography() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(420, 220), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            policy: TimeAxisLabelPolicy::LogicalDecimal { precision: 0 },
            ..TimeAxisLabelConfig::default()
        })
        .expect("set time-axis config");
    let style = RenderStyle {
        time_axis_height_px: 10.0,
        time_axis_label_font_size_px: 20.0,
        major_time_label_font_size_px: 22.0,
        time_axis_label_offset_y_px: 8.0,
        major_time_label_offset_y_px: 10.0,
        show_time_axis_tick_marks: true,
        time_axis_tick_mark_length_px: 9.0,
        show_major_time_tick_marks: true,
        major_time_tick_mark_length_px: 11.0,
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
                && (line.y1 - line.y2).abs() <= 1e-9
                && (line.x1 - 0.0).abs() <= 1e-9
                && (line.x2 - viewport_width).abs() <= 1e-9
        })
        .expect("time axis border line");

    let effective_height = viewport_height - time_axis_border.y1;
    assert!(effective_height > style.time_axis_height_px);
}

#[test]
fn time_axis_labels_stay_collision_safe_under_zoom_and_pan() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(1200, 420), 0.0, 2_000.0)
        .with_price_domain(0.0, 200.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            policy: TimeAxisLabelPolicy::LogicalDecimal { precision: 0 },
            ..TimeAxisLabelConfig::default()
        })
        .expect("set logical time labels");
    let points: Vec<DataPoint> = (0..2_000)
        .map(|i| {
            let t = i as f64;
            DataPoint::new(t, 100.0 + (t * 0.05).sin() * 20.0)
        })
        .collect();
    engine.set_data(points);

    // Wide span (zoomed out).
    engine
        .set_time_visible_range(0.0, 2_000.0)
        .expect("set wide range");
    let wide_frame = engine.build_render_frame().expect("wide frame");
    let wide_time_xs = sorted_time_label_xs(&wide_frame);
    assert!(!wide_time_xs.is_empty());
    assert!(
        wide_time_xs
            .windows(2)
            .all(|pair| pair[1] - pair[0] >= 56.0)
    );

    // Narrow span (zoomed in) with pan offset.
    engine
        .set_time_visible_range(800.0, 1_000.0)
        .expect("set narrow range");
    engine.pan_time_visible_by(75.0).expect("pan narrow range");
    let zoomed_frame = engine.build_render_frame().expect("zoomed frame");
    let zoomed_time_xs = sorted_time_label_xs(&zoomed_frame);
    assert!(!zoomed_time_xs.is_empty());
    assert!(
        zoomed_time_xs
            .windows(2)
            .all(|pair| pair[1] - pair[0] >= 56.0)
    );
}

#[test]
fn time_axis_tick_density_changes_with_zoom_level() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(1200, 420), 0.0, 2_000.0)
        .with_price_domain(0.0, 200.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            policy: TimeAxisLabelPolicy::LogicalDecimal { precision: 0 },
            ..TimeAxisLabelConfig::default()
        })
        .expect("set logical time labels");
    let points: Vec<DataPoint> = (0..2_000)
        .map(|i| {
            let t = i as f64;
            DataPoint::new(t, 100.0 + (t * 0.1).sin() * 10.0)
        })
        .collect();
    engine.set_data(points);

    engine
        .set_time_visible_range(0.0, 2_000.0)
        .expect("zoomed-out range");
    let zoomed_out = engine.build_render_frame().expect("zoomed-out frame");
    let zoomed_out_count = time_label_count(&zoomed_out);

    engine
        .set_time_visible_range(600.0, 1_400.0)
        .expect("mid zoom range");
    let mid_zoom = engine.build_render_frame().expect("mid zoom frame");
    let mid_zoom_count = time_label_count(&mid_zoom);

    engine
        .set_time_visible_range(940.0, 1_020.0)
        .expect("zoomed-in range");
    let zoomed_in = engine.build_render_frame().expect("zoomed-in frame");
    let zoomed_in_count = time_label_count(&zoomed_in);

    assert!(zoomed_out_count < mid_zoom_count);
    assert!(mid_zoom_count <= zoomed_in_count);
    assert!(zoomed_in_count >= zoomed_out_count + 3);
}

#[test]
fn time_axis_label_spacing_remains_reasonably_even_after_zoom_changes() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(1200, 420), 0.0, 2_000.0)
        .with_price_domain(0.0, 200.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            policy: TimeAxisLabelPolicy::LogicalDecimal { precision: 0 },
            ..TimeAxisLabelConfig::default()
        })
        .expect("set logical time labels");

    engine
        .set_time_visible_range(900.0, 1_020.0)
        .expect("set zoomed range");
    let frame = engine.build_render_frame().expect("build frame");
    let xs = sorted_time_label_xs(&frame);
    assert!(xs.len() >= 3);

    let mut deltas = xs.windows(2).map(|pair| pair[1] - pair[0]);
    let first = deltas.next().expect("at least one delta");
    let (min_delta, max_delta) = deltas.fold((first, first), |(min_d, max_d), delta| {
        (min_d.min(delta), max_d.max(delta))
    });
    assert!(min_delta >= 56.0);
    assert!(
        max_delta <= min_delta * 1.7,
        "time-axis label spacing should stay approximately even (min={min_delta}, max={max_delta})"
    );
}

#[test]
fn high_precision_logical_time_labels_keep_readable_cadence() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(920, 360), 12_345.0, 12_350.0)
        .with_price_domain(0.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            policy: TimeAxisLabelPolicy::LogicalDecimal { precision: 10 },
            ..TimeAxisLabelConfig::default()
        })
        .expect("set high-precision logical labels");

    let frame = engine.build_render_frame().expect("frame");
    let time_xs = sorted_time_label_xs(&frame);
    assert!(time_xs.len() >= 5, "expected readable time-label cadence");
    assert!(time_xs.windows(2).all(|pair| pair[1] - pair[0] >= 56.0));
}

#[test]
fn price_axis_labels_stay_collision_safe_after_vertical_axis_drag_scaling() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 200.0).with_price_domain(10.0, 30.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(0.0, 12.0),
        DataPoint::new(1.0, 15.0),
        DataPoint::new(2.0, 22.0),
        DataPoint::new(3.0, 28.0),
    ]);

    for (drag_delta, anchor_y) in [(180.0, 120.0), (-240.0, 240.0), (300.0, 80.0)] {
        let _ = engine
            .axis_drag_scale_price(drag_delta, anchor_y, 0.2, 1e-6)
            .expect("price-axis drag scale");
        let frame = engine.build_render_frame().expect("build frame");
        let price_ys = sorted_price_label_ys(&frame);
        assert!(!price_ys.is_empty());
        assert!(price_ys.windows(2).all(|pair| pair[1] - pair[0] >= 22.0));
    }
}

#[test]
fn price_axis_tick_density_changes_with_vertical_scale_zoom() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 200.0).with_price_domain(10.0, 30.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(0.0, 12.0),
        DataPoint::new(1.0, 15.0),
        DataPoint::new(2.0, 22.0),
        DataPoint::new(3.0, 28.0),
    ]);
    engine
        .set_render_style(RenderStyle {
            show_last_price_label: false,
            show_last_price_line: false,
            ..engine.render_style()
        })
        .expect("disable last-price overlays");

    let baseline = engine.build_render_frame().expect("baseline frame");
    let baseline_count = price_label_count(&baseline);

    let _ = engine
        .axis_drag_scale_price(360.0, 220.0, 0.2, 1e-6)
        .expect("zoom out");
    let zoomed_out = engine.build_render_frame().expect("zoomed-out frame");
    let zoomed_out_count = price_label_count(&zoomed_out);

    let _ = engine
        .axis_drag_scale_price(-720.0, 220.0, 0.2, 1e-6)
        .expect("zoom in");
    let zoomed_in = engine.build_render_frame().expect("zoomed-in frame");
    let zoomed_in_count = price_label_count(&zoomed_in);

    assert!(zoomed_out_count < baseline_count);
    assert!(baseline_count < zoomed_in_count);
    assert!(zoomed_out_count < zoomed_in_count);
}

#[test]
fn major_time_labels_are_retained_and_collision_safe_under_mixed_zoom_density() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(1200, 420), 1_704_205_800.0, 1_704_241_800.0)
        .with_price_domain(90.0, 130.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let points: Vec<DataPoint> = (0..360)
        .map(|index| {
            let t = 1_704_205_800.0 + index as f64 * 100.0;
            DataPoint::new(t, 100.0 + (index as f64 * 0.08).sin() * 8.0)
        })
        .collect();
    engine.set_data(points);
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            locale: chart_rs::api::AxisLabelLocale::EnUs,
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
        .expect("time-axis config");
    engine
        .set_render_style(RenderStyle {
            time_axis_label_font_size_px: 11.0,
            major_time_label_font_size_px: 14.0,
            show_last_price_label: false,
            show_last_price_line: false,
            ..engine.render_style()
        })
        .expect("render style");

    engine
        .set_time_visible_range(1_704_205_800.0, 1_704_229_800.0)
        .expect("mixed-zoom range");
    let frame = engine.build_render_frame().expect("frame");

    let mut time_labels: Vec<_> = frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .collect();
    time_labels.sort_by(|left, right| left.x.total_cmp(&right.x));
    assert!(time_labels.len() >= 4);
    assert!(
        time_labels
            .windows(2)
            .all(|pair| pair[1].x - pair[0].x >= 56.0)
    );

    let major_count = time_labels
        .iter()
        .filter(|label| (label.font_size_px - 14.0).abs() <= 1e-9)
        .count();
    let minor_count = time_labels.len() - major_count;
    assert!(major_count >= 1, "expected at least one major time label");
    assert!(minor_count >= 1, "expected at least one minor time label");
}
