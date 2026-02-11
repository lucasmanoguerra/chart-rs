use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::NullRenderer;
use proptest::prelude::*;

proptest! {
    #[test]
    fn projected_line_segment_count_matches_points(
        times in proptest::collection::vec(-10_000.0f64..10_000.0, 2..64),
        prices in proptest::collection::vec(-1_000.0f64..1_000.0, 2..64)
    ) {
        let len = times.len().min(prices.len());
        prop_assume!(len >= 2);

        let mut points = Vec::with_capacity(len);
        for i in 0..len {
            points.push(DataPoint::new(times[i], prices[i]));
        }

        let mut min_time = f64::INFINITY;
        let mut max_time = f64::NEG_INFINITY;
        let mut min_price = f64::INFINITY;
        let mut max_price = f64::NEG_INFINITY;
        for point in &points {
            min_time = min_time.min(point.x);
            max_time = max_time.max(point.x);
            min_price = min_price.min(point.y);
            max_price = max_price.max(point.y);
        }

        if min_time == max_time {
            max_time += 1.0;
        }
        if min_price == max_price {
            max_price += 1.0;
        }

        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(
            Viewport::new(1200, 700),
            min_time,
            max_time,
        )
        .with_price_domain(min_price, max_price);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(points);

        let segments = engine.project_line_segments().expect("project");
        prop_assert_eq!(segments.len(), len - 1);

        for segment in &segments {
            prop_assert!(segment.x1.is_finite());
            prop_assert!(segment.y1.is_finite());
            prop_assert!(segment.x2.is_finite());
            prop_assert!(segment.y2.is_finite());
        }
    }
}
