use std::sync::Arc;

use chart_rs::ChartError;
use chart_rs::api::{
    AxisLabelLocale, ChartEngine, ChartEngineConfig, PriceAxisLabelConfig, PriceAxisLabelPolicy,
};
use chart_rs::core::Viewport;
use chart_rs::render::{NullRenderer, TextHAlign};

fn price_labels(frame: &chart_rs::render::RenderFrame) -> Vec<&str> {
    frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Right)
        .map(|label| label.text.as_str())
        .collect()
}

fn fraction_len(label: &str) -> usize {
    if let Some((_, fraction)) = label.split_once('.') {
        return fraction.len();
    }
    if let Some((_, fraction)) = label.split_once(',') {
        return fraction.len();
    }
    0
}

#[test]
fn price_axis_fixed_decimals_policy_is_applied() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(0.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::FixedDecimals { precision: 4 },
        })
        .expect("set price label config");

    let frame = engine.build_render_frame().expect("build frame");
    let labels = price_labels(&frame);
    assert!(!labels.is_empty());
    assert!(labels.iter().all(|label| fraction_len(label) == 4));
}

#[test]
fn price_axis_locale_es_uses_comma_separator() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(0.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EsEs,
            policy: PriceAxisLabelPolicy::FixedDecimals { precision: 1 },
        })
        .expect("set price label config");

    let frame = engine.build_render_frame().expect("build frame");
    let labels = price_labels(&frame);
    assert!(!labels.is_empty());
    assert!(labels.iter().all(|label| label.contains(',')));
}

#[test]
fn adaptive_price_policy_increases_precision_for_narrow_domains() {
    let renderer = NullRenderer::default();
    let wide_config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(0.0, 1_000.0);
    let mut wide_engine = ChartEngine::new(renderer, wide_config).expect("wide engine init");
    wide_engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::Adaptive,
        })
        .expect("set wide policy");

    let narrow_config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(1.0, 1.02);
    let mut narrow_engine =
        ChartEngine::new(NullRenderer::default(), narrow_config).expect("narrow engine init");
    narrow_engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::Adaptive,
        })
        .expect("set narrow policy");

    let wide_frame = wide_engine.build_render_frame().expect("wide frame");
    let narrow_frame = narrow_engine.build_render_frame().expect("narrow frame");
    let wide_labels = price_labels(&wide_frame);
    let narrow_labels = price_labels(&narrow_frame);
    let wide_max_fraction = wide_labels
        .iter()
        .map(|label| fraction_len(label))
        .max()
        .expect("wide labels");
    let narrow_max_fraction = narrow_labels
        .iter()
        .map(|label| fraction_len(label))
        .max()
        .expect("narrow labels");

    assert!(narrow_max_fraction > wide_max_fraction);
}

#[test]
fn min_move_policy_snaps_price_labels() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(100.0, 101.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::MinMove {
                min_move: 0.25,
                trim_trailing_zeros: false,
            },
        })
        .expect("set min-move policy");

    let frame = engine.build_render_frame().expect("build frame");
    let labels = price_labels(&frame);
    assert!(!labels.is_empty());
    assert!(labels.iter().all(|label| fraction_len(label) == 2));
    assert!(labels.iter().all(|label| {
        let value = label.parse::<f64>().expect("parse label");
        ((value * 4.0).round() - (value * 4.0)).abs() < 1e-6
    }));
}

#[test]
fn min_move_policy_can_trim_trailing_zeros() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(100.0, 101.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::MinMove {
                min_move: 0.5,
                trim_trailing_zeros: true,
            },
        })
        .expect("set min-move policy");

    let frame = engine.build_render_frame().expect("build frame");
    let labels = price_labels(&frame);
    assert!(!labels.is_empty());
    assert!(labels.iter().all(|label| !label.ends_with(".0")));
}

#[test]
fn custom_price_formatter_overrides_policy() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(0.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_price_label_formatter(Arc::new(|value| format!("px={value:.1}")));

    let frame = engine.build_render_frame().expect("build frame");
    let labels = price_labels(&frame);
    assert!(!labels.is_empty());
    assert!(labels.iter().all(|label| label.starts_with("px=")));
}

#[test]
fn invalid_price_axis_precision_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(0.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let err = engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::FixedDecimals { precision: 32 },
        })
        .expect_err("precision should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn invalid_price_axis_min_move_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(0.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let err = engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::MinMove {
                min_move: 0.0,
                trim_trailing_zeros: false,
            },
        })
        .expect_err("min_move should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}
