#![cfg(feature = "cairo-backend")]

use cairo::{Context, Format, ImageSurface};
use chart_rs::ChartError;
use chart_rs::api::{ChartEngine, ChartEngineConfig, InvalidationLevel, RenderStyle};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::{CairoRenderer, Color};

#[test]
fn cairo_renderer_rejects_invalid_surface_size() {
    let err = CairoRenderer::new(0, 480).expect_err("invalid width must fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn cairo_renderer_renders_series_and_axis_primitives() {
    let renderer = CairoRenderer::new(900, 500).expect("renderer");
    let config =
        ChartEngineConfig::new(Viewport::new(900, 500), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(10.0, 10.0),
        DataPoint::new(20.0, 20.0),
        DataPoint::new(40.0, 15.0),
    ]);
    let frame = engine.build_render_frame().expect("build frame");

    engine.render().expect("render");
    let renderer = engine.into_renderer();
    let stats = renderer.last_stats();

    assert_eq!(stats.lines_drawn, frame.lines.len());
    assert_eq!(stats.rects_drawn, frame.rects.len());
    assert_eq!(stats.texts_drawn, frame.texts.len());
}

#[test]
fn cairo_renderer_can_draw_on_external_context() {
    let renderer = CairoRenderer::new(600, 320).expect("renderer");
    let config =
        ChartEngineConfig::new(Viewport::new(600, 320), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(30.0, 20.0),
        DataPoint::new(60.0, 15.0),
    ]);

    let surface = ImageSurface::create(Format::ARgb32, 600, 320).expect("surface");
    let context = Context::new(&surface).expect("context");
    engine
        .render_on_cairo_context(&context)
        .expect("render on context");

    let renderer = engine.into_renderer();
    assert!(renderer.last_stats().lines_drawn >= 6);
}

#[test]
fn cairo_renderer_draws_last_price_label_box_rectangles() {
    let renderer = CairoRenderer::new(600, 320).expect("renderer");
    let config =
        ChartEngineConfig::new(Viewport::new(600, 320), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(30.0, 20.0),
        DataPoint::new(60.0, 15.0),
    ]);
    engine
        .set_render_style(RenderStyle {
            show_last_price_label_box: true,
            last_price_label_box_use_marker_color: false,
            last_price_label_box_color: Color::rgb(0.12, 0.12, 0.12),
            last_price_label_box_auto_text_contrast: false,
            last_price_label_box_text_color: Color::rgb(0.95, 0.95, 0.95),
            last_price_label_box_border_width_px: 1.0,
            last_price_label_box_border_color: Color::rgb(0.8, 0.8, 0.8),
            last_price_label_box_corner_radius_px: 4.0,
            ..engine.render_style()
        })
        .expect("set style");

    engine.render().expect("render");

    let renderer = engine.into_renderer();
    assert!(renderer.last_stats().rects_drawn >= 1);
}

#[test]
fn cairo_renderer_supports_multi_pane_cursor_partial_render_path() {
    let renderer = CairoRenderer::new(600, 320).expect("renderer");
    let config =
        ChartEngineConfig::new(Viewport::new(600, 320), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(30.0, 20.0),
        DataPoint::new(60.0, 15.0),
    ]);
    let _aux = engine.create_pane(1.0).expect("create pane");
    engine.clear_pending_invalidation();
    engine.pointer_move(180.0, 140.0);
    let pending_snapshot = engine
        .lwc_pending_invalidation_snapshot()
        .expect("pending snapshot");
    assert_eq!(pending_snapshot.level, InvalidationLevel::Cursor);
    assert_eq!(pending_snapshot.time_scale_invalidation_count, 0);

    let surface = ImageSurface::create(Format::ARgb32, 600, 320).expect("surface");
    let context = Context::new(&surface).expect("context");
    engine
        .render_on_cairo_context(&context)
        .expect("render on context");

    assert!(!engine.has_pending_invalidation());
    let renderer = engine.into_renderer();
    assert!(renderer.last_stats().lines_drawn > 0);
}

#[test]
fn cairo_renderer_exposes_lwc_snapshot_for_time_scale_mutation_before_render() {
    let renderer = CairoRenderer::new(600, 320).expect("renderer");
    let config =
        ChartEngineConfig::new(Viewport::new(600, 320), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(0.0, 10.0),
        DataPoint::new(30.0, 20.0),
        DataPoint::new(60.0, 15.0),
    ]);
    let _aux = engine.create_pane(1.0).expect("create pane");
    engine.clear_pending_invalidation();

    engine
        .pan_time_visible_by_pixels(24.0)
        .expect("pan by pixels should work");
    let pending_snapshot = engine
        .lwc_pending_invalidation_snapshot()
        .expect("pending snapshot");
    assert_eq!(pending_snapshot.level, InvalidationLevel::Light);
    assert!(pending_snapshot.time_scale_invalidation_count > 0);

    let surface = ImageSurface::create(Format::ARgb32, 600, 320).expect("surface");
    let context = Context::new(&surface).expect("context");
    engine
        .render_on_cairo_context(&context)
        .expect("render on context");
    assert!(engine.lwc_pending_invalidation_snapshot().is_none());
}
