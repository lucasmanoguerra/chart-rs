use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, PriceScale, TimeScale, Viewport, project_line_segments};
use chart_rs::render::NullRenderer;

#[test]
fn line_projection_returns_empty_for_short_series() {
    let viewport = Viewport::new(800, 600);
    let time_scale = TimeScale::new(0.0, 10.0).expect("time scale");
    let price_scale = PriceScale::new(0.0, 100.0).expect("price scale");

    let empty = project_line_segments(&[], time_scale, price_scale, viewport).expect("project");
    assert!(empty.is_empty());

    let single = project_line_segments(
        &[DataPoint::new(1.0, 10.0)],
        time_scale,
        price_scale,
        viewport,
    )
    .expect("project");
    assert!(single.is_empty());
}

#[test]
fn line_projection_is_deterministic() {
    let viewport = Viewport::new(1000, 500);
    let time_scale = TimeScale::new(0.0, 10.0).expect("time scale");
    let price_scale = PriceScale::new(0.0, 100.0).expect("price scale");
    let points = vec![
        DataPoint::new(0.0, 0.0),
        DataPoint::new(5.0, 50.0),
        DataPoint::new(10.0, 100.0),
    ];

    let segments =
        project_line_segments(&points, time_scale, price_scale, viewport).expect("project");
    assert_eq!(segments.len(), 2);

    assert!((segments[0].x1 - 0.0).abs() <= 1e-9);
    assert!((segments[0].y1 - 499.0).abs() <= 1e-9);
    assert!((segments[0].x2 - 500.0).abs() <= 1e-9);
    assert!((segments[0].y2 - 249.5).abs() <= 1e-9);

    assert!((segments[1].x1 - 500.0).abs() <= 1e-9);
    assert!((segments[1].y1 - 249.5).abs() <= 1e-9);
    assert!((segments[1].x2 - 1000.0).abs() <= 1e-9);
    assert!((segments[1].y2 - 0.0).abs() <= 1e-9);
}

#[test]
fn engine_projects_line_segments_with_current_visible_range() {
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

    let segments = engine.project_line_segments().expect("project");
    assert_eq!(segments.len(), 2);

    // With visible range 25..75 and width 1000:
    // x(25)=0, x(50)=500, x(75)=1000.
    assert!((segments[0].x1 - 0.0).abs() <= 1e-9);
    assert!((segments[0].x2 - 500.0).abs() <= 1e-9);
    assert!((segments[1].x1 - 500.0).abs() <= 1e-9);
    assert!((segments[1].x2 - 1000.0).abs() <= 1e-9);
}
