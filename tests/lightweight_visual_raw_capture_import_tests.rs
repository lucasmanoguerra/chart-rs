use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ImportedVisualCorpus {
    schema_version: u32,
    source: String,
    source_notes: String,
    fixtures: Vec<ImportedVisualFixture>,
}

#[derive(Debug, Deserialize)]
struct ImportedVisualFixture {
    id: String,
    baseline_png_relpath: String,
    input: ImportedVisualInput,
}

#[derive(Debug, Deserialize)]
struct ImportedVisualInput {
    actions: Vec<ImportedVisualAction>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ImportedVisualAction {
    SetPriceScaleMode {
        mode: String,
    },
    SetTimeVisibleRange {
        start: f64,
        end: f64,
    },
    AxisDragScalePrice {
        drag_delta_y_px: f64,
        anchor_y_px: f64,
    },
    AxisDragScaleTime {
        drag_delta_x_px: f64,
        anchor_x_px: f64,
    },
    SetCrosshairMode {
        mode: String,
    },
    PointerMove {
        x: f64,
        y: f64,
    },
}

#[test]
fn lightweight_real_visual_capture_import_maps_events_into_visual_corpus_without_manual_normalization()
 {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/lightweight_visual_differential/lightweight_real_capture_visual_imported_corpus.json"
    );
    let raw = std::fs::read_to_string(path).expect("imported visual corpus fixture exists");
    let corpus: ImportedVisualCorpus = serde_json::from_str(&raw).expect("corpus parses");

    assert_eq!(corpus.schema_version, 1);
    assert!(corpus.source.contains("lightweight"));
    assert!(corpus.source_notes.contains("without manual normalization"));
    assert_eq!(corpus.fixtures.len(), 2);

    let first = &corpus.fixtures[0];
    assert_eq!(first.id, "lwc-real-capture-candles-log-visual");
    assert!(
        first
            .baseline_png_relpath
            .ends_with("/lwc-real-capture-candles-log-visual.png")
    );
    assert_eq!(first.input.actions.len(), 5);
    assert!(matches!(
        first.input.actions[0],
        ImportedVisualAction::SetPriceScaleMode { ref mode } if mode == "log"
    ));
    assert!(matches!(
        first.input.actions[1],
        ImportedVisualAction::AxisDragScalePrice {
            drag_delta_y_px: 80.0,
            anchor_y_px: 360.0
        }
    ));
    assert!(matches!(
        first.input.actions[2],
        ImportedVisualAction::AxisDragScalePrice {
            drag_delta_y_px: -80.0,
            anchor_y_px: 360.0
        }
    ));
    assert!(matches!(
        first.input.actions[3],
        ImportedVisualAction::SetCrosshairMode { ref mode } if mode == "magnet"
    ));
    assert!(matches!(
        first.input.actions[4],
        ImportedVisualAction::PointerMove { x: 530.0, y: 170.0 }
    ));

    let second = &corpus.fixtures[1];
    assert_eq!(second.id, "lwc-real-capture-session-timezone-visual");
    assert!(
        second
            .baseline_png_relpath
            .ends_with("/lwc-real-capture-session-timezone-visual.png")
    );
    assert_eq!(second.input.actions.len(), 5);
    assert!(matches!(
        second.input.actions[0],
        ImportedVisualAction::SetTimeVisibleRange {
            start: 1704205950.0,
            end: 1704207450.0
        }
    ));
    assert!(matches!(
        second.input.actions[1],
        ImportedVisualAction::AxisDragScaleTime {
            drag_delta_x_px: 180.0,
            anchor_x_px: 540.0
        }
    ));
    assert!(matches!(
        second.input.actions[2],
        ImportedVisualAction::AxisDragScaleTime {
            drag_delta_x_px: -300.0,
            anchor_x_px: 540.0
        }
    ));
    assert!(matches!(
        second.input.actions[3],
        ImportedVisualAction::SetCrosshairMode { ref mode } if mode == "magnet"
    ));
    assert!(matches!(
        second.input.actions[4],
        ImportedVisualAction::PointerMove { x: 520.0, y: 210.0 }
    ));
}
