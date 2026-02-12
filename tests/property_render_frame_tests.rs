use chart_rs::api::{ChartEngine, ChartEngineConfig, CrosshairMode, RenderStyle};
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
}
