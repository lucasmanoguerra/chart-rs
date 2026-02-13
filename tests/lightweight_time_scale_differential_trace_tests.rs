use chart_rs::api::{
    ChartEngine, ChartEngineConfig, TimeScaleNavigationBehavior, TimeScaleScrollZoomBehavior,
};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::NullRenderer;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DifferentialTraceFile {
    trace_name: String,
    source: String,
    source_notes: String,
    viewport: DifferentialViewport,
    time_range: DifferentialTimeRange,
    price_range: DifferentialPriceRange,
    tolerance: f64,
    scenarios: Vec<DifferentialScenario>,
}

#[derive(Debug, Deserialize)]
struct DifferentialViewport {
    width: u32,
    height: u32,
}

#[derive(Debug, Deserialize)]
struct DifferentialTimeRange {
    start: f64,
    end: f64,
}

#[derive(Debug, Deserialize)]
struct DifferentialPriceRange {
    min: f64,
    max: f64,
}

#[derive(Debug, Deserialize)]
struct DifferentialScenario {
    id: String,
    points: Vec<DifferentialPoint>,
    steps: Vec<DifferentialStep>,
}

#[derive(Debug, Deserialize)]
struct DifferentialPoint {
    time: f64,
    value: f64,
}

#[derive(Debug, Deserialize)]
struct DifferentialStep {
    action: DifferentialAction,
    expect: DifferentialExpectation,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DifferentialAction {
    SetNavigation {
        right_offset_bars: f64,
        bar_spacing_px: Option<f64>,
    },
    SetRightOffsetPx {
        value: f64,
    },
    SetScrollZoomBehavior {
        right_bar_stays_on_scroll: bool,
    },
    PanByPixels {
        delta_px: f64,
    },
    WheelZoom {
        wheel_delta_y: f64,
        anchor_px: f64,
        zoom_step_ratio: f64,
        min_span_absolute: f64,
    },
}

#[derive(Debug, Deserialize)]
struct DifferentialExpectation {
    visible_start: Option<f64>,
    visible_end: Option<f64>,
    visible_span: Option<f64>,
    right_margin_px: Option<f64>,
    scroll_position_bars: Option<f64>,
}

fn load_trace() -> DifferentialTraceFile {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/lightweight_differential/time_scale_zoom_pan_right_offset_pixels_trace.json"
    );
    let raw = std::fs::read_to_string(path).expect("trace fixture exists");
    serde_json::from_str(&raw).expect("trace fixture parses")
}

fn build_engine(trace: &DifferentialTraceFile) -> ChartEngine<NullRenderer> {
    let config = ChartEngineConfig::new(
        Viewport::new(trace.viewport.width, trace.viewport.height),
        trace.time_range.start,
        trace.time_range.end,
    )
    .with_price_domain(trace.price_range.min, trace.price_range.max);
    ChartEngine::new(NullRenderer::default(), config).expect("engine init")
}

fn right_margin_px(engine: &ChartEngine<NullRenderer>) -> f64 {
    let width = f64::from(engine.viewport().width).max(1.0);
    let (visible_start, visible_end) = engine.time_visible_range();
    let (_, full_end) = engine.time_full_range();
    let span = (visible_end - visible_start).max(1e-12);
    ((visible_end - full_end) / span) * width
}

fn assert_close(actual: f64, expected: f64, tolerance: f64, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "{context}: expected={expected}, actual={actual}, delta={delta}, tolerance={tolerance}"
    );
}

#[test]
fn lightweight_v51_time_scale_differential_trace_zoom_pan_right_offset_pixels() {
    let trace = load_trace();
    assert!(!trace.trace_name.is_empty());
    assert!(trace.source.contains("lightweight"));
    assert!(!trace.source_notes.is_empty());

    for scenario in &trace.scenarios {
        let mut engine = build_engine(&trace);
        if !scenario.points.is_empty() {
            let points = scenario
                .points
                .iter()
                .map(|point| DataPoint::new(point.time, point.value))
                .collect::<Vec<_>>();
            engine.set_data(points);
        }

        for (step_idx, step) in scenario.steps.iter().enumerate() {
            match step.action {
                DifferentialAction::SetNavigation {
                    right_offset_bars,
                    bar_spacing_px,
                } => {
                    engine
                        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
                            right_offset_bars,
                            bar_spacing_px,
                        })
                        .expect("set navigation behavior");
                }
                DifferentialAction::SetRightOffsetPx { value } => {
                    engine
                        .set_time_scale_right_offset_px(Some(value))
                        .expect("set right offset px");
                }
                DifferentialAction::SetScrollZoomBehavior {
                    right_bar_stays_on_scroll,
                } => {
                    engine
                        .set_time_scale_scroll_zoom_behavior(TimeScaleScrollZoomBehavior {
                            right_bar_stays_on_scroll,
                        })
                        .expect("set scroll zoom behavior");
                }
                DifferentialAction::PanByPixels { delta_px } => {
                    engine
                        .pan_time_visible_by_pixels(delta_px)
                        .expect("pan by pixels");
                }
                DifferentialAction::WheelZoom {
                    wheel_delta_y,
                    anchor_px,
                    zoom_step_ratio,
                    min_span_absolute,
                } => {
                    let _ = engine
                        .wheel_zoom_time_visible(
                            wheel_delta_y,
                            anchor_px,
                            zoom_step_ratio,
                            min_span_absolute,
                        )
                        .expect("wheel zoom");
                }
            }

            let (visible_start, visible_end) = engine.time_visible_range();
            let visible_span = visible_end - visible_start;
            let margin_px = right_margin_px(&engine);
            let scroll_position_bars = engine.time_scroll_position_bars();

            let context = format!("scenario={}, step={step_idx}", scenario.id);
            if let Some(expected) = step.expect.visible_start {
                assert_close(
                    visible_start,
                    expected,
                    trace.tolerance,
                    &format!("{context}, visible_start"),
                );
            }
            if let Some(expected) = step.expect.visible_end {
                assert_close(
                    visible_end,
                    expected,
                    trace.tolerance,
                    &format!("{context}, visible_end"),
                );
            }
            if let Some(expected) = step.expect.visible_span {
                assert_close(
                    visible_span,
                    expected,
                    trace.tolerance,
                    &format!("{context}, visible_span"),
                );
            }
            if let Some(expected) = step.expect.right_margin_px {
                assert_close(
                    margin_px,
                    expected,
                    trace.tolerance,
                    &format!("{context}, right_margin_px"),
                );
            }
            if let Some(expected) = step.expect.scroll_position_bars {
                let actual = scroll_position_bars.expect("scroll position requires data step");
                assert_close(
                    actual,
                    expected,
                    trace.tolerance,
                    &format!("{context}, scroll_position_bars"),
                );
            }
        }
    }
}
