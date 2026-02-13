use chart_rs::api::{
    ChartEngine, ChartEngineConfig, PriceAxisLabelConfig, PriceScaleRealtimeBehavior,
    PriceScaleTransformedBaseBehavior, TimeAxisLabelConfig, TimeScaleNavigationBehavior,
    TimeScaleScrollZoomBehavior,
};
use chart_rs::core::{DataPoint, OhlcBar, PriceScaleMode, Viewport};
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::NullRenderer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandKind {
    ExportTime,
    ImportTime,
    ExportPrice,
    ImportPrice,
    ExportInteraction,
    ImportInteraction,
    ImportLwcInteraction,
    ImportLwcVisual,
}

#[derive(Debug)]
struct CliArgs {
    command: CommandKind,
    input: PathBuf,
    output: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeTraceFile {
    trace_name: String,
    source: String,
    source_notes: String,
    viewport: DifferentialViewport,
    time_range: DifferentialTimeRange,
    price_range: DifferentialPriceRange,
    tolerance: f64,
    scenarios: Vec<TimeScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeCaptureFile {
    trace_name: String,
    source: String,
    source_notes: String,
    viewport: DifferentialViewport,
    time_range: DifferentialTimeRange,
    price_range: DifferentialPriceRange,
    tolerance: f64,
    scenarios: Vec<TimeCaptureScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DifferentialViewport {
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DifferentialTimeRange {
    start: f64,
    end: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DifferentialPriceRange {
    min: f64,
    max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeScenario {
    id: String,
    points: Vec<DifferentialPoint>,
    steps: Vec<TimeStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeCaptureScenario {
    id: String,
    points: Vec<DifferentialPoint>,
    steps: Vec<TimeCaptureStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DifferentialPoint {
    time: f64,
    value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeStep {
    action: TimeAction,
    #[serde(default)]
    expect: Option<TimeExpectation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeCaptureStep {
    action: TimeAction,
    observed: TimeExpectation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum TimeAction {
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TimeExpectation {
    #[serde(default)]
    visible_start: Option<f64>,
    #[serde(default)]
    visible_end: Option<f64>,
    #[serde(default)]
    visible_span: Option<f64>,
    #[serde(default)]
    right_margin_px: Option<f64>,
    #[serde(default)]
    scroll_position_bars: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PriceTraceFile {
    trace_name: String,
    source: String,
    source_notes: String,
    viewport: DifferentialViewport,
    time_range: DifferentialTimeRange,
    price_range: DifferentialPriceRange,
    tolerance: f64,
    scenarios: Vec<PriceScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PriceCaptureFile {
    trace_name: String,
    source: String,
    source_notes: String,
    viewport: DifferentialViewport,
    time_range: DifferentialTimeRange,
    price_range: DifferentialPriceRange,
    tolerance: f64,
    scenarios: Vec<PriceCaptureScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PriceScenario {
    id: String,
    points: Vec<DifferentialPoint>,
    #[serde(default)]
    probe_price: Option<f64>,
    #[serde(default)]
    probe_pixel: Option<f64>,
    steps: Vec<PriceStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PriceCaptureScenario {
    id: String,
    points: Vec<DifferentialPoint>,
    #[serde(default)]
    probe_price: Option<f64>,
    #[serde(default)]
    probe_pixel: Option<f64>,
    steps: Vec<PriceCaptureStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PriceStep {
    action: PriceAction,
    #[serde(default)]
    expect: Option<PriceExpectation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PriceCaptureStep {
    action: PriceAction,
    observed: PriceExpectation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum PriceAction {
    SetPriceScaleMode {
        mode: TracePriceScaleMode,
    },
    SetTransformedBaseBehavior {
        #[serde(default)]
        explicit_base_price: Option<f64>,
        dynamic_source: TraceTransformedBaseSource,
    },
    SetTimeVisibleRange {
        start: f64,
        end: f64,
    },
    AutoscaleVisibleData,
    SetPriceScaleRealtimeBehavior {
        autoscale_on_data_update: bool,
        autoscale_on_data_set: bool,
        autoscale_on_time_range_change: bool,
    },
    AppendPoint {
        time: f64,
        value: f64,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TraceTransformedBaseSource {
    DomainStart,
    FirstData,
    LastData,
    FirstVisibleData,
    LastVisibleData,
}

impl From<TraceTransformedBaseSource> for chart_rs::api::PriceScaleTransformedBaseSource {
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PriceExpectation {
    #[serde(default)]
    base_value: Option<f64>,
    #[serde(default)]
    price_domain_start: Option<f64>,
    #[serde(default)]
    price_domain_end: Option<f64>,
    #[serde(default)]
    price_domain_span: Option<f64>,
    #[serde(default)]
    probe_price_pixel: Option<f64>,
    #[serde(default)]
    probe_pixel_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InteractionTraceFile {
    trace_name: String,
    source: String,
    source_notes: String,
    viewport: DifferentialViewport,
    time_range: DifferentialTimeRange,
    price_range: DifferentialPriceRange,
    tolerance: f64,
    scenarios: Vec<InteractionScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InteractionCaptureFile {
    trace_name: String,
    source: String,
    source_notes: String,
    viewport: DifferentialViewport,
    time_range: DifferentialTimeRange,
    price_range: DifferentialPriceRange,
    tolerance: f64,
    scenarios: Vec<InteractionCaptureScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LightweightInteractionCaptureFile {
    #[serde(default)]
    trace_name: Option<String>,
    #[serde(default)]
    source_notes: Option<String>,
    viewport: DifferentialViewport,
    time_range: DifferentialTimeRange,
    price_range: DifferentialPriceRange,
    #[serde(default)]
    tolerance: Option<f64>,
    #[serde(default)]
    points: Vec<DifferentialPoint>,
    #[serde(default)]
    scenarios: Vec<LightweightInteractionCaptureScenario>,
    #[serde(default)]
    events: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LightweightInteractionCaptureScenario {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    points: Vec<DifferentialPoint>,
    #[serde(default)]
    events: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InteractionScenario {
    id: String,
    points: Vec<DifferentialPoint>,
    steps: Vec<InteractionStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InteractionCaptureScenario {
    id: String,
    points: Vec<DifferentialPoint>,
    steps: Vec<InteractionCaptureStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InteractionStep {
    action: InteractionAction,
    #[serde(default)]
    expect: Option<InteractionExpectation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InteractionCaptureStep {
    action: InteractionAction,
    observed: InteractionExpectation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum InteractionAction {
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct InteractionExpectation {
    #[serde(default)]
    visible_start: Option<f64>,
    #[serde(default)]
    visible_end: Option<f64>,
    #[serde(default)]
    visible_span: Option<f64>,
    #[serde(default)]
    scroll_position_bars: Option<f64>,
    #[serde(default)]
    kinetic_active: Option<bool>,
    #[serde(default)]
    kinetic_velocity_time_per_sec: Option<f64>,
    #[serde(default)]
    crosshair_visible: Option<bool>,
    #[serde(default)]
    crosshair_x: Option<f64>,
    #[serde(default)]
    crosshair_y: Option<f64>,
    #[serde(default)]
    crosshair_snapped_x: Option<f64>,
    #[serde(default)]
    crosshair_snapped_y: Option<f64>,
    #[serde(default)]
    crosshair_snapped_time: Option<f64>,
    #[serde(default)]
    crosshair_snapped_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VisualCorpusFile {
    schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    trace_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_notes: Option<String>,
    fixtures: Vec<VisualCorpusFixture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VisualCorpusFixture {
    id: String,
    description: String,
    input: VisualCorpusInput,
    baseline_png_relpath: String,
    tolerance: VisualCorpusTolerance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VisualCorpusInput {
    viewport: Viewport,
    time_range: [f64; 2],
    price_range: [f64; 2],
    #[serde(default)]
    points: Vec<DataPoint>,
    #[serde(default)]
    candles: Vec<OhlcBar>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    time_axis_label_config: Option<TimeAxisLabelConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    price_axis_label_config: Option<PriceAxisLabelConfig>,
    actions: Vec<VisualCorpusAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VisualCorpusTolerance {
    max_channel_abs_diff: u8,
    mean_channel_abs_diff: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum VisualCorpusAction {
    SetTimeVisibleRange {
        start: f64,
        end: f64,
    },
    SetPriceScaleMode {
        mode: TracePriceScaleMode,
    },
    SetTransformedBaseBehavior {
        #[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LightweightVisualCaptureFile {
    #[serde(default)]
    trace_name: Option<String>,
    #[serde(default)]
    source_notes: Option<String>,
    #[serde(default)]
    viewport: Option<DifferentialViewport>,
    #[serde(default)]
    time_range: Option<DifferentialTimeRange>,
    #[serde(default)]
    price_range: Option<DifferentialPriceRange>,
    #[serde(default)]
    tolerance: Option<VisualCorpusTolerance>,
    #[serde(default)]
    points: Vec<DataPoint>,
    #[serde(default)]
    candles: Vec<OhlcBar>,
    #[serde(default)]
    time_axis_label_config: Option<TimeAxisLabelConfig>,
    #[serde(default)]
    price_axis_label_config: Option<PriceAxisLabelConfig>,
    #[serde(default)]
    events: Vec<Value>,
    #[serde(default)]
    fixtures: Vec<LightweightVisualCaptureFixture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LightweightVisualCaptureFixture {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    viewport: Option<DifferentialViewport>,
    #[serde(default)]
    time_range: Option<DifferentialTimeRange>,
    #[serde(default)]
    price_range: Option<DifferentialPriceRange>,
    #[serde(default)]
    points: Vec<DataPoint>,
    #[serde(default)]
    candles: Vec<OhlcBar>,
    #[serde(default)]
    time_axis_label_config: Option<TimeAxisLabelConfig>,
    #[serde(default)]
    price_axis_label_config: Option<PriceAxisLabelConfig>,
    #[serde(default)]
    tolerance: Option<VisualCorpusTolerance>,
    #[serde(default)]
    baseline_png_relpath: Option<String>,
    #[serde(default)]
    events: Vec<Value>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    match args.command {
        CommandKind::ExportTime => {
            let raw = fs::read_to_string(&args.input)
                .map_err(|err| format!("failed to read `{}`: {err}", args.input.display()))?;
            let mut trace: TimeTraceFile =
                serde_json::from_str(&raw).map_err(|err| format!("invalid json: {err}"))?;
            export_time_trace(&mut trace)?;
            write_json(&args.output, &trace)
        }
        CommandKind::ImportTime => {
            let raw = fs::read_to_string(&args.input)
                .map_err(|err| format!("failed to read `{}`: {err}", args.input.display()))?;
            let capture: TimeCaptureFile =
                serde_json::from_str(&raw).map_err(|err| format!("invalid json: {err}"))?;
            let trace = import_time_capture(capture);
            write_json(&args.output, &trace)
        }
        CommandKind::ExportPrice => {
            let raw = fs::read_to_string(&args.input)
                .map_err(|err| format!("failed to read `{}`: {err}", args.input.display()))?;
            let mut trace: PriceTraceFile =
                serde_json::from_str(&raw).map_err(|err| format!("invalid json: {err}"))?;
            export_price_trace(&mut trace)?;
            write_json(&args.output, &trace)
        }
        CommandKind::ImportPrice => {
            let raw = fs::read_to_string(&args.input)
                .map_err(|err| format!("failed to read `{}`: {err}", args.input.display()))?;
            let capture: PriceCaptureFile =
                serde_json::from_str(&raw).map_err(|err| format!("invalid json: {err}"))?;
            let trace = import_price_capture(capture);
            write_json(&args.output, &trace)
        }
        CommandKind::ExportInteraction => {
            let raw = fs::read_to_string(&args.input)
                .map_err(|err| format!("failed to read `{}`: {err}", args.input.display()))?;
            let mut trace: InteractionTraceFile =
                serde_json::from_str(&raw).map_err(|err| format!("invalid json: {err}"))?;
            export_interaction_trace(&mut trace)?;
            write_json(&args.output, &trace)
        }
        CommandKind::ImportInteraction => {
            let raw = fs::read_to_string(&args.input)
                .map_err(|err| format!("failed to read `{}`: {err}", args.input.display()))?;
            let capture: InteractionCaptureFile =
                serde_json::from_str(&raw).map_err(|err| format!("invalid json: {err}"))?;
            let trace = import_interaction_capture(capture);
            write_json(&args.output, &trace)
        }
        CommandKind::ImportLwcInteraction => {
            let raw = fs::read_to_string(&args.input)
                .map_err(|err| format!("failed to read `{}`: {err}", args.input.display()))?;
            let capture: LightweightInteractionCaptureFile =
                serde_json::from_str(&raw).map_err(|err| format!("invalid json: {err}"))?;
            let trace = import_lightweight_interaction_capture(capture)?;
            write_json(&args.output, &trace)
        }
        CommandKind::ImportLwcVisual => {
            let raw = fs::read_to_string(&args.input)
                .map_err(|err| format!("failed to read `{}`: {err}", args.input.display()))?;
            let capture: LightweightVisualCaptureFile =
                serde_json::from_str(&raw).map_err(|err| format!("invalid json: {err}"))?;
            let corpus = import_lightweight_visual_capture(capture)?;
            write_json(&args.output, &corpus)
        }
    }
}

fn write_json<T: Serialize>(path: &PathBuf, value: &T) -> Result<(), String> {
    let payload = serde_json::to_string_pretty(value)
        .map_err(|err| format!("failed to serialize json: {err}"))?;
    fs::write(path, payload).map_err(|err| format!("failed to write `{}`: {err}", path.display()))
}

fn parse_args() -> Result<CliArgs, String> {
    let mut args = std::env::args().skip(1);
    let command = match args.next().as_deref() {
        Some("export-time") => CommandKind::ExportTime,
        Some("import-time") => CommandKind::ImportTime,
        Some("export-price") => CommandKind::ExportPrice,
        Some("import-price") => CommandKind::ImportPrice,
        Some("export-interaction") => CommandKind::ExportInteraction,
        Some("import-interaction") => CommandKind::ImportInteraction,
        Some("import-lwc-interaction") => CommandKind::ImportLwcInteraction,
        Some("import-lwc-visual") => CommandKind::ImportLwcVisual,
        _ => {
            return Err(
                "usage: differential_trace_tool <export-time|import-time|export-price|import-price|export-interaction|import-interaction|import-lwc-interaction|import-lwc-visual> --input <path> --output <path>"
                    .to_owned(),
            );
        }
    };

    let mut input = None::<PathBuf>;
    let mut output = None::<PathBuf>;

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--input" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --input".to_owned())?;
                input = Some(PathBuf::from(value));
            }
            "--output" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --output".to_owned())?;
                output = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                return Err(
                    "usage: differential_trace_tool <export-time|import-time|export-price|import-price|export-interaction|import-interaction|import-lwc-interaction|import-lwc-visual> --input <path> --output <path>".to_owned(),
                )
            }
            _ => return Err(format!("unknown argument `{flag}`")),
        }
    }

    let input = input.ok_or_else(|| "missing --input".to_owned())?;
    let output = output.ok_or_else(|| "missing --output".to_owned())?;
    Ok(CliArgs {
        command,
        input,
        output,
    })
}

fn build_engine(
    viewport: Viewport,
    time_start: f64,
    time_end: f64,
    price_min: f64,
    price_max: f64,
) -> ChartEngine<NullRenderer> {
    let config = ChartEngineConfig::new(viewport, time_start, time_end)
        .with_price_domain(price_min, price_max);
    ChartEngine::new(NullRenderer::default(), config).expect("engine init")
}

fn right_margin_px(engine: &ChartEngine<NullRenderer>) -> f64 {
    let width = f64::from(engine.viewport().width).max(1.0);
    let (visible_start, visible_end) = engine.time_visible_range();
    let (_, full_end) = engine.time_full_range();
    let span = (visible_end - visible_start).max(1e-12);
    ((visible_end - full_end) / span) * width
}

fn export_time_trace(trace: &mut TimeTraceFile) -> Result<(), String> {
    for scenario in &mut trace.scenarios {
        let mut engine = build_engine(
            Viewport::new(trace.viewport.width, trace.viewport.height),
            trace.time_range.start,
            trace.time_range.end,
            trace.price_range.min,
            trace.price_range.max,
        );

        if !scenario.points.is_empty() {
            let points = scenario
                .points
                .iter()
                .map(|point| DataPoint::new(point.time, point.value))
                .collect::<Vec<_>>();
            engine.set_data(points);
        }

        for step in &mut scenario.steps {
            match step.action {
                TimeAction::SetNavigation {
                    right_offset_bars,
                    bar_spacing_px,
                } => {
                    engine
                        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
                            right_offset_bars,
                            bar_spacing_px,
                        })
                        .map_err(|err| format!("set_navigation failed: {err}"))?;
                }
                TimeAction::SetRightOffsetPx { value } => {
                    engine
                        .set_time_scale_right_offset_px(Some(value))
                        .map_err(|err| format!("set_right_offset_px failed: {err}"))?;
                }
                TimeAction::SetScrollZoomBehavior {
                    right_bar_stays_on_scroll,
                } => {
                    engine
                        .set_time_scale_scroll_zoom_behavior(TimeScaleScrollZoomBehavior {
                            right_bar_stays_on_scroll,
                        })
                        .map_err(|err| format!("set_scroll_zoom_behavior failed: {err}"))?;
                }
                TimeAction::PanByPixels { delta_px } => {
                    engine
                        .pan_time_visible_by_pixels(delta_px)
                        .map_err(|err| format!("pan_by_pixels failed: {err}"))?;
                }
                TimeAction::WheelZoom {
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
                        .map_err(|err| format!("wheel_zoom failed: {err}"))?;
                }
            }

            let (visible_start, visible_end) = engine.time_visible_range();
            let expect = TimeExpectation {
                visible_start: Some(visible_start),
                visible_end: Some(visible_end),
                visible_span: Some(visible_end - visible_start),
                right_margin_px: Some(right_margin_px(&engine)),
                scroll_position_bars: engine.time_scroll_position_bars(),
            };
            step.expect = Some(expect);
        }
    }

    Ok(())
}

fn import_time_capture(capture: TimeCaptureFile) -> TimeTraceFile {
    TimeTraceFile {
        trace_name: capture.trace_name,
        source: capture.source,
        source_notes: capture.source_notes,
        viewport: capture.viewport,
        time_range: capture.time_range,
        price_range: capture.price_range,
        tolerance: capture.tolerance,
        scenarios: capture
            .scenarios
            .into_iter()
            .map(|scenario| TimeScenario {
                id: scenario.id,
                points: scenario.points,
                steps: scenario
                    .steps
                    .into_iter()
                    .map(|step| TimeStep {
                        action: step.action,
                        expect: Some(step.observed),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn export_price_trace(trace: &mut PriceTraceFile) -> Result<(), String> {
    for scenario in &mut trace.scenarios {
        let mut engine = build_engine(
            Viewport::new(trace.viewport.width, trace.viewport.height),
            trace.time_range.start,
            trace.time_range.end,
            trace.price_range.min,
            trace.price_range.max,
        );

        if !scenario.points.is_empty() {
            let points = scenario
                .points
                .iter()
                .map(|point| DataPoint::new(point.time, point.value))
                .collect::<Vec<_>>();
            engine.set_data(points);
        }

        for step in &mut scenario.steps {
            match step.action {
                PriceAction::SetPriceScaleMode { mode } => {
                    engine
                        .set_price_scale_mode(mode.into())
                        .map_err(|err| format!("set_price_scale_mode failed: {err}"))?;
                }
                PriceAction::SetTransformedBaseBehavior {
                    explicit_base_price,
                    dynamic_source,
                } => {
                    engine
                        .set_price_scale_transformed_base_behavior(
                            PriceScaleTransformedBaseBehavior {
                                explicit_base_price,
                                dynamic_source: dynamic_source.into(),
                            },
                        )
                        .map_err(|err| format!("set_transformed_base_behavior failed: {err}"))?;
                }
                PriceAction::SetTimeVisibleRange { start, end } => {
                    engine
                        .set_time_visible_range(start, end)
                        .map_err(|err| format!("set_time_visible_range failed: {err}"))?;
                }
                PriceAction::AutoscaleVisibleData => {
                    engine
                        .autoscale_price_from_visible_data()
                        .map_err(|err| format!("autoscale_visible_data failed: {err}"))?;
                }
                PriceAction::SetPriceScaleRealtimeBehavior {
                    autoscale_on_data_update,
                    autoscale_on_data_set,
                    autoscale_on_time_range_change,
                } => {
                    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
                        autoscale_on_data_update,
                        autoscale_on_data_set,
                        autoscale_on_time_range_change,
                    });
                }
                PriceAction::AppendPoint { time, value } => {
                    engine.append_point(DataPoint::new(time, value));
                }
            }

            let (domain_start, domain_end) = engine.price_domain();
            let probe_price_pixel = match scenario.probe_price {
                Some(price) => Some(
                    engine
                        .map_price_to_pixel(price)
                        .map_err(|err| format!("probe price->pixel failed: {err}"))?,
                ),
                None => None,
            };
            let probe_pixel_price = match scenario.probe_pixel {
                Some(pixel) => Some(
                    engine
                        .map_pixel_to_price(pixel)
                        .map_err(|err| format!("probe pixel->price failed: {err}"))?,
                ),
                None => None,
            };

            step.expect = Some(PriceExpectation {
                base_value: engine.price_scale_transformed_base_value(),
                price_domain_start: Some(domain_start),
                price_domain_end: Some(domain_end),
                price_domain_span: Some(domain_end - domain_start),
                probe_price_pixel,
                probe_pixel_price,
            });
        }
    }

    Ok(())
}

fn import_price_capture(capture: PriceCaptureFile) -> PriceTraceFile {
    PriceTraceFile {
        trace_name: capture.trace_name,
        source: capture.source,
        source_notes: capture.source_notes,
        viewport: capture.viewport,
        time_range: capture.time_range,
        price_range: capture.price_range,
        tolerance: capture.tolerance,
        scenarios: capture
            .scenarios
            .into_iter()
            .map(|scenario| PriceScenario {
                id: scenario.id,
                points: scenario.points,
                probe_price: scenario.probe_price,
                probe_pixel: scenario.probe_pixel,
                steps: scenario
                    .steps
                    .into_iter()
                    .map(|step| PriceStep {
                        action: step.action,
                        expect: Some(step.observed),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn export_interaction_trace(trace: &mut InteractionTraceFile) -> Result<(), String> {
    for scenario in &mut trace.scenarios {
        let mut engine = build_engine(
            Viewport::new(trace.viewport.width, trace.viewport.height),
            trace.time_range.start,
            trace.time_range.end,
            trace.price_range.min,
            trace.price_range.max,
        );

        if !scenario.points.is_empty() {
            let points = scenario
                .points
                .iter()
                .map(|point| DataPoint::new(point.time, point.value))
                .collect::<Vec<_>>();
            engine.set_data(points);
        }

        for step in &mut scenario.steps {
            match step.action {
                InteractionAction::SetNavigation {
                    right_offset_bars,
                    bar_spacing_px,
                } => {
                    engine
                        .set_time_scale_navigation_behavior(TimeScaleNavigationBehavior {
                            right_offset_bars,
                            bar_spacing_px,
                        })
                        .map_err(|err| format!("set_navigation failed: {err}"))?;
                }
                InteractionAction::WheelPan {
                    wheel_delta_x,
                    pan_step_ratio,
                } => {
                    let _ = engine
                        .wheel_pan_time_visible(wheel_delta_x, pan_step_ratio)
                        .map_err(|err| format!("wheel_pan failed: {err}"))?;
                }
                InteractionAction::TouchDragPan {
                    delta_x_px,
                    delta_y_px,
                } => {
                    let _ = engine
                        .touch_drag_pan_time_visible(delta_x_px, delta_y_px)
                        .map_err(|err| format!("touch_drag_pan failed: {err}"))?;
                }
                InteractionAction::WheelZoom {
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
                        .map_err(|err| format!("wheel_zoom failed: {err}"))?;
                }
                InteractionAction::PinchZoom {
                    factor,
                    anchor_px,
                    min_span_absolute,
                } => {
                    let _ = engine
                        .pinch_zoom_time_visible(factor, anchor_px, min_span_absolute)
                        .map_err(|err| format!("pinch_zoom failed: {err}"))?;
                }
                InteractionAction::StartKineticPan {
                    velocity_time_per_sec,
                } => {
                    engine
                        .start_kinetic_pan(velocity_time_per_sec)
                        .map_err(|err| format!("start_kinetic_pan failed: {err}"))?;
                }
                InteractionAction::StepKineticPan { delta_seconds } => {
                    let _ = engine
                        .step_kinetic_pan(delta_seconds)
                        .map_err(|err| format!("step_kinetic_pan failed: {err}"))?;
                }
                InteractionAction::SetCrosshairMode { mode } => {
                    engine.set_crosshair_mode(mode.into());
                }
                InteractionAction::PointerMove { x, y } => {
                    engine.pointer_move(x, y);
                }
                InteractionAction::PointerLeave => {
                    engine.pointer_leave();
                }
            }

            let (visible_start, visible_end) = engine.time_visible_range();
            let kinetic_state = engine.kinetic_pan_state();
            let crosshair = engine.crosshair_state();

            step.expect = Some(InteractionExpectation {
                visible_start: Some(visible_start),
                visible_end: Some(visible_end),
                visible_span: Some(visible_end - visible_start),
                scroll_position_bars: engine.time_scroll_position_bars(),
                kinetic_active: Some(kinetic_state.active),
                kinetic_velocity_time_per_sec: Some(kinetic_state.velocity_time_per_sec),
                crosshair_visible: Some(crosshair.visible),
                crosshair_x: Some(crosshair.x),
                crosshair_y: Some(crosshair.y),
                crosshair_snapped_x: crosshair.snapped_x,
                crosshair_snapped_y: crosshair.snapped_y,
                crosshair_snapped_time: crosshair.snapped_time,
                crosshair_snapped_price: crosshair.snapped_price,
            });
        }
    }

    Ok(())
}

fn import_interaction_capture(capture: InteractionCaptureFile) -> InteractionTraceFile {
    InteractionTraceFile {
        trace_name: capture.trace_name,
        source: capture.source,
        source_notes: capture.source_notes,
        viewport: capture.viewport,
        time_range: capture.time_range,
        price_range: capture.price_range,
        tolerance: capture.tolerance,
        scenarios: capture
            .scenarios
            .into_iter()
            .map(|scenario| InteractionScenario {
                id: scenario.id,
                points: scenario.points,
                steps: scenario
                    .steps
                    .into_iter()
                    .map(|step| InteractionStep {
                        action: step.action,
                        expect: Some(step.observed),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn import_lightweight_interaction_capture(
    capture: LightweightInteractionCaptureFile,
) -> Result<InteractionTraceFile, String> {
    let scenarios = if !capture.scenarios.is_empty() {
        capture
            .scenarios
            .into_iter()
            .enumerate()
            .map(|(index, scenario)| {
                let id = scenario
                    .id
                    .unwrap_or_else(|| format!("lightweight-scenario-{}", index + 1));
                import_lightweight_interaction_scenario(id, scenario.points, scenario.events)
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        vec![import_lightweight_interaction_scenario(
            "lightweight-scenario-1".to_owned(),
            capture.points,
            capture.events,
        )?]
    };

    Ok(InteractionTraceFile {
        trace_name: capture
            .trace_name
            .unwrap_or_else(|| "lightweight-real-capture-interaction".to_owned()),
        source: "lightweight-charts-v5.1-capture".to_owned(),
        source_notes: capture.source_notes.unwrap_or_else(|| {
            "Imported directly from Lightweight real capture without manual normalization."
                .to_owned()
        }),
        viewport: capture.viewport,
        time_range: capture.time_range,
        price_range: capture.price_range,
        tolerance: capture.tolerance.unwrap_or(1e-6),
        scenarios,
    })
}

fn import_lightweight_interaction_scenario(
    id: String,
    points: Vec<DifferentialPoint>,
    events: Vec<Value>,
) -> Result<InteractionScenario, String> {
    let mut steps = Vec::with_capacity(events.len());
    for event in events {
        let action = map_lightweight_event_to_action(&event)?;
        let expect = parse_optional_interaction_expectation(&event)?;
        steps.push(InteractionStep { action, expect });
    }
    Ok(InteractionScenario { id, points, steps })
}

fn map_lightweight_event_to_action(event: &Value) -> Result<InteractionAction, String> {
    let kind = event_kind(event)
        .ok_or_else(|| "lightweight capture event missing `type`/`event`/`kind`".to_owned())?;

    match kind.as_str() {
        "set_navigation" => Ok(InteractionAction::SetNavigation {
            right_offset_bars: event_f64(event, &["right_offset_bars", "rightOffsetBars"])
                .unwrap_or(0.0),
            bar_spacing_px: event_optional_f64(event, &["bar_spacing_px", "barSpacingPx"]),
        }),
        "wheel_pan" => Ok(InteractionAction::WheelPan {
            wheel_delta_x: event_f64(event, &["wheel_delta_x", "wheelDeltaX", "deltaX"])
                .ok_or_else(|| "wheel_pan event missing delta_x".to_owned())?,
            pan_step_ratio: event_f64(event, &["pan_step_ratio", "panStepRatio"]).unwrap_or(0.1),
        }),
        "touch_drag_pan" | "touch_pan" | "touch_move" => Ok(InteractionAction::TouchDragPan {
            delta_x_px: event_f64(event, &["delta_x_px", "deltaXPx", "deltaX"]).unwrap_or(0.0),
            delta_y_px: event_f64(event, &["delta_y_px", "deltaYPx", "deltaY"]).unwrap_or(0.0),
        }),
        "wheel_zoom" => Ok(InteractionAction::WheelZoom {
            wheel_delta_y: event_f64(event, &["wheel_delta_y", "wheelDeltaY", "deltaY"])
                .ok_or_else(|| "wheel_zoom event missing delta_y".to_owned())?,
            anchor_px: event_f64(event, &["anchor_px", "anchorX", "anchorPx"]).unwrap_or(0.0),
            zoom_step_ratio: event_f64(event, &["zoom_step_ratio", "zoomStepRatio"]).unwrap_or(0.2),
            min_span_absolute: event_f64(event, &["min_span_absolute", "minSpanAbsolute"])
                .unwrap_or(1e-6),
        }),
        "wheel" => {
            let delta_y =
                event_f64(event, &["wheel_delta_y", "wheelDeltaY", "deltaY"]).unwrap_or(0.0);
            if delta_y != 0.0 {
                Ok(InteractionAction::WheelZoom {
                    wheel_delta_y: delta_y,
                    anchor_px: event_f64(event, &["anchor_px", "anchorX", "anchorPx"])
                        .unwrap_or(0.0),
                    zoom_step_ratio: event_f64(event, &["zoom_step_ratio", "zoomStepRatio"])
                        .unwrap_or(0.2),
                    min_span_absolute: event_f64(event, &["min_span_absolute", "minSpanAbsolute"])
                        .unwrap_or(1e-6),
                })
            } else {
                Ok(InteractionAction::WheelPan {
                    wheel_delta_x: event_f64(event, &["wheel_delta_x", "wheelDeltaX", "deltaX"])
                        .ok_or_else(|| "wheel event missing delta_x".to_owned())?,
                    pan_step_ratio: event_f64(event, &["pan_step_ratio", "panStepRatio"])
                        .unwrap_or(0.1),
                })
            }
        }
        "pinch_zoom" | "pinch" => Ok(InteractionAction::PinchZoom {
            factor: event_f64(event, &["factor", "scale"])
                .ok_or_else(|| "pinch_zoom event missing factor/scale".to_owned())?,
            anchor_px: event_f64(event, &["anchor_px", "anchorX", "anchorPx"]).unwrap_or(0.0),
            min_span_absolute: event_f64(event, &["min_span_absolute", "minSpanAbsolute"])
                .unwrap_or(1e-6),
        }),
        "start_kinetic_pan" | "kinetic_start" => Ok(InteractionAction::StartKineticPan {
            velocity_time_per_sec: event_f64(
                event,
                &["velocity_time_per_sec", "velocityTimePerSec", "velocity"],
            )
            .ok_or_else(|| "kinetic_start event missing velocity".to_owned())?,
        }),
        "step_kinetic_pan" | "kinetic_step" => Ok(InteractionAction::StepKineticPan {
            delta_seconds: event_f64(event, &["delta_seconds", "deltaSeconds", "dt"])
                .unwrap_or(1.0),
        }),
        "set_crosshair_mode" | "crosshair_mode" => {
            let mode_raw = event_string(event, &["mode"])
                .ok_or_else(|| "crosshair_mode event missing mode".to_owned())?;
            let mode = match mode_raw.to_ascii_lowercase().as_str() {
                "magnet" => TraceCrosshairMode::Magnet,
                "normal" => TraceCrosshairMode::Normal,
                "hidden" => TraceCrosshairMode::Hidden,
                other => {
                    return Err(format!(
                        "unsupported crosshair mode `{other}` in lightweight event"
                    ));
                }
            };
            Ok(InteractionAction::SetCrosshairMode { mode })
        }
        "pointer_move" | "crosshair_move" => Ok(InteractionAction::PointerMove {
            x: event_f64(event, &["x", "clientX", "crosshairX"])
                .ok_or_else(|| "pointer_move event missing x".to_owned())?,
            y: event_f64(event, &["y", "clientY", "crosshairY"])
                .ok_or_else(|| "pointer_move event missing y".to_owned())?,
        }),
        "pointer_leave" | "crosshair_leave" => Ok(InteractionAction::PointerLeave),
        other => Err(format!("unsupported lightweight event type `{other}`")),
    }
}

fn parse_optional_interaction_expectation(
    event: &Value,
) -> Result<Option<InteractionExpectation>, String> {
    let observed = event
        .get("observed")
        .or_else(|| event.get("expect"))
        .cloned();
    match observed {
        Some(value) => serde_json::from_value::<InteractionExpectation>(value)
            .map(Some)
            .map_err(|err| format!("invalid observed/expect payload: {err}")),
        None => Ok(None),
    }
}

fn import_lightweight_visual_capture(
    capture: LightweightVisualCaptureFile,
) -> Result<VisualCorpusFile, String> {
    let LightweightVisualCaptureFile {
        trace_name,
        source_notes,
        viewport,
        time_range,
        price_range,
        tolerance,
        points,
        candles,
        time_axis_label_config,
        price_axis_label_config,
        events,
        fixtures: input_fixtures,
    } = capture;

    let fixture_count = if input_fixtures.is_empty() {
        1
    } else {
        input_fixtures.len()
    };
    let mut fixtures = Vec::with_capacity(fixture_count);

    if input_fixtures.is_empty() {
        let fixture = LightweightVisualCaptureFixture {
            id: None,
            description: None,
            viewport,
            time_range,
            price_range,
            points,
            candles,
            time_axis_label_config,
            price_axis_label_config,
            tolerance,
            baseline_png_relpath: None,
            events,
        };
        fixtures.push(import_lightweight_visual_fixture(
            fixture, 0, None, None, None, None,
        )?);
    } else {
        for (index, fixture) in input_fixtures.into_iter().enumerate() {
            fixtures.push(import_lightweight_visual_fixture(
                fixture,
                index,
                viewport.clone(),
                time_range.clone(),
                price_range.clone(),
                tolerance.clone(),
            )?);
        }
    }

    Ok(VisualCorpusFile {
        schema_version: 1,
        trace_name,
        source: Some("lightweight-charts-v5.1-capture".to_owned()),
        source_notes: Some(source_notes.unwrap_or_else(|| {
            "Imported directly from Lightweight real visual capture without manual normalization."
                .to_owned()
        })),
        fixtures,
    })
}

fn import_lightweight_visual_fixture(
    fixture: LightweightVisualCaptureFixture,
    fixture_index: usize,
    default_viewport: Option<DifferentialViewport>,
    default_time_range: Option<DifferentialTimeRange>,
    default_price_range: Option<DifferentialPriceRange>,
    default_tolerance: Option<VisualCorpusTolerance>,
) -> Result<VisualCorpusFixture, String> {
    let id = fixture
        .id
        .unwrap_or_else(|| format!("lwc-visual-fixture-{}", fixture_index + 1));
    let description = fixture
        .description
        .unwrap_or_else(|| "Imported directly from Lightweight visual capture".to_owned());

    let viewport = fixture
        .viewport
        .or(default_viewport)
        .map(|viewport| Viewport::new(viewport.width, viewport.height))
        .ok_or_else(|| format!("visual fixture `{id}` missing viewport"))?;
    let time_range = fixture
        .time_range
        .or(default_time_range)
        .ok_or_else(|| format!("visual fixture `{id}` missing time_range"))?;
    let price_range = fixture
        .price_range
        .or(default_price_range)
        .ok_or_else(|| format!("visual fixture `{id}` missing price_range"))?;

    let mut actions = Vec::with_capacity(fixture.events.len());
    for event in fixture.events {
        actions.push(map_lightweight_event_to_visual_action(&event)?);
    }

    let tolerance = fixture
        .tolerance
        .or(default_tolerance)
        .unwrap_or(VisualCorpusTolerance {
            max_channel_abs_diff: 0,
            mean_channel_abs_diff: 0.0,
        });
    let baseline_png_relpath = fixture.baseline_png_relpath.unwrap_or_else(|| {
        format!(
            "tests/fixtures/lightweight_visual_differential/reference_png/{}.png",
            sanitize_fixture_id_for_relpath(&id)
        )
    });

    Ok(VisualCorpusFixture {
        id,
        description,
        input: VisualCorpusInput {
            viewport,
            time_range: [time_range.start, time_range.end],
            price_range: [price_range.min, price_range.max],
            points: fixture.points,
            candles: fixture.candles,
            time_axis_label_config: fixture.time_axis_label_config,
            price_axis_label_config: fixture.price_axis_label_config,
            actions,
        },
        baseline_png_relpath,
        tolerance,
    })
}

fn sanitize_fixture_id_for_relpath(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('-');
        }
    }
    let out = out.trim_matches('-');
    if out.is_empty() {
        "lwc-visual-fixture".to_owned()
    } else {
        out.to_owned()
    }
}

fn map_lightweight_event_to_visual_action(event: &Value) -> Result<VisualCorpusAction, String> {
    let kind = event_kind(event).ok_or_else(|| {
        "lightweight visual capture event missing `type`/`event`/`kind`".to_owned()
    })?;

    match kind.as_str() {
        "set_time_visible_range" | "time_visible_range" | "set_visible_range_time" => {
            Ok(VisualCorpusAction::SetTimeVisibleRange {
                start: event_f64(event, &["start", "from", "visibleStart"])
                    .ok_or_else(|| "set_time_visible_range event missing start".to_owned())?,
                end: event_f64(event, &["end", "to", "visibleEnd"])
                    .ok_or_else(|| "set_time_visible_range event missing end".to_owned())?,
            })
        }
        "set_price_scale_mode" | "price_scale_mode" => Ok(VisualCorpusAction::SetPriceScaleMode {
            mode: parse_trace_price_scale_mode(event)?,
        }),
        "set_transformed_base_behavior" | "transformed_base_behavior" => {
            Ok(VisualCorpusAction::SetTransformedBaseBehavior {
                explicit_base_price: event_optional_f64(
                    event,
                    &[
                        "explicit_base_price",
                        "explicitBasePrice",
                        "base_price",
                        "basePrice",
                    ],
                ),
                dynamic_source: parse_trace_transformed_base_source(event)?,
            })
        }
        "autoscale_visible_data" | "autoscale_visible" => {
            Ok(VisualCorpusAction::AutoscaleVisibleData)
        }
        "set_crosshair_mode" | "crosshair_mode" => Ok(VisualCorpusAction::SetCrosshairMode {
            mode: parse_trace_crosshair_mode(event)?,
        }),
        "pointer_move" | "crosshair_move" => Ok(VisualCorpusAction::PointerMove {
            x: event_f64(event, &["x", "clientX", "crosshairX"])
                .ok_or_else(|| "pointer_move event missing x".to_owned())?,
            y: event_f64(event, &["y", "clientY", "crosshairY"])
                .ok_or_else(|| "pointer_move event missing y".to_owned())?,
        }),
        "axis_drag_scale_price" | "axis_scale_price" => {
            Ok(VisualCorpusAction::AxisDragScalePrice {
                drag_delta_y_px: event_f64(event, &["drag_delta_y_px", "dragDeltaYPx", "deltaY"])
                    .ok_or_else(|| {
                    "axis_drag_scale_price event missing deltaY".to_owned()
                })?,
                anchor_y_px: event_f64(event, &["anchor_y_px", "anchorYPx", "anchorY"])
                    .ok_or_else(|| "axis_drag_scale_price event missing anchorY".to_owned())?,
                zoom_step_ratio: event_f64(event, &["zoom_step_ratio", "zoomStepRatio"])
                    .unwrap_or(0.2),
                min_span_absolute: event_f64(event, &["min_span_absolute", "minSpanAbsolute"])
                    .unwrap_or(1e-6),
            })
        }
        "axis_drag_scale_time" | "axis_scale_time" => Ok(VisualCorpusAction::AxisDragScaleTime {
            drag_delta_x_px: event_f64(event, &["drag_delta_x_px", "dragDeltaXPx", "deltaX"])
                .ok_or_else(|| "axis_drag_scale_time event missing deltaX".to_owned())?,
            anchor_x_px: event_f64(event, &["anchor_x_px", "anchorXPx", "anchorX"])
                .ok_or_else(|| "axis_drag_scale_time event missing anchorX".to_owned())?,
            zoom_step_ratio: event_f64(event, &["zoom_step_ratio", "zoomStepRatio"]).unwrap_or(0.2),
            min_span_absolute: event_f64(event, &["min_span_absolute", "minSpanAbsolute"])
                .unwrap_or(1e-6),
        }),
        other => Err(format!(
            "unsupported lightweight visual event type `{other}`"
        )),
    }
}

fn parse_trace_price_scale_mode(event: &Value) -> Result<TracePriceScaleMode, String> {
    let raw = event_string(event, &["mode", "priceScaleMode"])
        .ok_or_else(|| "price_scale_mode event missing mode".to_owned())?;
    match raw.to_ascii_lowercase().as_str() {
        "linear" | "normal" => Ok(TracePriceScaleMode::Linear),
        "log" | "logarithmic" => Ok(TracePriceScaleMode::Log),
        "percentage" | "percent" => Ok(TracePriceScaleMode::Percentage),
        "indexed_to_100" | "indexedto100" | "indexed" => Ok(TracePriceScaleMode::IndexedTo100),
        other => Err(format!("unsupported price scale mode `{other}`")),
    }
}

fn parse_trace_transformed_base_source(
    event: &Value,
) -> Result<TraceTransformedBaseSource, String> {
    let raw = event_string(event, &["dynamic_source", "dynamicSource", "baseSource"])
        .unwrap_or_else(|| "last_visible_data".to_owned());
    match raw.to_ascii_lowercase().as_str() {
        "domain_start" | "domainstart" => Ok(TraceTransformedBaseSource::DomainStart),
        "first_data" | "firstdata" => Ok(TraceTransformedBaseSource::FirstData),
        "last_data" | "lastdata" => Ok(TraceTransformedBaseSource::LastData),
        "first_visible_data" | "firstvisibledata" => {
            Ok(TraceTransformedBaseSource::FirstVisibleData)
        }
        "last_visible_data" | "lastvisibledata" => Ok(TraceTransformedBaseSource::LastVisibleData),
        other => Err(format!("unsupported transformed base source `{other}`")),
    }
}

fn parse_trace_crosshair_mode(event: &Value) -> Result<TraceCrosshairMode, String> {
    let raw = event_string(event, &["mode"])
        .ok_or_else(|| "crosshair_mode event missing mode".to_owned())?;
    match raw.to_ascii_lowercase().as_str() {
        "magnet" => Ok(TraceCrosshairMode::Magnet),
        "normal" => Ok(TraceCrosshairMode::Normal),
        "hidden" => Ok(TraceCrosshairMode::Hidden),
        other => Err(format!("unsupported crosshair mode `{other}`")),
    }
}

fn event_kind(event: &Value) -> Option<String> {
    event
        .get("type")
        .or_else(|| event.get("event"))
        .or_else(|| event.get("kind"))
        .and_then(Value::as_str)
        .map(|raw| raw.trim().to_ascii_lowercase())
}

fn event_f64(event: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| event.get(*key))
        .and_then(|value| value.as_f64())
}

fn event_optional_f64(event: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|key| match event.get(*key) {
        Some(Value::Null) => None,
        Some(value) => value.as_f64(),
        None => None,
    })
}

fn event_string(event: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| event.get(*key))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_time_capture_maps_observed_to_expect() {
        let capture = TimeCaptureFile {
            trace_name: "t".to_owned(),
            source: "lightweight".to_owned(),
            source_notes: "note".to_owned(),
            viewport: DifferentialViewport {
                width: 1000,
                height: 500,
            },
            time_range: DifferentialTimeRange {
                start: 0.0,
                end: 100.0,
            },
            price_range: DifferentialPriceRange { min: 0.0, max: 1.0 },
            tolerance: 1e-6,
            scenarios: vec![TimeCaptureScenario {
                id: "s1".to_owned(),
                points: vec![],
                steps: vec![TimeCaptureStep {
                    action: TimeAction::SetRightOffsetPx { value: 100.0 },
                    observed: TimeExpectation {
                        visible_start: Some(10.0),
                        visible_end: Some(110.0),
                        visible_span: Some(100.0),
                        right_margin_px: Some(100.0),
                        scroll_position_bars: Some(1.0),
                    },
                }],
            }],
        };

        let imported = import_time_capture(capture);
        assert_eq!(imported.scenarios.len(), 1);
        assert_eq!(imported.scenarios[0].steps.len(), 1);
        assert!(imported.scenarios[0].steps[0].expect.is_some());
    }

    #[test]
    fn export_time_trace_populates_expectation_fields() {
        let mut trace = TimeTraceFile {
            trace_name: "t".to_owned(),
            source: "lightweight".to_owned(),
            source_notes: "note".to_owned(),
            viewport: DifferentialViewport {
                width: 1000,
                height: 500,
            },
            time_range: DifferentialTimeRange {
                start: 0.0,
                end: 100.0,
            },
            price_range: DifferentialPriceRange { min: 0.0, max: 1.0 },
            tolerance: 1e-6,
            scenarios: vec![TimeScenario {
                id: "s1".to_owned(),
                points: vec![
                    DifferentialPoint {
                        time: 0.0,
                        value: 0.5,
                    },
                    DifferentialPoint {
                        time: 10.0,
                        value: 0.6,
                    },
                ],
                steps: vec![TimeStep {
                    action: TimeAction::SetRightOffsetPx { value: 120.0 },
                    expect: None,
                }],
            }],
        };

        export_time_trace(&mut trace).expect("export time trace");
        let expect = trace.scenarios[0].steps[0]
            .expect
            .as_ref()
            .expect("expectation generated");
        assert!(expect.visible_start.is_some());
        assert!(expect.visible_end.is_some());
        assert!(expect.visible_span.is_some());
        assert!(expect.right_margin_px.is_some());
    }

    #[test]
    fn import_interaction_capture_maps_observed_to_expect() {
        let capture = InteractionCaptureFile {
            trace_name: "interaction".to_owned(),
            source: "lightweight".to_owned(),
            source_notes: "note".to_owned(),
            viewport: DifferentialViewport {
                width: 800,
                height: 400,
            },
            time_range: DifferentialTimeRange {
                start: 0.0,
                end: 100.0,
            },
            price_range: DifferentialPriceRange {
                min: 0.0,
                max: 100.0,
            },
            tolerance: 1e-6,
            scenarios: vec![InteractionCaptureScenario {
                id: "s1".to_owned(),
                points: vec![],
                steps: vec![InteractionCaptureStep {
                    action: InteractionAction::PointerLeave,
                    observed: InteractionExpectation {
                        crosshair_visible: Some(false),
                        ..InteractionExpectation::default()
                    },
                }],
            }],
        };

        let imported = import_interaction_capture(capture);
        assert_eq!(imported.scenarios.len(), 1);
        assert_eq!(imported.scenarios[0].steps.len(), 1);
        assert!(imported.scenarios[0].steps[0].expect.is_some());
    }

    #[test]
    fn export_interaction_trace_populates_expectation_fields() {
        let mut trace = InteractionTraceFile {
            trace_name: "interaction".to_owned(),
            source: "lightweight".to_owned(),
            source_notes: "note".to_owned(),
            viewport: DifferentialViewport {
                width: 800,
                height: 400,
            },
            time_range: DifferentialTimeRange {
                start: 0.0,
                end: 100.0,
            },
            price_range: DifferentialPriceRange {
                min: 0.0,
                max: 100.0,
            },
            tolerance: 1e-6,
            scenarios: vec![InteractionScenario {
                id: "s1".to_owned(),
                points: vec![
                    DifferentialPoint {
                        time: 0.0,
                        value: 10.0,
                    },
                    DifferentialPoint {
                        time: 10.0,
                        value: 12.0,
                    },
                ],
                steps: vec![InteractionStep {
                    action: InteractionAction::PointerMove { x: 100.0, y: 120.0 },
                    expect: None,
                }],
            }],
        };

        export_interaction_trace(&mut trace).expect("export interaction trace");
        let expect = trace.scenarios[0].steps[0]
            .expect
            .as_ref()
            .expect("expectation generated");
        assert!(expect.visible_start.is_some());
        assert!(expect.visible_end.is_some());
        assert!(expect.visible_span.is_some());
        assert!(expect.crosshair_visible.is_some());
        assert!(expect.crosshair_x.is_some());
        assert!(expect.crosshair_y.is_some());
    }

    #[test]
    fn import_lwc_interaction_capture_maps_wheel_touch_crosshair_events() {
        let capture = LightweightInteractionCaptureFile {
            trace_name: Some("raw-capture".to_owned()),
            source_notes: Some("captured from lwc".to_owned()),
            viewport: DifferentialViewport {
                width: 1000,
                height: 500,
            },
            time_range: DifferentialTimeRange {
                start: 0.0,
                end: 100.0,
            },
            price_range: DifferentialPriceRange {
                min: 0.0,
                max: 100.0,
            },
            tolerance: Some(1e-6),
            points: vec![
                DifferentialPoint {
                    time: 0.0,
                    value: 10.0,
                },
                DifferentialPoint {
                    time: 10.0,
                    value: 20.0,
                },
            ],
            scenarios: vec![],
            events: vec![
                serde_json::json!({
                    "type": "wheel",
                    "deltaY": -120.0,
                    "anchorX": 300.0,
                    "zoomStepRatio": 0.2,
                    "minSpanAbsolute": 1e-6,
                    "observed": { "visible_span": 80.0 }
                }),
                serde_json::json!({
                    "type": "touch_move",
                    "deltaX": 40.0,
                    "deltaY": 5.0
                }),
                serde_json::json!({
                    "event": "crosshair_mode",
                    "mode": "magnet"
                }),
                serde_json::json!({
                    "kind": "crosshair_move",
                    "x": 250.0,
                    "y": 120.0
                }),
            ],
        };

        let trace =
            import_lightweight_interaction_capture(capture).expect("import lightweight capture");
        assert_eq!(trace.scenarios.len(), 1);
        assert_eq!(trace.scenarios[0].steps.len(), 4);

        assert!(matches!(
            trace.scenarios[0].steps[0].action,
            InteractionAction::WheelZoom { .. }
        ));
        assert!(matches!(
            trace.scenarios[0].steps[1].action,
            InteractionAction::TouchDragPan { .. }
        ));
        assert!(matches!(
            trace.scenarios[0].steps[2].action,
            InteractionAction::SetCrosshairMode { .. }
        ));
        assert!(matches!(
            trace.scenarios[0].steps[3].action,
            InteractionAction::PointerMove { .. }
        ));

        let observed = trace.scenarios[0].steps[0]
            .expect
            .as_ref()
            .expect("observed payload should map to expect");
        assert_eq!(observed.visible_span, Some(80.0));
    }

    #[test]
    fn import_lwc_visual_capture_maps_events_into_visual_corpus() {
        let capture = LightweightVisualCaptureFile {
            trace_name: Some("visual-raw".to_owned()),
            source_notes: None,
            viewport: None,
            time_range: None,
            price_range: None,
            tolerance: Some(VisualCorpusTolerance {
                max_channel_abs_diff: 2,
                mean_channel_abs_diff: 0.5,
            }),
            points: vec![],
            candles: vec![],
            time_axis_label_config: None,
            price_axis_label_config: None,
            events: vec![],
            fixtures: vec![LightweightVisualCaptureFixture {
                id: Some("LWC Fixture 01".to_owned()),
                description: Some("visual fixture".to_owned()),
                viewport: Some(DifferentialViewport {
                    width: 800,
                    height: 400,
                }),
                time_range: Some(DifferentialTimeRange {
                    start: 0.0,
                    end: 100.0,
                }),
                price_range: Some(DifferentialPriceRange {
                    min: 10.0,
                    max: 90.0,
                }),
                points: vec![DataPoint::new(0.0, 10.0), DataPoint::new(100.0, 90.0)],
                candles: vec![],
                time_axis_label_config: None,
                price_axis_label_config: None,
                tolerance: None,
                baseline_png_relpath: None,
                events: vec![
                    serde_json::json!({
                        "type": "price_scale_mode",
                        "mode": "log"
                    }),
                    serde_json::json!({
                        "event": "axis_scale_price",
                        "deltaY": 120.0,
                        "anchorY": 200.0
                    }),
                    serde_json::json!({
                        "kind": "crosshair_move",
                        "x": 330.0,
                        "y": 120.0
                    }),
                ],
            }],
        };

        let corpus = import_lightweight_visual_capture(capture).expect("import visual capture");
        assert_eq!(corpus.schema_version, 1);
        assert_eq!(corpus.trace_name.as_deref(), Some("visual-raw"));
        assert!(
            corpus
                .source_notes
                .as_deref()
                .expect("source notes present")
                .contains("without manual normalization")
        );
        assert_eq!(corpus.fixtures.len(), 1);

        let fixture = &corpus.fixtures[0];
        assert_eq!(fixture.id, "LWC Fixture 01");
        assert_eq!(
            fixture.baseline_png_relpath,
            "tests/fixtures/lightweight_visual_differential/reference_png/lwc-fixture-01.png"
        );
        assert_eq!(fixture.tolerance.max_channel_abs_diff, 2);
        assert_eq!(fixture.tolerance.mean_channel_abs_diff, 0.5);
        assert_eq!(fixture.input.actions.len(), 3);

        assert!(matches!(
            fixture.input.actions[0],
            VisualCorpusAction::SetPriceScaleMode {
                mode: TracePriceScaleMode::Log
            }
        ));
        assert!(matches!(
            fixture.input.actions[1],
            VisualCorpusAction::AxisDragScalePrice { .. }
        ));
        assert!(matches!(
            fixture.input.actions[2],
            VisualCorpusAction::PointerMove { .. }
        ));
    }
}
