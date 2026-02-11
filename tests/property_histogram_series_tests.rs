use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::NullRenderer;
use proptest::prelude::*;

proptest! {
    #[test]
    fn histogram_geometry_matches_point_count_and_axis_invariants(
        times in proptest::collection::vec(-10_000.0f64..10_000.0, 1..64),
        prices in proptest::collection::vec(-1_000.0f64..1_000.0, 1..64),
        bar_width in 1.0f64..20.0
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
        let baseline_price = (min_price + max_price) * 0.5;

        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(
            Viewport::new(1200, 700),
            min_time,
            max_time,
        )
        .with_price_domain(min_price, max_price);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine.set_data(points);

        let bars = engine
            .project_histogram_bars(bar_width, baseline_price)
            .expect("project");
        let baseline_y = engine
            .map_price_to_pixel(baseline_price)
            .expect("baseline y");

        prop_assert_eq!(bars.len(), len);
        for bar in &bars {
            prop_assert!(bar.x_center.is_finite());
            prop_assert!(bar.x_left.is_finite());
            prop_assert!(bar.x_right.is_finite());
            prop_assert!(bar.y_top.is_finite());
            prop_assert!(bar.y_bottom.is_finite());
            prop_assert!(bar.x_left <= bar.x_center + 1e-9);
            prop_assert!(bar.x_center <= bar.x_right + 1e-9);
            prop_assert!(bar.y_top <= bar.y_bottom + 1e-9);
            prop_assert!(bar.y_top <= baseline_y + 1e-9);
            prop_assert!(bar.y_bottom >= baseline_y - 1e-9);
        }
    }
}
