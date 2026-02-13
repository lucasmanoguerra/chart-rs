use chart_rs::api::{
    ChartEngine, ChartEngineConfig, RenderStyle, TimeAxisLabelConfig, TimeAxisLabelPolicy,
};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::{NullRenderer, TextHAlign};

fn time_label_count(frame: &chart_rs::render::RenderFrame) -> usize {
    frame
        .texts
        .iter()
        .filter(|text| text.h_align == TextHAlign::Center)
        .count()
}

fn price_label_count(frame: &chart_rs::render::RenderFrame) -> usize {
    frame
        .texts
        .iter()
        .filter(|text| text.h_align == TextHAlign::Right)
        .count()
}

#[test]
fn lightweight_v51_reference_time_axis_tick_cadence_is_zoom_monotonic() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1100, 420), 0.0, 10_000.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            policy: TimeAxisLabelPolicy::LogicalDecimal { precision: 0 },
            ..TimeAxisLabelConfig::default()
        })
        .expect("time-axis policy");
    let points: Vec<DataPoint> = (0..1_000)
        .map(|i| {
            let time = i as f64 * 10.0;
            DataPoint::new(time, 0.5 + (time * 0.002).sin() * 0.2)
        })
        .collect();
    engine.set_data(points);

    engine
        .set_time_visible_range(0.0, 10_000.0)
        .expect("zoomed out");
    let out_count = time_label_count(&engine.build_render_frame().expect("frame"));

    engine
        .set_time_visible_range(2_000.0, 6_000.0)
        .expect("mid zoom");
    let mid_count = time_label_count(&engine.build_render_frame().expect("frame"));

    engine
        .set_time_visible_range(4_500.0, 5_100.0)
        .expect("zoomed in");
    let in_count = time_label_count(&engine.build_render_frame().expect("frame"));

    // Lightweight Charts cadence expectation: zoom-in should permit denser marks.
    assert!(out_count < mid_count, "out={out_count}, mid={mid_count}");
    assert!(mid_count <= in_count, "mid={mid_count}, in={in_count}");
    assert!(in_count >= out_count + 3, "out={out_count}, in={in_count}");
}

#[test]
fn lightweight_v51_reference_time_axis_tick_cadence_tracks_intermediate_zoom_windows() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(1100, 420), 0.0, 10_000.0).with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            policy: TimeAxisLabelPolicy::LogicalDecimal { precision: 0 },
            ..TimeAxisLabelConfig::default()
        })
        .expect("time-axis policy");
    let points: Vec<DataPoint> = (0..1_000)
        .map(|i| {
            let time = i as f64 * 10.0;
            DataPoint::new(time, 0.5 + (time * 0.002).sin() * 0.2)
        })
        .collect();
    engine.set_data(points);

    engine
        .set_time_visible_range(0.0, 10_000.0)
        .expect("zoomed out");
    let out_count = time_label_count(&engine.build_render_frame().expect("frame"));

    engine
        .set_time_visible_range(1_500.0, 7_500.0)
        .expect("mid-1 zoom");
    let mid1_count = time_label_count(&engine.build_render_frame().expect("frame"));

    engine
        .set_time_visible_range(2_500.0, 5_500.0)
        .expect("mid-2 zoom");
    let mid2_count = time_label_count(&engine.build_render_frame().expect("frame"));

    engine
        .set_time_visible_range(4_300.0, 5_200.0)
        .expect("near zoom-in");
    let in1_count = time_label_count(&engine.build_render_frame().expect("frame"));

    engine
        .set_time_visible_range(4_700.0, 5_050.0)
        .expect("deep zoom-in");
    let in2_count = time_label_count(&engine.build_render_frame().expect("frame"));

    assert!(out_count < mid1_count, "out={out_count}, mid1={mid1_count}");
    assert!(
        mid1_count <= mid2_count,
        "mid1={mid1_count}, mid2={mid2_count}"
    );
    assert!(
        mid2_count <= in1_count,
        "mid2={mid2_count}, in1={in1_count}"
    );
    assert!(in1_count <= in2_count, "in1={in1_count}, in2={in2_count}");
    assert!(
        in2_count >= out_count + 3,
        "out={out_count}, in2={in2_count}"
    );
}

#[test]
fn lightweight_v51_reference_price_axis_tick_cadence_is_scale_zoom_monotonic() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 100.0).with_price_domain(80.0, 120.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(0.0, 90.0),
        DataPoint::new(25.0, 96.0),
        DataPoint::new(50.0, 104.0),
        DataPoint::new(75.0, 108.0),
        DataPoint::new(100.0, 113.0),
    ]);
    engine
        .set_render_style(RenderStyle {
            show_last_price_label: false,
            show_last_price_line: false,
            ..engine.render_style()
        })
        .expect("style");

    let baseline_count = price_label_count(&engine.build_render_frame().expect("baseline frame"));

    let _ = engine
        .axis_drag_scale_price(360.0, 210.0, 0.2, 1e-6)
        .expect("zoom out");
    let out_count = price_label_count(&engine.build_render_frame().expect("zoomed-out frame"));

    let _ = engine
        .axis_drag_scale_price(-720.0, 210.0, 0.2, 1e-6)
        .expect("zoom in");
    let in_count = price_label_count(&engine.build_render_frame().expect("zoomed-in frame"));

    // Lightweight Charts cadence expectation: tighter price range allows denser labels.
    assert!(
        out_count < baseline_count,
        "out={out_count}, baseline={baseline_count}"
    );
    assert!(
        baseline_count < in_count,
        "baseline={baseline_count}, in={in_count}"
    );
}

#[test]
fn lightweight_v51_reference_price_axis_tick_cadence_tracks_multi_step_scale_zoom() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 100.0).with_price_domain(80.0, 120.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_data(vec![
        DataPoint::new(0.0, 90.0),
        DataPoint::new(25.0, 96.0),
        DataPoint::new(50.0, 104.0),
        DataPoint::new(75.0, 108.0),
        DataPoint::new(100.0, 113.0),
    ]);
    engine
        .set_render_style(RenderStyle {
            show_last_price_label: false,
            show_last_price_line: false,
            ..engine.render_style()
        })
        .expect("style");

    let baseline = price_label_count(&engine.build_render_frame().expect("baseline frame"));

    let _ = engine
        .axis_drag_scale_price(240.0, 210.0, 0.2, 1e-6)
        .expect("zoom-out step 1");
    let out1 = price_label_count(&engine.build_render_frame().expect("zoom-out frame 1"));

    let _ = engine
        .axis_drag_scale_price(240.0, 210.0, 0.2, 1e-6)
        .expect("zoom-out step 2");
    let out2 = price_label_count(&engine.build_render_frame().expect("zoom-out frame 2"));

    let _ = engine
        .axis_drag_scale_price(-840.0, 210.0, 0.2, 1e-6)
        .expect("zoom-in step 1");
    let in1 = price_label_count(&engine.build_render_frame().expect("zoom-in frame 1"));

    let _ = engine
        .axis_drag_scale_price(-720.0, 210.0, 0.2, 1e-6)
        .expect("zoom-in step 2");
    let in2 = price_label_count(&engine.build_render_frame().expect("zoom-in frame 2"));

    assert!(out2 <= out1, "out2={out2}, out1={out1}");
    assert!(out1 < baseline, "out1={out1}, baseline={baseline}");
    assert!(baseline < in1, "baseline={baseline}, in1={in1}");
    assert!(in1 <= in2, "in1={in1}, in2={in2}");
    assert!(in2 >= baseline + 2, "baseline={baseline}, in2={in2}");
}
