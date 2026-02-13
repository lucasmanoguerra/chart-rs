#![cfg(feature = "cairo-backend")]

use cairo::ImageSurface;
use chart_rs::api::{
    ChartEngine, ChartEngineConfig, PriceAxisLabelConfig, PriceScaleTransformedBaseBehavior,
    PriceScaleTransformedBaseSource, TimeAxisLabelConfig,
};
use chart_rs::core::{DataPoint, OhlcBar, PriceScaleMode, Viewport};
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::{CairoRenderer, NullRenderer, Renderer};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};

const VISUAL_CORPUS_JSON: &str =
    include_str!("fixtures/lightweight_visual_differential/visual_baseline_corpus.json");

#[derive(Debug, Deserialize)]
struct VisualCorpus {
    schema_version: u32,
    fixtures: Vec<VisualFixture>,
}

#[derive(Debug, Deserialize)]
struct VisualFixture {
    id: String,
    description: String,
    input: VisualInput,
    baseline_png_relpath: String,
    tolerance: VisualTolerance,
}

#[derive(Debug, Deserialize)]
struct VisualInput {
    viewport: Viewport,
    time_range: [f64; 2],
    price_range: [f64; 2],
    #[serde(default)]
    points: Vec<DataPoint>,
    #[serde(default)]
    candles: Vec<OhlcBar>,
    #[serde(default)]
    time_axis_label_config: Option<TimeAxisLabelConfig>,
    #[serde(default)]
    price_axis_label_config: Option<PriceAxisLabelConfig>,
    actions: Vec<VisualAction>,
}

#[derive(Debug, Deserialize)]
struct VisualTolerance {
    max_channel_abs_diff: u8,
    mean_channel_abs_diff: f64,
}

#[derive(Debug, Serialize)]
struct VisualDiffArtifactSummary {
    fixtures: Vec<VisualDiffArtifactFixture>,
}

#[derive(Debug, Serialize)]
struct VisualDiffArtifactFixture {
    id: String,
    description: String,
    max_channel_abs_diff: u8,
    mean_channel_abs_diff: f64,
    tolerance_max_channel_abs_diff: u8,
    tolerance_mean_channel_abs_diff: f64,
    pass: bool,
    actual_png: String,
    baseline_png: String,
    diff_png: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TracePriceScaleMode {
    Linear,
    Log,
    Percentage,
    IndexedTo100,
}

impl From<TracePriceScaleMode> for PriceScaleMode {
    fn from(value: TracePriceScaleMode) -> Self {
        match value {
            TracePriceScaleMode::Linear => Self::Linear,
            TracePriceScaleMode::Log => Self::Log,
            TracePriceScaleMode::Percentage => Self::Percentage,
            TracePriceScaleMode::IndexedTo100 => Self::IndexedTo100,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TraceTransformedBaseSource {
    DomainStart,
    FirstData,
    LastData,
    FirstVisibleData,
    LastVisibleData,
}

impl From<TraceTransformedBaseSource> for PriceScaleTransformedBaseSource {
    fn from(value: TraceTransformedBaseSource) -> Self {
        match value {
            TraceTransformedBaseSource::DomainStart => Self::DomainStart,
            TraceTransformedBaseSource::FirstData => Self::FirstData,
            TraceTransformedBaseSource::LastData => Self::LastData,
            TraceTransformedBaseSource::FirstVisibleData => Self::FirstVisibleData,
            TraceTransformedBaseSource::LastVisibleData => Self::LastVisibleData,
        }
    }
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
enum VisualAction {
    SetTimeVisibleRange {
        start: f64,
        end: f64,
    },
    SetPriceScaleMode {
        mode: TracePriceScaleMode,
    },
    SetTransformedBaseBehavior {
        explicit_base_price: Option<f64>,
        dynamic_source: TraceTransformedBaseSource,
    },
    AutoscaleVisibleData,
    SetCrosshairMode {
        mode: TraceCrosshairMode,
    },
    PointerMove {
        x: f64,
        y: f64,
    },
    AxisDragScalePrice {
        drag_delta_y_px: f64,
        anchor_y_px: f64,
        zoom_step_ratio: f64,
        min_span_absolute: f64,
    },
    AxisDragScaleTime {
        drag_delta_x_px: f64,
        anchor_x_px: f64,
        zoom_step_ratio: f64,
        min_span_absolute: f64,
    },
}

fn load_visual_corpus() -> VisualCorpus {
    let corpus: VisualCorpus =
        serde_json::from_str(VISUAL_CORPUS_JSON).expect("visual fixture corpus should parse");
    assert_eq!(
        corpus.schema_version, 1,
        "unexpected visual fixture schema version"
    );
    assert!(
        !corpus.fixtures.is_empty(),
        "visual fixture corpus should not be empty"
    );
    corpus
}

fn render_fixture_png_bytes(fixture: &VisualFixture) -> Vec<u8> {
    let input = &fixture.input;
    let config = ChartEngineConfig::new(input.viewport, input.time_range[0], input.time_range[1])
        .with_price_domain(input.price_range[0], input.price_range[1]);

    let mut engine =
        ChartEngine::new(NullRenderer::default(), config).expect("visual fixture engine init");
    if !input.points.is_empty() {
        engine.set_data(input.points.clone());
    }
    if !input.candles.is_empty() {
        engine.set_candles(input.candles.clone());
    }
    if let Some(config) = input.time_axis_label_config {
        engine
            .set_time_axis_label_config(config)
            .expect("set time axis label config");
    }
    if let Some(config) = input.price_axis_label_config {
        engine
            .set_price_axis_label_config(config)
            .expect("set price axis label config");
    }

    for action in &input.actions {
        match *action {
            VisualAction::SetTimeVisibleRange { start, end } => {
                engine
                    .set_time_visible_range(start, end)
                    .expect("set time visible range");
            }
            VisualAction::SetPriceScaleMode { mode } => {
                engine
                    .set_price_scale_mode(mode.into())
                    .expect("set price scale mode");
            }
            VisualAction::SetTransformedBaseBehavior {
                explicit_base_price,
                dynamic_source,
            } => {
                engine
                    .set_price_scale_transformed_base_behavior(PriceScaleTransformedBaseBehavior {
                        explicit_base_price,
                        dynamic_source: dynamic_source.into(),
                    })
                    .expect("set transformed base behavior");
            }
            VisualAction::AutoscaleVisibleData => {
                engine
                    .autoscale_price_from_visible_data()
                    .expect("autoscale visible data");
            }
            VisualAction::SetCrosshairMode { mode } => {
                engine.set_crosshair_mode(mode.into());
            }
            VisualAction::PointerMove { x, y } => {
                engine.pointer_move(x, y);
            }
            VisualAction::AxisDragScalePrice {
                drag_delta_y_px,
                anchor_y_px,
                zoom_step_ratio,
                min_span_absolute,
            } => {
                let _ = engine
                    .axis_drag_scale_price(
                        drag_delta_y_px,
                        anchor_y_px,
                        zoom_step_ratio,
                        min_span_absolute,
                    )
                    .expect("axis drag scale price");
            }
            VisualAction::AxisDragScaleTime {
                drag_delta_x_px,
                anchor_x_px,
                zoom_step_ratio,
                min_span_absolute,
            } => {
                let _ = engine
                    .axis_drag_scale_time(
                        drag_delta_x_px,
                        anchor_x_px,
                        zoom_step_ratio,
                        min_span_absolute,
                    )
                    .expect("axis drag scale time");
            }
        }
    }

    let frame = engine.build_render_frame().expect("build render frame");
    let width = i32::try_from(input.viewport.width).expect("viewport width overflow");
    let height = i32::try_from(input.viewport.height).expect("viewport height overflow");
    let mut renderer = CairoRenderer::new(width, height).expect("cairo renderer");
    renderer.render(&frame).expect("render frame to cairo");

    let mut bytes = Vec::new();
    renderer
        .surface()
        .write_to_png(&mut bytes)
        .expect("encode png bytes");
    bytes
}

fn decode_png_surface(bytes: &[u8]) -> ImageSurface {
    let mut cursor = Cursor::new(bytes);
    ImageSurface::create_from_png(&mut cursor).expect("decode png surface")
}

fn surface_bytes(surface: &mut ImageSurface) -> (usize, usize, usize, Vec<u8>) {
    surface.flush();
    let width = usize::try_from(surface.width()).expect("width fits usize");
    let height = usize::try_from(surface.height()).expect("height fits usize");
    let stride = usize::try_from(surface.stride()).expect("stride fits usize");
    let data = surface.data().expect("surface bytes").to_vec();
    (width, height, stride, data)
}

fn compare_surfaces(actual: &mut ImageSurface, expected: &mut ImageSurface) -> (u8, f64) {
    let (actual_width, actual_height, actual_stride, actual_data) = surface_bytes(actual);
    let (expected_width, expected_height, expected_stride, expected_data) = surface_bytes(expected);

    assert_eq!(actual_width, expected_width, "surface width mismatch");
    assert_eq!(actual_height, expected_height, "surface height mismatch");

    let row_bytes = actual_width * 4;
    assert!(
        actual_stride >= row_bytes,
        "actual stride must cover visible row bytes"
    );
    assert!(
        expected_stride >= row_bytes,
        "expected stride must cover visible row bytes"
    );

    let mut max_diff = 0u8;
    let mut sum_diff = 0u64;
    let mut compared = 0u64;

    for y in 0..actual_height {
        let actual_row_start = y * actual_stride;
        let expected_row_start = y * expected_stride;
        for x in 0..row_bytes {
            let actual_byte = actual_data[actual_row_start + x];
            let expected_byte = expected_data[expected_row_start + x];
            let diff = actual_byte.abs_diff(expected_byte);
            if diff > max_diff {
                max_diff = diff;
            }
            sum_diff += u64::from(diff);
            compared += 1;
        }
    }

    let mean_diff = if compared > 0 {
        sum_diff as f64 / compared as f64
    } else {
        0.0
    };

    (max_diff, mean_diff)
}

fn sanitize_artifact_stem(id: &str) -> String {
    let mut stem = String::with_capacity(id.len());
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            stem.push(ch.to_ascii_lowercase());
        } else {
            stem.push('-');
        }
    }
    let stem = stem.trim_matches('-');
    if stem.is_empty() {
        "fixture".to_owned()
    } else {
        stem.to_owned()
    }
}

fn build_diff_png(actual: &mut ImageSurface, expected: &mut ImageSurface) -> Vec<u8> {
    let (actual_width, actual_height, actual_stride, actual_data) = surface_bytes(actual);
    let (expected_width, expected_height, expected_stride, expected_data) = surface_bytes(expected);
    assert_eq!(actual_width, expected_width, "diff width mismatch");
    assert_eq!(actual_height, expected_height, "diff height mismatch");

    let width = i32::try_from(actual_width).expect("diff width fits i32");
    let height = i32::try_from(actual_height).expect("diff height fits i32");
    let mut diff_surface =
        ImageSurface::create(cairo::Format::ARgb32, width, height).expect("create diff surface");
    let row_bytes = actual_width * 4;
    let diff_stride = usize::try_from(diff_surface.stride()).expect("diff stride fits usize");
    {
        let mut diff_data = diff_surface.data().expect("diff data");
        for y in 0..actual_height {
            let actual_row_start = y * actual_stride;
            let expected_row_start = y * expected_stride;
            let diff_row_start = y * diff_stride;
            for x in (0..row_bytes).step_by(4) {
                let actual_b = actual_data[actual_row_start + x];
                let actual_g = actual_data[actual_row_start + x + 1];
                let actual_r = actual_data[actual_row_start + x + 2];
                let expected_b = expected_data[expected_row_start + x];
                let expected_g = expected_data[expected_row_start + x + 1];
                let expected_r = expected_data[expected_row_start + x + 2];
                let diff_value = actual_b
                    .abs_diff(expected_b)
                    .max(actual_g.abs_diff(expected_g))
                    .max(actual_r.abs_diff(expected_r));

                diff_data[diff_row_start + x] = 0;
                diff_data[diff_row_start + x + 1] = 0;
                diff_data[diff_row_start + x + 2] = diff_value;
                diff_data[diff_row_start + x + 3] = u8::MAX;
            }
        }
    }

    let mut bytes = Vec::new();
    diff_surface
        .write_to_png(&mut bytes)
        .expect("encode diff png bytes");
    bytes
}

#[test]
#[ignore = "heavy visual differential gate; run explicitly via `cargo test-visual`"]
fn lightweight_visual_baseline_png_diff_stays_within_tolerance() {
    let corpus = load_visual_corpus();
    let artifact_dir = std::env::var_os("LIGHTWEIGHT_VISUAL_DIFF_ARTIFACT_DIR").map(PathBuf::from);
    if let Some(dir) = &artifact_dir {
        fs::create_dir_all(dir).expect("create visual diff artifact directory");
    }
    let mut artifact_fixtures = Vec::<VisualDiffArtifactFixture>::new();
    let mut failures = Vec::<String>::new();

    for fixture in &corpus.fixtures {
        let actual_png = render_fixture_png_bytes(fixture);
        let baseline_path = PathBuf::from(&fixture.baseline_png_relpath);
        let baseline_png = fs::read(&baseline_path).unwrap_or_else(|err| {
            panic!(
                "fixture `{}` missing baseline png `{}`: {err}",
                fixture.id,
                baseline_path.display()
            )
        });

        let mut actual_surface = decode_png_surface(&actual_png);
        let mut baseline_surface = decode_png_surface(&baseline_png);
        let (max_diff, mean_diff) = compare_surfaces(&mut actual_surface, &mut baseline_surface);
        let pass = max_diff <= fixture.tolerance.max_channel_abs_diff
            && mean_diff <= fixture.tolerance.mean_channel_abs_diff;
        if !pass {
            failures.push(format!(
                "fixture `{}` ({}) diff exceeded tolerance: max_diff={} (tol {}), mean_diff={} (tol {})",
                fixture.id,
                fixture.description,
                max_diff,
                fixture.tolerance.max_channel_abs_diff,
                mean_diff,
                fixture.tolerance.mean_channel_abs_diff
            ));
        }

        if let Some(dir) = &artifact_dir {
            let stem = sanitize_artifact_stem(&fixture.id);
            let actual_name = format!("{stem}.actual.png");
            let baseline_name = format!("{stem}.baseline.png");
            let diff_name = format!("{stem}.diff.png");
            let diff_png = build_diff_png(&mut actual_surface, &mut baseline_surface);
            fs::write(dir.join(&actual_name), &actual_png).expect("write actual visual artifact");
            fs::write(dir.join(&baseline_name), &baseline_png)
                .expect("write baseline visual artifact");
            fs::write(dir.join(&diff_name), diff_png).expect("write diff visual artifact");

            artifact_fixtures.push(VisualDiffArtifactFixture {
                id: fixture.id.clone(),
                description: fixture.description.clone(),
                max_channel_abs_diff: max_diff,
                mean_channel_abs_diff: mean_diff,
                tolerance_max_channel_abs_diff: fixture.tolerance.max_channel_abs_diff,
                tolerance_mean_channel_abs_diff: fixture.tolerance.mean_channel_abs_diff,
                pass,
                actual_png: actual_name,
                baseline_png: baseline_name,
                diff_png: diff_name,
            });
        }
    }

    if let Some(dir) = &artifact_dir {
        let summary = VisualDiffArtifactSummary {
            fixtures: artifact_fixtures,
        };
        let summary_json =
            serde_json::to_string_pretty(&summary).expect("serialize visual artifact summary");
        fs::write(dir.join("summary.json"), summary_json).expect("write visual artifact summary");
    }

    if !failures.is_empty() {
        panic!(
            "visual differential mismatches detected:\n{}",
            failures.join("\n")
        );
    }
}

#[test]
#[ignore = "manual baseline refresh utility"]
fn regenerate_lightweight_visual_baselines() {
    let corpus = load_visual_corpus();

    for fixture in &corpus.fixtures {
        let output_path = Path::new(&fixture.baseline_png_relpath);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).expect("create baseline directory");
        }

        let png = render_fixture_png_bytes(fixture);
        let mut file = File::create(output_path).expect("create baseline file");
        std::io::Write::write_all(&mut file, &png).expect("write baseline png");
    }
}
