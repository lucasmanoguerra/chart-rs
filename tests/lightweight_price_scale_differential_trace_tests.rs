use chart_rs::api::{
    ChartEngine, ChartEngineConfig, PriceScaleRealtimeBehavior, PriceScaleTransformedBaseBehavior,
};
use chart_rs::core::{DataPoint, PriceScaleMode, Viewport};
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
    probe_price: Option<f64>,
    probe_pixel: Option<f64>,
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

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DifferentialAction {
    SetPriceScaleMode {
        mode: TracePriceScaleMode,
    },
    SetTransformedBaseBehavior {
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

#[derive(Debug, Deserialize)]
struct DifferentialExpectation {
    base_value: Option<f64>,
    price_domain_start: Option<f64>,
    price_domain_end: Option<f64>,
    price_domain_span: Option<f64>,
    probe_price_pixel: Option<f64>,
    probe_pixel_price: Option<f64>,
}

fn load_trace() -> DifferentialTraceFile {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/lightweight_differential/price_scale_transformed_autoscale_trace.json"
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
fn lightweight_v51_price_scale_differential_trace_transformed_autoscale() {
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
                DifferentialAction::SetPriceScaleMode { mode } => {
                    engine
                        .set_price_scale_mode(mode.into())
                        .expect("set price-scale mode");
                }
                DifferentialAction::SetTransformedBaseBehavior {
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
                        .expect("set transformed-base behavior");
                }
                DifferentialAction::SetTimeVisibleRange { start, end } => {
                    engine
                        .set_time_visible_range(start, end)
                        .expect("set time visible range");
                }
                DifferentialAction::AutoscaleVisibleData => {
                    engine
                        .autoscale_price_from_visible_data()
                        .expect("autoscale visible data");
                }
                DifferentialAction::SetPriceScaleRealtimeBehavior {
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
                DifferentialAction::AppendPoint { time, value } => {
                    engine.append_point(DataPoint::new(time, value));
                }
            }

            let (price_domain_start, price_domain_end) = engine.price_domain();
            let price_domain_span = price_domain_end - price_domain_start;
            let base_value = engine.price_scale_transformed_base_value();

            let probe_price_pixel = scenario.probe_price.map(|probe_price| {
                engine
                    .map_price_to_pixel(probe_price)
                    .expect("probe price to pixel")
            });
            let probe_pixel_price = scenario.probe_pixel.map(|probe_pixel| {
                engine
                    .map_pixel_to_price(probe_pixel)
                    .expect("probe pixel to price")
            });

            let context = format!("scenario={}, step={step_idx}", scenario.id);
            if let Some(expected) = step.expect.base_value {
                let actual = base_value.expect("expected transformed base value");
                assert_close(
                    actual,
                    expected,
                    trace.tolerance,
                    &format!("{context}, base_value"),
                );
            }
            if let Some(expected) = step.expect.price_domain_start {
                assert_close(
                    price_domain_start,
                    expected,
                    trace.tolerance,
                    &format!("{context}, price_domain_start"),
                );
            }
            if let Some(expected) = step.expect.price_domain_end {
                assert_close(
                    price_domain_end,
                    expected,
                    trace.tolerance,
                    &format!("{context}, price_domain_end"),
                );
            }
            if let Some(expected) = step.expect.price_domain_span {
                assert_close(
                    price_domain_span,
                    expected,
                    trace.tolerance,
                    &format!("{context}, price_domain_span"),
                );
            }
            if let Some(expected) = step.expect.probe_price_pixel {
                let actual = probe_price_pixel.expect("expected probe price->pixel result");
                assert_close(
                    actual,
                    expected,
                    trace.tolerance,
                    &format!("{context}, probe_price_pixel"),
                );
            }
            if let Some(expected) = step.expect.probe_pixel_price {
                let actual = probe_pixel_price.expect("expected probe pixel->price result");
                assert_close(
                    actual,
                    expected,
                    trace.tolerance,
                    &format!("{context}, probe_pixel_price"),
                );
            }
        }
    }
}
