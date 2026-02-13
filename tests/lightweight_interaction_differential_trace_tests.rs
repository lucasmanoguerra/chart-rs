use chart_rs::api::{ChartEngine, ChartEngineConfig, TimeScaleNavigationBehavior};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::interaction::CrosshairMode;
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

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TraceCrosshairMode {
    Magnet,
    Normal,
    Hidden,
}

impl From<TraceCrosshairMode> for CrosshairMode {
    fn from(value: TraceCrosshairMode) -> Self {
        match value {
            TraceCrosshairMode::Magnet => Self::Magnet,
            TraceCrosshairMode::Normal => Self::Normal,
            TraceCrosshairMode::Hidden => Self::Hidden,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DifferentialAction {
    SetNavigation {
        right_offset_bars: f64,
        bar_spacing_px: Option<f64>,
    },
    WheelPan {
        wheel_delta_x: f64,
        pan_step_ratio: f64,
    },
    TouchDragPan {
        delta_x_px: f64,
        delta_y_px: f64,
    },
    WheelZoom {
        wheel_delta_y: f64,
        anchor_px: f64,
        zoom_step_ratio: f64,
        min_span_absolute: f64,
    },
    PinchZoom {
        factor: f64,
        anchor_px: f64,
        min_span_absolute: f64,
    },
    StartKineticPan {
        velocity_time_per_sec: f64,
    },
    StepKineticPan {
        delta_seconds: f64,
    },
    SetCrosshairMode {
        mode: TraceCrosshairMode,
    },
    PointerMove {
        x: f64,
        y: f64,
    },
    PointerLeave,
}

#[derive(Debug, Deserialize, Default)]
struct DifferentialExpectation {
    visible_start: Option<f64>,
    visible_end: Option<f64>,
    visible_span: Option<f64>,
    scroll_position_bars: Option<f64>,
    kinetic_active: Option<bool>,
    kinetic_velocity_time_per_sec: Option<f64>,
    crosshair_visible: Option<bool>,
    crosshair_x: Option<f64>,
    crosshair_y: Option<f64>,
    crosshair_snapped_x: Option<f64>,
    crosshair_snapped_y: Option<f64>,
    crosshair_snapped_time: Option<f64>,
    crosshair_snapped_price: Option<f64>,
}

fn load_trace() -> DifferentialTraceFile {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/lightweight_differential/interaction_zoom_pan_kinetic_crosshair_trace.json"
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

fn assert_close(actual: f64, expected: f64, tolerance: f64, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "{context}: expected={expected}, actual={actual}, delta={delta}, tolerance={tolerance}"
    );
}

#[test]
fn lightweight_v51_interaction_differential_trace_zoom_pan_kinetic_crosshair() {
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
                DifferentialAction::WheelPan {
                    wheel_delta_x,
                    pan_step_ratio,
                } => {
                    let _ = engine
                        .wheel_pan_time_visible(wheel_delta_x, pan_step_ratio)
                        .expect("wheel pan");
                }
                DifferentialAction::TouchDragPan {
                    delta_x_px,
                    delta_y_px,
                } => {
                    let _ = engine
                        .touch_drag_pan_time_visible(delta_x_px, delta_y_px)
                        .expect("touch drag pan");
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
                DifferentialAction::PinchZoom {
                    factor,
                    anchor_px,
                    min_span_absolute,
                } => {
                    let _ = engine
                        .pinch_zoom_time_visible(factor, anchor_px, min_span_absolute)
                        .expect("pinch zoom");
                }
                DifferentialAction::StartKineticPan {
                    velocity_time_per_sec,
                } => {
                    engine
                        .start_kinetic_pan(velocity_time_per_sec)
                        .expect("start kinetic pan");
                }
                DifferentialAction::StepKineticPan { delta_seconds } => {
                    let _ = engine
                        .step_kinetic_pan(delta_seconds)
                        .expect("step kinetic pan");
                }
                DifferentialAction::SetCrosshairMode { mode } => {
                    engine.set_crosshair_mode(mode.into());
                }
                DifferentialAction::PointerMove { x, y } => {
                    engine.pointer_move(x, y);
                }
                DifferentialAction::PointerLeave => {
                    engine.pointer_leave();
                }
            }

            let (visible_start, visible_end) = engine.time_visible_range();
            let visible_span = visible_end - visible_start;
            let scroll_position_bars = engine.time_scroll_position_bars();
            let kinetic = engine.kinetic_pan_state();
            let crosshair = engine.crosshair_state();

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
            if let Some(expected) = step.expect.scroll_position_bars {
                let actual = scroll_position_bars.expect("scroll position requires data step");
                assert_close(
                    actual,
                    expected,
                    trace.tolerance,
                    &format!("{context}, scroll_position_bars"),
                );
            }
            if let Some(expected) = step.expect.kinetic_active {
                assert_eq!(
                    kinetic.active, expected,
                    "{context}, kinetic_active: expected={expected}, actual={}",
                    kinetic.active
                );
            }
            if let Some(expected) = step.expect.kinetic_velocity_time_per_sec {
                assert_close(
                    kinetic.velocity_time_per_sec,
                    expected,
                    trace.tolerance,
                    &format!("{context}, kinetic_velocity_time_per_sec"),
                );
            }
            if let Some(expected) = step.expect.crosshair_visible {
                assert_eq!(
                    crosshair.visible, expected,
                    "{context}, crosshair_visible: expected={expected}, actual={}",
                    crosshair.visible
                );
            }
            if let Some(expected) = step.expect.crosshair_x {
                assert_close(
                    crosshair.x,
                    expected,
                    trace.tolerance,
                    &format!("{context}, crosshair_x"),
                );
            }
            if let Some(expected) = step.expect.crosshair_y {
                assert_close(
                    crosshair.y,
                    expected,
                    trace.tolerance,
                    &format!("{context}, crosshair_y"),
                );
            }
            if let Some(expected) = step.expect.crosshair_snapped_x {
                let actual = crosshair
                    .snapped_x
                    .expect("expected crosshair snapped_x to exist");
                assert_close(
                    actual,
                    expected,
                    trace.tolerance,
                    &format!("{context}, crosshair_snapped_x"),
                );
            } else {
                assert!(
                    crosshair.snapped_x.is_none(),
                    "{context}, crosshair_snapped_x expected None, actual={:?}",
                    crosshair.snapped_x
                );
            }
            if let Some(expected) = step.expect.crosshair_snapped_y {
                let actual = crosshair
                    .snapped_y
                    .expect("expected crosshair snapped_y to exist");
                assert_close(
                    actual,
                    expected,
                    trace.tolerance,
                    &format!("{context}, crosshair_snapped_y"),
                );
            } else {
                assert!(
                    crosshair.snapped_y.is_none(),
                    "{context}, crosshair_snapped_y expected None, actual={:?}",
                    crosshair.snapped_y
                );
            }
            if let Some(expected) = step.expect.crosshair_snapped_time {
                let actual = crosshair
                    .snapped_time
                    .expect("expected crosshair snapped_time to exist");
                assert_close(
                    actual,
                    expected,
                    trace.tolerance,
                    &format!("{context}, crosshair_snapped_time"),
                );
            } else {
                assert!(
                    crosshair.snapped_time.is_none(),
                    "{context}, crosshair_snapped_time expected None, actual={:?}",
                    crosshair.snapped_time
                );
            }
            if let Some(expected) = step.expect.crosshair_snapped_price {
                let actual = crosshair
                    .snapped_price
                    .expect("expected crosshair snapped_price to exist");
                assert_close(
                    actual,
                    expected,
                    trace.tolerance,
                    &format!("{context}, crosshair_snapped_price"),
                );
            } else {
                assert!(
                    crosshair.snapped_price.is_none(),
                    "{context}, crosshair_snapped_price expected None, actual={:?}",
                    crosshair.snapped_price
                );
            }
        }
    }
}

#[test]
fn lightweight_v51_touch_interaction_trace_has_multi_step_pinch_kinetic_decay_and_gap_snap() {
    let trace = load_trace();
    let scenario = trace
        .scenarios
        .iter()
        .find(|scenario| scenario.id == "touch-pinch-kinetic-gap-snap-advanced")
        .expect("advanced touch scenario exists");

    let pinch_zoom_steps = scenario
        .steps
        .iter()
        .filter(|step| matches!(step.action, DifferentialAction::PinchZoom { .. }))
        .count();
    assert!(
        pinch_zoom_steps >= 3,
        "expected at least 3 pinch steps, got {pinch_zoom_steps}"
    );

    let kinetic_velocities = scenario
        .steps
        .iter()
        .filter_map(|step| match step.action {
            DifferentialAction::StepKineticPan { .. } => step.expect.kinetic_velocity_time_per_sec,
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        kinetic_velocities.len() >= 3,
        "expected at least 3 kinetic-step velocities"
    );
    assert!(
        kinetic_velocities
            .windows(2)
            .all(|pair| pair[1] <= pair[0] + trace.tolerance),
        "kinetic velocity should decay monotonically: {kinetic_velocities:?}"
    );

    let gap_snap_step = scenario
        .steps
        .iter()
        .find(|step| matches!(step.action, DifferentialAction::PointerMove { .. }))
        .expect("pointer_move step exists");
    let snapped_time = gap_snap_step
        .expect
        .crosshair_snapped_time
        .expect("pointer move should produce snapped time in magnet mode");
    let snapped_x = gap_snap_step
        .expect
        .crosshair_snapped_x
        .expect("pointer move should produce snapped x");
    let crosshair_x = gap_snap_step
        .expect
        .crosshair_x
        .expect("pointer move should report crosshair x");

    assert_eq!(
        snapped_time, 78.0,
        "expected sparse-gap snap target at time=78"
    );
    assert!(
        (snapped_x - crosshair_x).abs() > 1.0,
        "gap snap should move to nearest filled slot; snapped_x={snapped_x}, crosshair_x={crosshair_x}"
    );
}
