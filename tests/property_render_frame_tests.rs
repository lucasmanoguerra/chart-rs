use chart_rs::api::{
    ChartEngine, ChartEngineConfig, CrosshairLabelBoxWidthMode, CrosshairMode, RenderStyle,
};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::{Color, NullRenderer, TextHAlign};
use proptest::prelude::*;

proptest! {
    #[test]
    fn render_frame_build_is_deterministic_and_finite(
        samples in prop::collection::vec((0u16..2000u16, -5000.0f64..5000.0f64), 2..128)
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");

        let points: Vec<DataPoint> = samples
            .into_iter()
            .map(|(time, price)| DataPoint::new(f64::from(time), price))
            .collect();
        engine.set_data(points);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        let style = engine.render_style();

        prop_assert_eq!(&first, &second);
        let time_labels: Vec<f64> = first
            .texts
            .iter()
            .filter(|label| label.h_align == TextHAlign::Center)
            .map(|label| label.x)
            .collect();
        let price_labels: Vec<f64> = first
            .texts
            .iter()
            .filter(|label| label.h_align == TextHAlign::Right && label.color == style.axis_label_color)
            .map(|label| label.y + 8.0)
            .collect();

        prop_assert!(!time_labels.is_empty());
        prop_assert!(!price_labels.is_empty());
        prop_assert!(first.lines.iter().all(|line|
            line.x1.is_finite()
            && line.y1.is_finite()
            && line.x2.is_finite()
            && line.y2.is_finite()
            && line.stroke_width.is_finite()
            && line.stroke_width > 0.0
        ));

        let mut sorted_time = time_labels;
        sorted_time.sort_by(f64::total_cmp);
        prop_assert!(sorted_time.windows(2).all(|pair| pair[1] - pair[0] >= 56.0));

        let mut sorted_price = price_labels;
        sorted_price.sort_by(f64::total_cmp);
        prop_assert!(sorted_price.windows(2).all(|pair| pair[1] - pair[0] >= 22.0));
    }

    #[test]
    fn crosshair_render_lines_are_deterministic_and_toggleable(
        x in 0.0f64..1280.0f64,
        y in 0.0f64..720.0f64,
        show_horizontal in any::<bool>(),
        show_vertical in any::<bool>(),
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_line_color: Color::rgb(0.93, 0.21, 0.17),
            crosshair_line_width: 2.0,
            show_crosshair_horizontal_line: show_horizontal,
            show_crosshair_vertical_line: show_vertical,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(x, y);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);

        let crosshair_lines: Vec<_> = first
            .lines
            .iter()
            .filter(|line| {
                line.color == style.crosshair_line_color
                    && (line.stroke_width - style.crosshair_line_width).abs() <= 1e-12
            })
            .collect();
        let expected_count = usize::from(show_horizontal) + usize::from(show_vertical);
        prop_assert_eq!(crosshair_lines.len(), expected_count);
    }

    #[test]
    fn crosshair_axis_labels_are_deterministic_and_toggleable(
        x in 0.0f64..1280.0f64,
        y in 0.0f64..720.0f64,
        show_time_label in any::<bool>(),
        show_price_label in any::<bool>(),
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_time_label_color: Color::rgb(0.88, 0.22, 0.19),
            crosshair_price_label_color: Color::rgb(0.19, 0.43, 0.88),
            show_crosshair_time_label_box: false,
            show_crosshair_price_label_box: false,
            show_crosshair_time_label: show_time_label,
            show_crosshair_price_label: show_price_label,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(x, y);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);

        let time_labels = first
            .texts
            .iter()
            .filter(|text| text.color == style.crosshair_time_label_color)
            .count();
        let price_labels = first
            .texts
            .iter()
            .filter(|text| text.color == style.crosshair_price_label_color)
            .count();
        prop_assert_eq!(time_labels, usize::from(show_time_label));
        prop_assert_eq!(price_labels, usize::from(show_price_label));
    }

    #[test]
    fn crosshair_axis_label_boxes_are_deterministic_and_toggleable(
        x in 0.0f64..1280.0f64,
        y in 0.0f64..720.0f64,
        show_time_box in any::<bool>(),
        show_price_box in any::<bool>(),
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_label_box_color: Color::rgb(0.93, 0.82, 0.18),
            show_crosshair_time_label_box: show_time_box,
            show_crosshair_price_label_box: show_price_box,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(x, y);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);

        let box_count = first
            .rects
            .iter()
            .filter(|rect| rect.fill_color == style.crosshair_label_box_color)
            .count();
        let expected_count = usize::from(show_time_box) + usize::from(show_price_box);
        prop_assert_eq!(box_count, expected_count);
    }

    #[test]
    fn crosshair_axis_label_box_radius_is_clamped(
        requested_radius in 0.0f64..200.0f64,
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_label_box_color: Color::rgb(0.93, 0.82, 0.18),
            crosshair_label_box_border_width_px: 1.0,
            crosshair_label_box_corner_radius_px: requested_radius,
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(400.0, 250.0);

        let frame = engine.build_render_frame().expect("frame");
        let boxes: Vec<_> = frame
            .rects
            .iter()
            .filter(|rect| rect.fill_color == style.crosshair_label_box_color)
            .collect();
        prop_assert!(!boxes.is_empty());
        prop_assert!(boxes.iter().all(|rect| {
            rect.corner_radius <= (rect.width.min(rect.height)) * 0.5 + 1e-9
                && rect.border_width >= 0.0
        }));
    }

    #[test]
    fn crosshair_axis_label_box_auto_contrast_is_deterministic(
        bright_fill in any::<bool>(),
        auto_contrast in any::<bool>(),
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let fill = if bright_fill {
            Color::rgb(0.95, 0.95, 0.95)
        } else {
            Color::rgb(0.12, 0.12, 0.12)
        };
        let style = RenderStyle {
            crosshair_label_box_color: fill,
            crosshair_label_box_text_color: Color::rgb(0.9, 0.2, 0.2),
            crosshair_label_box_auto_text_contrast: auto_contrast,
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(400.0, 250.0);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);

        let expected_color = if auto_contrast {
            if bright_fill {
                Color::rgb(0.06, 0.08, 0.11)
            } else {
                Color::rgb(1.0, 1.0, 1.0)
            }
        } else {
            style.crosshair_label_box_text_color
        };
        prop_assert!(first.texts.iter().any(|text| {
            text.h_align == TextHAlign::Center && text.color == expected_color
        }));
        prop_assert!(first.texts.iter().any(|text| {
            text.h_align == TextHAlign::Right && text.color == expected_color
        }));
    }

    #[test]
    fn crosshair_axis_label_box_text_policy_is_deterministic_per_axis(
        time_auto_contrast in any::<bool>(),
        price_auto_contrast in any::<bool>(),
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_label_box_color: Color::rgb(0.95, 0.95, 0.95),
            crosshair_label_box_text_color: Color::rgb(0.9, 0.2, 0.2),
            crosshair_label_box_auto_text_contrast: false,
            crosshair_time_label_box_text_color: Some(Color::rgb(0.12, 0.76, 0.33)),
            crosshair_price_label_box_text_color: Some(Color::rgb(0.22, 0.41, 0.90)),
            crosshair_time_label_box_auto_text_contrast: Some(time_auto_contrast),
            crosshair_price_label_box_auto_text_contrast: Some(price_auto_contrast),
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(400.0, 250.0);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);
    }

    #[test]
    fn crosshair_axis_label_box_fill_color_is_deterministic_per_axis(
        time_fill_g in 0.0f64..1.0f64,
        price_fill_b in 0.0f64..1.0f64,
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_label_box_color: Color::rgb(0.2, 0.2, 0.2),
            crosshair_time_label_box_color: Some(Color::rgb(0.9, time_fill_g, 0.2)),
            crosshair_price_label_box_color: Some(Color::rgb(0.2, 0.4, price_fill_b)),
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(400.0, 250.0);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);
    }

    #[test]
    fn crosshair_axis_label_box_full_axis_mode_is_deterministic(
        x in 0.0f64..1280.0f64,
        y in 0.0f64..720.0f64,
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_label_box_color: Color::rgb(0.93, 0.82, 0.18),
            crosshair_label_box_width_mode: CrosshairLabelBoxWidthMode::FullAxis,
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(x, y);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);

        let viewport_width = f64::from(engine.viewport().width);
        let plot_right = (viewport_width - style.price_axis_width_px).clamp(0.0, viewport_width);
        let axis_panel_width = (viewport_width - plot_right).max(0.0);
        let boxes: Vec<_> = first
            .rects
            .iter()
            .filter(|rect| rect.fill_color == style.crosshair_label_box_color)
            .collect();
        prop_assert_eq!(boxes.len(), 2);
        prop_assert!(boxes.iter().any(|rect| (rect.width - plot_right).abs() <= 1e-9));
        prop_assert!(boxes.iter().any(|rect| (rect.width - axis_panel_width).abs() <= 1e-9));
    }

    #[test]
    fn crosshair_axis_label_box_width_mode_is_deterministic_per_axis(
        time_full_axis in any::<bool>(),
        price_full_axis in any::<bool>(),
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_label_box_width_mode: CrosshairLabelBoxWidthMode::FitText,
            crosshair_time_label_box_width_mode: Some(if time_full_axis {
                CrosshairLabelBoxWidthMode::FullAxis
            } else {
                CrosshairLabelBoxWidthMode::FitText
            }),
            crosshair_price_label_box_width_mode: Some(if price_full_axis {
                CrosshairLabelBoxWidthMode::FullAxis
            } else {
                CrosshairLabelBoxWidthMode::FitText
            }),
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(3.0, 250.0);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);
    }

    #[test]
    fn crosshair_axis_label_box_min_width_is_deterministic_per_axis(
        time_min_width in 0.0f64..300.0f64,
        price_min_width in 0.0f64..70.0f64,
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_label_box_color: Color::rgb(0.93, 0.82, 0.18),
            crosshair_label_box_width_mode: CrosshairLabelBoxWidthMode::FitText,
            crosshair_time_label_box_min_width_px: time_min_width,
            crosshair_price_label_box_min_width_px: price_min_width,
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(3.0, 250.0);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);
    }

    #[test]
    fn crosshair_axis_label_box_border_visibility_is_deterministic(
        show_time_border in any::<bool>(),
        show_price_border in any::<bool>(),
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_label_box_color: Color::rgb(0.93, 0.82, 0.18),
            crosshair_label_box_border_width_px: 1.5,
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            show_crosshair_time_label_box_border: show_time_border,
            show_crosshair_price_label_box_border: show_price_border,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(400.0, 250.0);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);

        let viewport_width = f64::from(engine.viewport().width);
        let plot_right = (viewport_width - style.price_axis_width_px).clamp(0.0, viewport_width);
        let mut actual_time_border = false;
        let mut actual_price_border = false;
        for rect in first
            .rects
            .iter()
            .filter(|rect| rect.fill_color == style.crosshair_label_box_color)
        {
            if rect.x < plot_right {
                actual_time_border = rect.border_width > 0.0;
            } else {
                actual_price_border = rect.border_width > 0.0;
            }
        }
        prop_assert_eq!(actual_time_border, show_time_border);
        prop_assert_eq!(actual_price_border, show_price_border);
    }

    #[test]
    fn crosshair_axis_label_offsets_are_deterministic(
        time_offset in 0.0f64..24.0f64,
        price_offset in 0.0f64..24.0f64,
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            show_crosshair_time_label_box: false,
            show_crosshair_price_label_box: false,
            crosshair_time_label_offset_y_px: time_offset,
            crosshair_price_label_offset_y_px: price_offset,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(400.0, 250.0);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);
    }

    #[test]
    fn crosshair_axis_label_horizontal_insets_are_deterministic(
        time_padding_x in 0.0f64..120.0f64,
        price_padding_right in 0.0f64..120.0f64,
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            show_crosshair_time_label_box: false,
            show_crosshair_price_label_box: false,
            crosshair_time_label_padding_x_px: time_padding_x,
            crosshair_price_label_padding_right_px: price_padding_right,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(3.0, 250.0);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);
    }

    #[test]
    fn crosshair_axis_label_font_sizes_are_deterministic(
        time_font_size in 8.0f64..22.0f64,
        price_font_size in 8.0f64..22.0f64,
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            show_crosshair_time_label_box: false,
            show_crosshair_price_label_box: false,
            crosshair_time_label_font_size_px: time_font_size,
            crosshair_price_label_font_size_px: price_font_size,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(3.0, 250.0);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);
    }

    #[test]
    fn crosshair_axis_label_box_padding_is_deterministic_per_axis(
        time_padding_x in 0.0f64..20.0f64,
        time_padding_y in 0.0f64..10.0f64,
        price_padding_x in 0.0f64..20.0f64,
        price_padding_y in 0.0f64..10.0f64,
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_label_box_width_mode: CrosshairLabelBoxWidthMode::FitText,
            crosshair_time_label_box_padding_x_px: time_padding_x,
            crosshair_time_label_box_padding_y_px: time_padding_y,
            crosshair_price_label_box_padding_x_px: price_padding_x,
            crosshair_price_label_box_padding_y_px: price_padding_y,
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(3.0, 250.0);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);
    }

    #[test]
    fn crosshair_axis_label_box_border_style_is_deterministic_per_axis(
        time_border_width in 0.0f64..3.0f64,
        price_border_width in 0.0f64..3.0f64,
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_time_label_box_border_width_px: time_border_width,
            crosshair_price_label_box_border_width_px: price_border_width,
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(3.0, 250.0);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);
    }

    #[test]
    fn crosshair_axis_label_box_corner_radius_is_deterministic_per_axis(
        time_corner_radius in 0.0f64..20.0f64,
        price_corner_radius in 0.0f64..20.0f64,
    ) {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(1280, 720), 0.0, 2000.0)
            .with_price_domain(-6000.0, 6000.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(vec![
            DataPoint::new(10.0, 100.0),
            DataPoint::new(100.0, 200.0),
            DataPoint::new(250.0, -50.0),
        ]);
        engine.set_crosshair_mode(CrosshairMode::Normal);
        let style = RenderStyle {
            crosshair_label_box_corner_radius_px: 0.0,
            crosshair_time_label_box_corner_radius_px: time_corner_radius,
            crosshair_price_label_box_corner_radius_px: price_corner_radius,
            show_crosshair_time_label_box: true,
            show_crosshair_price_label_box: true,
            ..engine.render_style()
        };
        engine.set_render_style(style).expect("set style");
        engine.pointer_move(3.0, 250.0);

        let first = engine.build_render_frame().expect("first frame");
        let second = engine.build_render_frame().expect("second frame");
        prop_assert_eq!(first, second);
        prop_assert!(first.rects.iter().all(|rect| {
            rect.corner_radius <= (rect.width.min(rect.height)) * 0.5 + 1e-9
        }));
    }
}
