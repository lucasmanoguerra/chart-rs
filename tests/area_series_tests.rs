use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, PriceScale, TimeScale, Viewport, project_area_geometry};
use chart_rs::render::NullRenderer;

#[test]
fn area_projection_returns_empty_for_empty_series() {
    let viewport = Viewport::new(800, 600);
    let time_scale = TimeScale::new(0.0, 10.0).expect("time scale");
    let price_scale = PriceScale::new(0.0, 100.0).expect("price scale");

    let geometry = project_area_geometry(&[], time_scale, price_scale, viewport).expect("project");
    assert!(geometry.line_points.is_empty());
    assert!(geometry.fill_polygon.is_empty());
}

#[test]
fn area_projection_is_deterministic() {
    let viewport = Viewport::new(1000, 500);
    let time_scale = TimeScale::new(0.0, 10.0).expect("time scale");
    let price_scale = PriceScale::new(0.0, 100.0).expect("price scale");
    let points = vec![
        DataPoint::new(0.0, 0.0),
        DataPoint::new(5.0, 50.0),
        DataPoint::new(10.0, 100.0),
    ];

    let geometry =
        project_area_geometry(&points, time_scale, price_scale, viewport).expect("project");
    assert_eq!(geometry.line_points.len(), 3);
    assert_eq!(geometry.fill_polygon.len(), 6);

    assert!((geometry.line_points[0].x - 0.0).abs() <= 1e-9);
    assert!((geometry.line_points[0].y - 499.0).abs() <= 1e-9);
    assert!((geometry.line_points[1].x - 500.0).abs() <= 1e-9);
    assert!((geometry.line_points[1].y - 249.5).abs() <= 1e-9);
    assert!((geometry.line_points[2].x - 1000.0).abs() <= 1e-9);
    assert!((geometry.line_points[2].y - 0.0).abs() <= 1e-9);

    // Explicitly closed baseline polygon:
    // [baseline-start, line points..., baseline-end, baseline-start]
    assert!((geometry.fill_polygon[0].x - 0.0).abs() <= 1e-9);
    assert!((geometry.fill_polygon[0].y - 500.0).abs() <= 1e-9);
    assert!((geometry.fill_polygon[4].x - 1000.0).abs() <= 1e-9);
    assert!((geometry.fill_polygon[4].y - 500.0).abs() <= 1e-9);
    assert!((geometry.fill_polygon[5].x - 0.0).abs() <= 1e-9);
    assert!((geometry.fill_polygon[5].y - 500.0).abs() <= 1e-9);
}

#[test]
fn engine_projects_visible_area_with_current_visible_range() {
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

    let geometry = engine.project_visible_area_geometry().expect("project");
    assert_eq!(geometry.line_points.len(), 3);
    assert_eq!(geometry.fill_polygon.len(), 6);

    // With visible range 25..75 and width 1000:
    // x(25)=0, x(50)=500, x(75)=1000.
    assert!((geometry.line_points[0].x - 0.0).abs() <= 1e-9);
    assert!((geometry.line_points[1].x - 500.0).abs() <= 1e-9);
    assert!((geometry.line_points[2].x - 1000.0).abs() <= 1e-9);
}

#[test]
fn area_projection_with_overscan_includes_neighbors() {
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

    let visible = engine.project_visible_area_geometry().expect("project");
    assert_eq!(visible.line_points.len(), 1);

    let overscanned = engine
        .project_visible_area_geometry_with_overscan(1.0)
        .expect("project with overscan");
    assert_eq!(overscanned.line_points.len(), 3);
}
