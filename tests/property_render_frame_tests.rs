use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::NullRenderer;
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

        prop_assert_eq!(&first, &second);
        prop_assert_eq!(first.texts.len(), 10);
        prop_assert!(first.lines.iter().all(|line|
            line.x1.is_finite()
            && line.y1.is_finite()
            && line.x2.is_finite()
            && line.y2.is_finite()
            && line.stroke_width.is_finite()
            && line.stroke_width > 0.0
        ));
    }
}
