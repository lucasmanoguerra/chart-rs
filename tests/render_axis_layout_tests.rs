use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::Viewport;
use chart_rs::render::{NullRenderer, TextHAlign};

#[test]
fn narrow_viewport_uses_collision_safe_axis_labels() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(180, 120), 0.0, 100.0).with_price_domain(0.0, 50.0);
    let engine = ChartEngine::new(renderer, config).expect("engine init");
    let frame = engine.build_render_frame().expect("build frame");

    let mut time_xs: Vec<f64> = frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .map(|label| label.x)
        .collect();
    let mut price_ys: Vec<f64> = frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Right)
        .map(|label| label.y + 8.0)
        .collect();

    time_xs.sort_by(f64::total_cmp);
    price_ys.sort_by(f64::total_cmp);

    assert!(time_xs.windows(2).all(|pair| pair[1] - pair[0] >= 56.0));
    assert!(price_ys.windows(2).all(|pair| pair[1] - pair[0] >= 22.0));
    assert!(
        time_xs.len() <= 4,
        "time labels should be compact on narrow views"
    );
    assert!(
        price_ys.len() <= 5,
        "price labels should be compact on narrow views"
    );
}

#[test]
fn wide_viewport_produces_more_labels_than_narrow_viewport() {
    let narrow = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(180, 120), 0.0, 100.0).with_price_domain(0.0, 50.0),
    )
    .expect("narrow engine");
    let wide = ChartEngine::new(
        NullRenderer::default(),
        ChartEngineConfig::new(Viewport::new(1200, 900), 0.0, 100.0).with_price_domain(0.0, 50.0),
    )
    .expect("wide engine");

    let narrow_frame = narrow.build_render_frame().expect("narrow frame");
    let wide_frame = wide.build_render_frame().expect("wide frame");

    let narrow_time = narrow_frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .count();
    let wide_time = wide_frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .count();

    let narrow_price = narrow_frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Right)
        .count();
    let wide_price = wide_frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Right)
        .count();

    assert!(wide_time > narrow_time);
    assert!(wide_price > narrow_price);
}
