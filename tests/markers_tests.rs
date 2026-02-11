use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{OhlcBar, Viewport};
use chart_rs::extensions::{
    MarkerPlacementConfig, MarkerPosition, MarkerSide, SeriesMarker, place_markers_on_candles,
};
use chart_rs::render::NullRenderer;

#[test]
fn marker_placement_avoids_overlap_inside_lane() {
    let candles = vec![
        OhlcBar::new(1.0, 40.0, 45.0, 38.0, 42.0).expect("c1"),
        OhlcBar::new(2.0, 41.0, 46.0, 39.0, 43.0).expect("c2"),
        OhlcBar::new(3.0, 42.0, 47.0, 40.0, 44.0).expect("c3"),
    ];
    let markers = vec![
        SeriesMarker::new("m1", 1.0, MarkerPosition::AboveBar).with_text("alpha"),
        SeriesMarker::new("m2", 1.05, MarkerPosition::AboveBar).with_text("beta"),
        SeriesMarker::new("m3", 1.1, MarkerPosition::AboveBar).with_text("gamma"),
    ];

    let config = MarkerPlacementConfig::default();
    let placed = place_markers_on_candles(
        &markers,
        &candles,
        chart_rs::core::TimeScale::new(0.0, 4.0).expect("time scale"),
        chart_rs::core::PriceScale::new(0.0, 100.0).expect("price scale"),
        Viewport::new(600, 400),
        config,
    )
    .expect("placement");

    assert_eq!(placed.len(), 3);
    assert!(placed.iter().any(|marker| marker.lane > 0));

    for i in 0..placed.len() {
        for j in (i + 1)..placed.len() {
            let a = &placed[i];
            let b = &placed[j];
            if a.side == b.side && a.lane == b.lane {
                let non_overlap = a.collision_right_px + config.min_horizontal_gap_px
                    <= b.collision_left_px
                    || b.collision_right_px + config.min_horizontal_gap_px <= a.collision_left_px;
                assert!(non_overlap);
            }
        }
    }
}

#[test]
fn marker_position_uses_expected_anchor_price() {
    let candles = vec![OhlcBar::new(50.0, 55.0, 80.0, 20.0, 60.0).expect("candle")];
    let markers = vec![
        SeriesMarker::new("above", 50.0, MarkerPosition::AboveBar),
        SeriesMarker::new("in", 50.0, MarkerPosition::InBar),
        SeriesMarker::new("below", 50.0, MarkerPosition::BelowBar),
        SeriesMarker::new("price", 50.0, MarkerPosition::Price(70.0)),
    ];
    let placed = place_markers_on_candles(
        &markers,
        &candles,
        chart_rs::core::TimeScale::new(0.0, 100.0).expect("time scale"),
        chart_rs::core::PriceScale::new(0.0, 100.0).expect("price scale"),
        Viewport::new(800, 400),
        MarkerPlacementConfig::default(),
    )
    .expect("placement");

    let above = placed
        .iter()
        .find(|m| m.id == "above")
        .expect("above marker");
    let inbar = placed.iter().find(|m| m.id == "in").expect("in marker");
    let below = placed
        .iter()
        .find(|m| m.id == "below")
        .expect("below marker");
    let price = placed
        .iter()
        .find(|m| m.id == "price")
        .expect("price marker");

    assert_eq!(above.side, MarkerSide::Above);
    assert_eq!(inbar.side, MarkerSide::Center);
    assert_eq!(below.side, MarkerSide::Below);
    assert!((price.price - 70.0).abs() <= 1e-9);

    assert!(above.y < inbar.y);
    assert!(inbar.y < below.y);
}

#[test]
fn visible_marker_projection_filters_by_window() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 400), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_candles(vec![
        OhlcBar::new(10.0, 20.0, 30.0, 15.0, 25.0).expect("c1"),
        OhlcBar::new(50.0, 40.0, 55.0, 35.0, 48.0).expect("c2"),
        OhlcBar::new(90.0, 60.0, 75.0, 58.0, 70.0).expect("c3"),
    ]);
    engine
        .set_time_visible_range(30.0, 80.0)
        .expect("set visible");

    let markers = vec![
        SeriesMarker::new("m-left", 10.0, MarkerPosition::AboveBar),
        SeriesMarker::new("m-mid", 50.0, MarkerPosition::AboveBar),
        SeriesMarker::new("m-right", 90.0, MarkerPosition::AboveBar),
    ];
    let projected = engine
        .project_visible_markers_on_candles(&markers, MarkerPlacementConfig::default())
        .expect("project visible markers");

    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].id, "m-mid");
}

#[test]
fn visible_marker_projection_with_overscan_includes_neighbors() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 400), 0.0, 100.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_candles(vec![
        OhlcBar::new(18.0, 20.0, 30.0, 15.0, 25.0).expect("c1"),
        OhlcBar::new(30.0, 30.0, 40.0, 25.0, 35.0).expect("c2"),
        OhlcBar::new(70.0, 40.0, 50.0, 35.0, 45.0).expect("c3"),
        OhlcBar::new(82.0, 50.0, 60.0, 45.0, 55.0).expect("c4"),
    ]);
    engine
        .set_time_visible_range(20.0, 80.0)
        .expect("set visible");

    let markers = vec![
        SeriesMarker::new("m-left", 18.0, MarkerPosition::AboveBar),
        SeriesMarker::new("m-30", 30.0, MarkerPosition::AboveBar),
        SeriesMarker::new("m-70", 70.0, MarkerPosition::AboveBar),
        SeriesMarker::new("m-right", 82.0, MarkerPosition::AboveBar),
    ];

    let base = engine
        .project_visible_markers_on_candles(&markers, MarkerPlacementConfig::default())
        .expect("visible markers");
    let overscan = engine
        .project_visible_markers_on_candles_with_overscan(
            &markers,
            0.05,
            MarkerPlacementConfig::default(),
        )
        .expect("visible markers overscan");

    assert_eq!(base.len(), 2);
    assert_eq!(overscan.len(), 4);
}
