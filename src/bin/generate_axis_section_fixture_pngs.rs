#[cfg(feature = "cairo-backend")]
use chart_rs::api::{
    ChartEngine, ChartEngineConfig, PriceAxisDisplayMode, PriceAxisLabelConfig,
    PriceScaleRealtimeBehavior, RenderStyle, TimeAxisLabelConfig,
};
#[cfg(feature = "cairo-backend")]
use chart_rs::core::{DataPoint, Viewport};
#[cfg(feature = "cairo-backend")]
use serde::Deserialize;
#[cfg(feature = "cairo-backend")]
use std::fs::{self, File};
#[cfg(feature = "cairo-backend")]
use std::path::{Path, PathBuf};

#[cfg(feature = "cairo-backend")]
const DEFAULT_MANIFEST_PATH: &str =
    "tests/fixtures/axis_section_sizing/axis_section_sizing_corpus.json";
#[cfg(feature = "cairo-backend")]
const DEFAULT_OUTPUT_ROOT: &str = "tests/fixtures/axis_section_sizing/reference_png";

#[cfg(feature = "cairo-backend")]
#[derive(Debug, Deserialize)]
struct FixtureCorpus {
    schema_version: u32,
    fixtures: Vec<AxisSectionSizingFixture>,
}

#[cfg(feature = "cairo-backend")]
#[derive(Debug, Deserialize)]
struct AxisSectionSizingFixture {
    id: String,
    description: String,
    input: FixtureInput,
    #[serde(default)]
    artifacts: FixtureArtifacts,
}

#[cfg(feature = "cairo-backend")]
#[derive(Debug, Deserialize, Default)]
struct FixtureArtifacts {
    reference_png_relpath: Option<String>,
}

#[cfg(feature = "cairo-backend")]
#[derive(Debug, Deserialize)]
struct FixtureInput {
    viewport: Viewport,
    time_range: [f64; 2],
    #[serde(default)]
    time_visible_range_override: Option<[f64; 2]>,
    price_domain: [f64; 2],
    #[serde(default)]
    disable_autoscale_on_data_set: bool,
    points: Vec<DataPoint>,
    #[serde(default)]
    price_axis_scale_steps: Vec<FixturePriceAxisScaleStep>,
    #[serde(default)]
    render_style_overrides: RenderStyleOverrides,
    #[serde(default)]
    time_axis_label_config: Option<TimeAxisLabelConfig>,
    #[serde(default)]
    price_axis_label_config: Option<PriceAxisLabelConfig>,
    #[serde(default)]
    price_axis_display_base_override: Option<FixtureDisplayBaseOverride>,
}

#[cfg(feature = "cairo-backend")]
#[derive(Debug, Clone, Copy, Deserialize)]
struct FixturePriceAxisScaleStep {
    delta_y_px: f64,
    anchor_y_px: f64,
    #[serde(default = "default_scale_strength")]
    scale_strength: f64,
    #[serde(default = "default_min_span")]
    min_span: f64,
}

#[cfg(feature = "cairo-backend")]
fn default_scale_strength() -> f64 {
    0.2
}

#[cfg(feature = "cairo-backend")]
fn default_min_span() -> f64 {
    1e-6
}

#[cfg(feature = "cairo-backend")]
#[derive(Debug, Clone, Copy, Deserialize)]
enum FixtureDisplayBaseOverride {
    #[serde(rename = "zero")]
    Zero,
    #[serde(rename = "nan")]
    NaN,
    #[serde(rename = "pos_inf")]
    PosInf,
    #[serde(rename = "neg_inf")]
    NegInf,
}

#[cfg(feature = "cairo-backend")]
impl FixtureDisplayBaseOverride {
    #[must_use]
    fn to_f64(self) -> f64 {
        match self {
            Self::Zero => 0.0,
            Self::NaN => f64::NAN,
            Self::PosInf => f64::INFINITY,
            Self::NegInf => f64::NEG_INFINITY,
        }
    }
}

#[cfg(feature = "cairo-backend")]
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RenderStyleOverrides {
    price_axis_width_px: Option<f64>,
    time_axis_height_px: Option<f64>,
    price_axis_label_font_size_px: Option<f64>,
    price_axis_label_padding_right_px: Option<f64>,
    time_axis_label_font_size_px: Option<f64>,
    major_time_label_font_size_px: Option<f64>,
    time_axis_label_offset_y_px: Option<f64>,
    major_time_label_offset_y_px: Option<f64>,
    time_axis_tick_mark_length_px: Option<f64>,
    major_time_tick_mark_length_px: Option<f64>,
    show_time_axis_tick_marks: Option<bool>,
    show_major_time_tick_marks: Option<bool>,
    show_last_price_label: Option<bool>,
    show_last_price_line: Option<bool>,
}

#[cfg(feature = "cairo-backend")]
#[derive(Debug)]
struct CliArgs {
    manifest_path: PathBuf,
    output_root: PathBuf,
    only_fixture_id: Option<String>,
}

#[cfg(feature = "cairo-backend")]
fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "cairo-backend"))]
fn main() {
    eprintln!("this tool requires feature `cairo-backend`");
    std::process::exit(1);
}

#[cfg(feature = "cairo-backend")]
fn run() -> Result<(), String> {
    use chart_rs::render::{CairoRenderer, Renderer};

    let args = parse_args()?;
    let raw = fs::read_to_string(&args.manifest_path).map_err(|err| {
        format!(
            "failed to read manifest `{}`: {err}",
            args.manifest_path.display()
        )
    })?;
    let corpus: FixtureCorpus = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse manifest json: {err}"))?;
    if corpus.schema_version != 1 {
        return Err(format!(
            "unsupported fixture schema version: {}",
            corpus.schema_version
        ));
    }

    let mut generated_count = 0usize;
    for fixture in &corpus.fixtures {
        if args
            .only_fixture_id
            .as_ref()
            .is_some_and(|id| id != &fixture.id)
        {
            continue;
        }

        let frame = build_frame_from_fixture(fixture)
            .map_err(|err| format!("fixture `{}` frame build failed: {err}", fixture.id))?;
        let viewport = fixture.input.viewport;
        let width = i32::try_from(viewport.width)
            .map_err(|_| format!("fixture `{}` viewport width overflows i32", fixture.id))?;
        let height = i32::try_from(viewport.height)
            .map_err(|_| format!("fixture `{}` viewport height overflows i32", fixture.id))?;

        let mut renderer = CairoRenderer::new(width, height)
            .map_err(|err| format!("fixture `{}` renderer init failed: {err}", fixture.id))?;
        renderer
            .render(&frame)
            .map_err(|err| format!("fixture `{}` render failed: {err}", fixture.id))?;

        let output_path = resolve_output_path(fixture, &args.output_root);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                format!("failed to create output dir `{}`: {err}", parent.display())
            })?;
        }
        let mut file = File::create(&output_path).map_err(|err| {
            format!(
                "failed to create png `{}` for fixture `{}`: {err}",
                output_path.display(),
                fixture.id
            )
        })?;
        renderer
            .surface()
            .write_to_png(&mut file)
            .map_err(|err| format!("failed to write png `{}`: {err}", output_path.display()))?;

        generated_count += 1;
        println!(
            "generated {} [{}] -> {}",
            fixture.id,
            fixture.description,
            output_path.display()
        );
    }

    println!("done: generated {generated_count} fixture png(s)");
    Ok(())
}

#[cfg(feature = "cairo-backend")]
fn parse_args() -> Result<CliArgs, String> {
    let mut manifest_path = PathBuf::from(DEFAULT_MANIFEST_PATH);
    let mut output_root = PathBuf::from(DEFAULT_OUTPUT_ROOT);
    let mut only_fixture_id: Option<String> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --manifest".to_owned())?;
                manifest_path = PathBuf::from(value);
            }
            "--output-root" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --output-root".to_owned())?;
                output_root = PathBuf::from(value);
            }
            "--only" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --only".to_owned())?;
                only_fixture_id = Some(value);
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => {
                return Err(format!("unknown argument `{arg}`\n\n{}", usage_message()));
            }
        }
    }

    Ok(CliArgs {
        manifest_path,
        output_root,
        only_fixture_id,
    })
}

#[cfg(feature = "cairo-backend")]
fn print_usage() {
    println!("{}", usage_message());
}

#[cfg(feature = "cairo-backend")]
fn usage_message() -> String {
    format!(
        "Usage: cargo run --features cairo-backend --bin generate_axis_section_fixture_pngs -- [options]\n\nOptions:\n  --manifest <path>      Fixture manifest path (default: {DEFAULT_MANIFEST_PATH})\n  --output-root <path>   Output root when fixture has no artifact path (default: {DEFAULT_OUTPUT_ROOT})\n  --only <fixture-id>    Generate a single fixture by id\n  -h, --help             Show this message"
    )
}

#[cfg(feature = "cairo-backend")]
fn resolve_output_path(fixture: &AxisSectionSizingFixture, output_root: &Path) -> PathBuf {
    if let Some(relpath) = &fixture.artifacts.reference_png_relpath {
        PathBuf::from(relpath)
    } else {
        output_root.join(format!("{}.png", fixture.id))
    }
}

#[cfg(feature = "cairo-backend")]
fn build_frame_from_fixture(
    fixture: &AxisSectionSizingFixture,
) -> chart_rs::ChartResult<chart_rs::render::RenderFrame> {
    let input = &fixture.input;
    let mut config =
        ChartEngineConfig::new(input.viewport, input.time_range[0], input.time_range[1])
            .with_price_domain(input.price_domain[0], input.price_domain[1]);
    if input.disable_autoscale_on_data_set {
        config = config.with_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
            autoscale_on_data_set: false,
            autoscale_on_data_update: false,
            autoscale_on_time_range_change: false,
        });
    }
    let renderer = chart_rs::render::NullRenderer::default();
    let mut engine = ChartEngine::new(renderer, config)?;

    if !input.points.is_empty() {
        engine.set_data(input.points.clone());
    }
    if let Some([visible_start, visible_end]) = input.time_visible_range_override {
        engine.set_time_visible_range(visible_start, visible_end)?;
    }
    for step in &input.price_axis_scale_steps {
        let _ = engine.axis_drag_scale_price(
            step.delta_y_px,
            step.anchor_y_px,
            step.scale_strength,
            step.min_span,
        )?;
    }
    if let Some(time_axis_config) = input.time_axis_label_config {
        engine.set_time_axis_label_config(time_axis_config)?;
    }
    if let Some(mut price_axis_config) = input.price_axis_label_config {
        if let Some(base_override) = input.price_axis_display_base_override {
            apply_display_base_override(&mut price_axis_config, base_override);
        }
        engine.set_price_axis_label_config(price_axis_config)?;
    }

    let mut style = engine.render_style();
    apply_style_overrides(&mut style, &input.render_style_overrides);
    engine.set_render_style(style)?;

    engine.build_render_frame()
}

#[cfg(feature = "cairo-backend")]
fn apply_display_base_override(
    config: &mut PriceAxisLabelConfig,
    override_base: FixtureDisplayBaseOverride,
) {
    let base_price = Some(override_base.to_f64());
    config.display_mode = match config.display_mode {
        PriceAxisDisplayMode::Normal => PriceAxisDisplayMode::Normal,
        PriceAxisDisplayMode::Percentage { .. } => PriceAxisDisplayMode::Percentage { base_price },
        PriceAxisDisplayMode::IndexedTo100 { .. } => {
            PriceAxisDisplayMode::IndexedTo100 { base_price }
        }
    };
}

#[cfg(feature = "cairo-backend")]
fn apply_style_overrides(style: &mut RenderStyle, overrides: &RenderStyleOverrides) {
    if let Some(value) = overrides.price_axis_width_px {
        style.price_axis_width_px = value;
    }
    if let Some(value) = overrides.time_axis_height_px {
        style.time_axis_height_px = value;
    }
    if let Some(value) = overrides.price_axis_label_font_size_px {
        style.price_axis_label_font_size_px = value;
    }
    if let Some(value) = overrides.price_axis_label_padding_right_px {
        style.price_axis_label_padding_right_px = value;
    }
    if let Some(value) = overrides.time_axis_label_font_size_px {
        style.time_axis_label_font_size_px = value;
    }
    if let Some(value) = overrides.major_time_label_font_size_px {
        style.major_time_label_font_size_px = value;
    }
    if let Some(value) = overrides.time_axis_label_offset_y_px {
        style.time_axis_label_offset_y_px = value;
    }
    if let Some(value) = overrides.major_time_label_offset_y_px {
        style.major_time_label_offset_y_px = value;
    }
    if let Some(value) = overrides.time_axis_tick_mark_length_px {
        style.time_axis_tick_mark_length_px = value;
    }
    if let Some(value) = overrides.major_time_tick_mark_length_px {
        style.major_time_tick_mark_length_px = value;
    }
    if let Some(value) = overrides.show_time_axis_tick_marks {
        style.show_time_axis_tick_marks = value;
    }
    if let Some(value) = overrides.show_major_time_tick_marks {
        style.show_major_time_tick_marks = value;
    }
    if let Some(value) = overrides.show_last_price_label {
        style.show_last_price_label = value;
    }
    if let Some(value) = overrides.show_last_price_line {
        style.show_last_price_line = value;
    }
}
