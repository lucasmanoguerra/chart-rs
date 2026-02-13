use chart_rs::api::{
    ChartEngine, ChartEngineConfig, PriceAxisDisplayMode, PriceAxisLabelConfig,
    PriceScaleRealtimeBehavior, RenderStyle, TimeAxisLabelConfig,
};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::{NullRenderer, RenderFrame, TextHAlign};
use serde::Deserialize;

const FIXTURE_CORPUS_JSON: &str =
    include_str!("fixtures/axis_section_sizing/axis_section_sizing_corpus.json");
const AXIS_EPSILON: f64 = 1e-6;

#[derive(Debug, Deserialize)]
struct FixtureCorpus {
    schema_version: u32,
    fixtures: Vec<AxisSectionSizingFixture>,
}

#[derive(Debug, Deserialize)]
struct AxisSectionSizingFixture {
    id: String,
    description: String,
    input: FixtureInput,
    expected: ExpectedLayoutSignature,
}

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

#[derive(Debug, Clone, Copy, Deserialize)]
struct FixturePriceAxisScaleStep {
    delta_y_px: f64,
    anchor_y_px: f64,
    #[serde(default = "default_scale_strength")]
    scale_strength: f64,
    #[serde(default = "default_min_span")]
    min_span: f64,
}

fn default_scale_strength() -> f64 {
    0.2
}

fn default_min_span() -> f64 {
    1e-6
}

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

#[derive(Debug, Deserialize)]
struct ExpectedLayoutSignature {
    plot_right_px: f64,
    plot_bottom_px: f64,
    price_axis_width_px: f64,
    time_axis_height_px: f64,
    price_label_count: usize,
    time_label_count: usize,
    major_time_label_count: usize,
    #[serde(default)]
    leftmost_time_label_text: Option<String>,
    #[serde(default)]
    top_price_label_text: Option<String>,
    #[serde(default)]
    major_time_tick_mark_count: Option<usize>,
}

#[derive(Debug)]
struct LayoutSignature {
    plot_right_px: f64,
    plot_bottom_px: f64,
    price_axis_width_px: f64,
    time_axis_height_px: f64,
    price_label_count: usize,
    time_label_count: usize,
    major_time_label_count: usize,
    leftmost_time_label_text: Option<String>,
    top_price_label_text: Option<String>,
    major_time_tick_mark_count: usize,
}

#[test]
fn axis_section_sizing_visual_fixture_corpus_matches_reference_signatures() {
    let corpus: FixtureCorpus =
        serde_json::from_str(FIXTURE_CORPUS_JSON).expect("fixture corpus should parse");
    assert_eq!(
        corpus.schema_version, 1,
        "unexpected fixture schema version"
    );

    let mut mismatches = Vec::new();

    for fixture in &corpus.fixtures {
        let actual = run_fixture(fixture).expect("fixture should build");
        let expected = &fixture.expected;

        let mut fixture_errors = Vec::new();
        assert_close(
            &mut fixture_errors,
            "plot_right_px",
            expected.plot_right_px,
            actual.plot_right_px,
        );
        assert_close(
            &mut fixture_errors,
            "plot_bottom_px",
            expected.plot_bottom_px,
            actual.plot_bottom_px,
        );
        assert_close(
            &mut fixture_errors,
            "price_axis_width_px",
            expected.price_axis_width_px,
            actual.price_axis_width_px,
        );
        assert_close(
            &mut fixture_errors,
            "time_axis_height_px",
            expected.time_axis_height_px,
            actual.time_axis_height_px,
        );
        assert_equal(
            &mut fixture_errors,
            "price_label_count",
            expected.price_label_count,
            actual.price_label_count,
        );
        assert_equal(
            &mut fixture_errors,
            "time_label_count",
            expected.time_label_count,
            actual.time_label_count,
        );
        assert_equal(
            &mut fixture_errors,
            "major_time_label_count",
            expected.major_time_label_count,
            actual.major_time_label_count,
        );
        assert_optional_string(
            &mut fixture_errors,
            "leftmost_time_label_text",
            expected.leftmost_time_label_text.as_deref(),
            actual.leftmost_time_label_text.as_deref(),
        );
        assert_optional_string(
            &mut fixture_errors,
            "top_price_label_text",
            expected.top_price_label_text.as_deref(),
            actual.top_price_label_text.as_deref(),
        );
        assert_optional_usize(
            &mut fixture_errors,
            "major_time_tick_mark_count",
            expected.major_time_tick_mark_count,
            actual.major_time_tick_mark_count,
        );

        if !fixture_errors.is_empty() {
            mismatches.push(format!(
                "fixture `{}` ({}) mismatches:\n{}\nactual signature: {:?}",
                fixture.id,
                fixture.description,
                fixture_errors.join("\n"),
                actual
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "axis-section sizing fixture drift detected:\n{}",
        mismatches.join("\n\n")
    );
}

fn run_fixture(fixture: &AxisSectionSizingFixture) -> chart_rs::ChartResult<LayoutSignature> {
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
    let renderer = NullRenderer::default();
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

    let frame = engine.build_render_frame()?;
    Ok(compute_layout_signature(&frame, style, input.viewport))
}

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

fn compute_layout_signature(
    frame: &RenderFrame,
    style: RenderStyle,
    viewport: Viewport,
) -> LayoutSignature {
    let viewport_width = f64::from(viewport.width);
    let viewport_height = f64::from(viewport.height);

    let plot_bottom = frame
        .lines
        .iter()
        .find(|line| {
            line.color == style.axis_border_color
                && (line.stroke_width - style.axis_line_width).abs() <= AXIS_EPSILON
                && (line.y1 - line.y2).abs() <= AXIS_EPSILON
                && (line.x1 - 0.0).abs() <= AXIS_EPSILON
                && (line.x2 - viewport_width).abs() <= AXIS_EPSILON
        })
        .map_or(
            (viewport_height - style.time_axis_height_px).clamp(0.0, viewport_height),
            |line| line.y1,
        );
    let plot_right = frame
        .lines
        .iter()
        .find(|line| {
            line.color == style.axis_border_color
                && (line.stroke_width - style.axis_line_width).abs() <= AXIS_EPSILON
                && (line.x1 - line.x2).abs() <= AXIS_EPSILON
                && (line.y1 - 0.0).abs() <= AXIS_EPSILON
                && (line.y2 - viewport_height).abs() <= AXIS_EPSILON
        })
        .map_or(
            (viewport_width - style.price_axis_width_px).clamp(0.0, viewport_width),
            |line| line.x1,
        );

    let price_label_count = frame
        .texts
        .iter()
        .filter(|text| text.h_align == TextHAlign::Right)
        .count();
    let time_label_count = frame
        .texts
        .iter()
        .filter(|text| text.h_align == TextHAlign::Center)
        .count();
    let major_time_label_count = frame
        .texts
        .iter()
        .filter(|text| {
            text.h_align == TextHAlign::Center
                && (text.font_size_px - style.major_time_label_font_size_px).abs() <= AXIS_EPSILON
        })
        .count();
    let leftmost_time_label_text = frame
        .texts
        .iter()
        .filter(|text| text.h_align == TextHAlign::Center)
        .min_by(|left, right| {
            left.x
                .total_cmp(&right.x)
                .then_with(|| left.y.total_cmp(&right.y))
        })
        .map(|text| text.text.clone());
    let top_price_label_text = frame
        .texts
        .iter()
        .filter(|text| text.h_align == TextHAlign::Right)
        .min_by(|left, right| {
            left.y
                .total_cmp(&right.y)
                .then_with(|| left.x.total_cmp(&right.x))
        })
        .map(|text| text.text.clone());
    let major_time_tick_end =
        (plot_bottom + style.major_time_tick_mark_length_px).min(viewport_height);
    let major_time_tick_mark_count = frame
        .lines
        .iter()
        .filter(|line| {
            line.color == style.major_time_tick_mark_color
                && (line.stroke_width - style.major_time_tick_mark_width).abs() <= AXIS_EPSILON
                && (line.x1 - line.x2).abs() <= AXIS_EPSILON
                && (line.y1 - plot_bottom).abs() <= AXIS_EPSILON
                && (line.y2 - major_time_tick_end).abs() <= AXIS_EPSILON
        })
        .count();

    LayoutSignature {
        plot_right_px: plot_right,
        plot_bottom_px: plot_bottom,
        price_axis_width_px: viewport_width - plot_right,
        time_axis_height_px: viewport_height - plot_bottom,
        price_label_count,
        time_label_count,
        major_time_label_count,
        leftmost_time_label_text,
        top_price_label_text,
        major_time_tick_mark_count,
    }
}

fn assert_close(errors: &mut Vec<String>, field: &str, expected: f64, actual: f64) {
    if (expected - actual).abs() > AXIS_EPSILON {
        errors.push(format!(
            "- {field}: expected {expected:.6}, got {actual:.6}"
        ));
    }
}

fn assert_equal(errors: &mut Vec<String>, field: &str, expected: usize, actual: usize) {
    if expected != actual {
        errors.push(format!("- {field}: expected {expected}, got {actual}"));
    }
}

fn assert_optional_string(
    errors: &mut Vec<String>,
    field: &str,
    expected: Option<&str>,
    actual: Option<&str>,
) {
    if let Some(expected_value) = expected {
        if actual != Some(expected_value) {
            errors.push(format!(
                "- {field}: expected {:?}, got {:?}",
                Some(expected_value),
                actual
            ));
        }
    }
}

fn assert_optional_usize(
    errors: &mut Vec<String>,
    field: &str,
    expected: Option<usize>,
    actual: usize,
) {
    if let Some(expected_value) = expected {
        if expected_value != actual {
            errors.push(format!(
                "- {field}: expected {expected_value}, got {actual}"
            ));
        }
    }
}
