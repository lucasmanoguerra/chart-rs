use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ImportedTraceFile {
    source: String,
    source_notes: String,
    scenarios: Vec<ImportedScenario>,
}

#[derive(Debug, Deserialize)]
struct ImportedScenario {
    id: String,
    steps: Vec<ImportedStep>,
}

#[derive(Debug, Deserialize)]
struct ImportedStep {
    action: ImportedAction,
    #[serde(default)]
    expect: Option<ImportedExpectation>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ImportedAction {
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
    SetCrosshairMode {
        mode: String,
    },
    PointerMove {
        x: f64,
        y: f64,
    },
    PointerLeave,
}

#[derive(Debug, Deserialize, Default)]
struct ImportedExpectation {
    visible_start: Option<f64>,
    visible_end: Option<f64>,
    visible_span: Option<f64>,
    scroll_position_bars: Option<f64>,
}

#[test]
fn lightweight_real_capture_import_maps_wheel_touch_crosshair_without_manual_normalization() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/lightweight_differential/lightweight_real_capture_interaction_trace.json"
    );
    let raw = std::fs::read_to_string(path).expect("imported trace fixture exists");
    let trace: ImportedTraceFile = serde_json::from_str(&raw).expect("trace parses");

    assert!(trace.source.contains("lightweight"));
    assert!(trace.source_notes.contains("without manual normalization"));
    assert_eq!(trace.scenarios.len(), 1);

    let scenario = &trace.scenarios[0];
    assert_eq!(scenario.id, "lightweight-scenario-1");
    assert_eq!(scenario.steps.len(), 7);

    match scenario.steps[0].action {
        ImportedAction::SetNavigation {
            right_offset_bars,
            bar_spacing_px,
        } => {
            assert_eq!(right_offset_bars, 0.0);
            assert!(bar_spacing_px.is_none());
        }
        _ => panic!("step0 should map to set_navigation"),
    }

    match scenario.steps[1].action {
        ImportedAction::WheelPan {
            wheel_delta_x,
            pan_step_ratio,
        } => {
            assert_eq!(wheel_delta_x, 120.0);
            assert_eq!(pan_step_ratio, 0.1);
        }
        _ => panic!("step1 should map to wheel_pan"),
    }

    match scenario.steps[2].action {
        ImportedAction::TouchDragPan {
            delta_x_px,
            delta_y_px,
        } => {
            assert_eq!(delta_x_px, 20.0);
            assert_eq!(delta_y_px, 6.0);
        }
        _ => panic!("step2 should map to touch_drag_pan"),
    }

    match scenario.steps[3].action {
        ImportedAction::WheelZoom {
            wheel_delta_y,
            anchor_px,
            zoom_step_ratio,
            min_span_absolute,
        } => {
            assert_eq!(wheel_delta_y, -120.0);
            assert_eq!(anchor_px, 500.0);
            assert_eq!(zoom_step_ratio, 0.2);
            assert_eq!(min_span_absolute, 1e-6);
        }
        _ => panic!("step3 should map to wheel_zoom"),
    }

    match &scenario.steps[4].action {
        ImportedAction::SetCrosshairMode { mode } => {
            assert_eq!(mode, "magnet");
        }
        _ => panic!("step4 should map to set_crosshair_mode"),
    }

    match scenario.steps[5].action {
        ImportedAction::PointerMove { x, y } => {
            assert_eq!(x, 333.0);
            assert_eq!(y, 111.0);
        }
        _ => panic!("step5 should map to pointer_move"),
    }
    assert!(matches!(
        scenario.steps[6].action,
        ImportedAction::PointerLeave
    ));

    let first_expect = scenario.steps[0]
        .expect
        .as_ref()
        .expect("first step should carry observed payload");
    assert_eq!(first_expect.visible_start, Some(0.0));
    assert_eq!(first_expect.visible_end, Some(100.0));
    assert_eq!(first_expect.visible_span, Some(100.0));

    let second_expect = scenario.steps[1]
        .expect
        .as_ref()
        .expect("second step should carry observed payload");
    assert_eq!(second_expect.scroll_position_bars, Some(1.0));

    assert!(scenario.steps[2].expect.is_none());
    assert!(scenario.steps[3].expect.is_none());
    assert!(scenario.steps[4].expect.is_none());
    assert!(scenario.steps[5].expect.is_none());
    assert!(scenario.steps[6].expect.is_none());
}
