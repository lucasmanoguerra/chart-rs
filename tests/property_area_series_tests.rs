use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::NullRenderer;
use proptest::prelude::*;

proptest! {
    #[test]
    fn area_geometry_matches_point_count_and_has_finite_vertices(
        times in proptest::collection::vec(-10_000.0f64..10_000.0, 1..64),
        prices in proptest::collection::vec(-1_000.0f64..1_000.0, 1..64)
    ) {
        let len = times.len().min(prices.len());
        prop_assume!(len >= 1);

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

        let geometry = engine.project_area_geometry().expect("project");
        prop_assert_eq!(geometry.line_points.len(), len);
        prop_assert_eq!(geometry.fill_polygon.len(), len + 3);

        for vertex in &geometry.line_points {
            prop_assert!(vertex.x.is_finite());
            prop_assert!(vertex.y.is_finite());
        }
        for vertex in &geometry.fill_polygon {
            prop_assert!(vertex.x.is_finite());
            prop_assert!(vertex.y.is_finite());
        }

        let baseline = 700.0;
        prop_assert!((geometry.fill_polygon[0].y - baseline).abs() <= 1e-9);
        prop_assert!((geometry.fill_polygon[geometry.fill_polygon.len() - 1].y - baseline).abs() <= 1e-9);
    }
}
