use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, PriceScale, TimeScale, Viewport, project_histogram_bars};
use chart_rs::render::NullRenderer;

#[test]
fn histogram_projection_returns_empty_for_empty_series() {
    let viewport = Viewport::new(800, 600);
    let time_scale = TimeScale::new(0.0, 10.0).expect("time scale");
    let price_scale = PriceScale::new(0.0, 100.0).expect("price scale");

    let bars =
        project_histogram_bars(&[], time_scale, price_scale, viewport, 6.0, 50.0).expect("project");
    assert!(bars.is_empty());
}

#[test]
fn histogram_projection_rejects_invalid_bar_width() {
    let viewport = Viewport::new(800, 600);
    let time_scale = TimeScale::new(0.0, 10.0).expect("time scale");
    let price_scale = PriceScale::new(0.0, 100.0).expect("price scale");
    let points = vec![DataPoint::new(1.0, 10.0)];

    let err = project_histogram_bars(&points, time_scale, price_scale, viewport, 0.0, 50.0)
        .expect_err("must reject width <= 0");
    assert!(format!("{err}").contains("histogram bar width"));
}

#[test]
fn histogram_projection_is_deterministic() {
    let viewport = Viewport::new(1000, 500);
    let time_scale = TimeScale::new(0.0, 10.0).expect("time scale");
    let price_scale = PriceScale::new(0.0, 100.0).expect("price scale");
    let points = vec![
        DataPoint::new(0.0, 0.0),
        DataPoint::new(5.0, 50.0),
        DataPoint::new(10.0, 100.0),
    ];

    let bars = project_histogram_bars(&points, time_scale, price_scale, viewport, 10.0, 50.0)
        .expect("project");
    assert_eq!(bars.len(), 3);

    assert!((bars[0].x_center - 0.0).abs() <= 1e-9);
    assert!((bars[0].x_left + 5.0).abs() <= 1e-9);
    assert!((bars[0].x_right - 5.0).abs() <= 1e-9);
    assert!((bars[0].y_top - 250.0).abs() <= 1e-9);
    assert!((bars[0].y_bottom - 500.0).abs() <= 1e-9);

    assert!((bars[1].x_center - 500.0).abs() <= 1e-9);
    assert!((bars[1].y_top - 250.0).abs() <= 1e-9);
    assert!((bars[1].y_bottom - 250.0).abs() <= 1e-9);

    assert!((bars[2].x_center - 1000.0).abs() <= 1e-9);
    assert!((bars[2].y_top - 0.0).abs() <= 1e-9);
    assert!((bars[2].y_bottom - 250.0).abs() <= 1e-9);
}

#[test]
fn engine_projects_visible_histogram_with_current_visible_range() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_data(vec![
        DataPoint::new(25.0, 25.0),
        DataPoint::new(50.0, 50.0),
        DataPoint::new(75.0, 75.0),
    ]);
    engine
        .set_time_visible_range(25.0, 75.0)
        .expect("visible range");

    let bars = engine
        .project_visible_histogram_bars(8.0, 50.0)
        .expect("project");
    assert_eq!(bars.len(), 3);
}

#[test]
fn histogram_projection_with_overscan_includes_neighbors() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_data(vec![
        DataPoint::new(25.0, 25.0),
        DataPoint::new(50.0, 50.0),
        DataPoint::new(75.0, 75.0),
    ]);
    engine
        .set_time_visible_range(40.0, 60.0)
        .expect("visible range");

    let visible = engine
        .project_visible_histogram_bars(8.0, 50.0)
        .expect("project");
    assert_eq!(visible.len(), 1);

    let overscanned = engine
        .project_visible_histogram_bars_with_overscan(8.0, 50.0, 1.0)
        .expect("project with overscan");
    assert_eq!(overscanned.len(), 3);
}
