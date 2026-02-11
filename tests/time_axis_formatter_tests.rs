use std::sync::Arc;

use chart_rs::ChartError;
use chart_rs::api::{
    AxisLabelLocale, ChartEngine, ChartEngineConfig, TimeAxisLabelConfig, TimeAxisLabelPolicy,
};
use chart_rs::core::Viewport;
use chart_rs::render::{NullRenderer, TextHAlign};

#[test]
fn time_axis_decimal_locale_es_uses_comma_separator() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(600, 300), 0.0, 100.0).with_price_domain(0.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            locale: AxisLabelLocale::EsEs,
            policy: TimeAxisLabelPolicy::LogicalDecimal { precision: 1 },
        })
        .expect("set label config");

    let frame = engine.build_render_frame().expect("build frame");
    let time_labels: Vec<&str> = frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .map(|label| label.text.as_str())
        .collect();

    assert!(!time_labels.is_empty());
    assert!(time_labels.iter().all(|text| text.contains(',')));
}

#[test]
fn time_axis_datetime_policy_formats_utc_labels() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(700, 320), 1_700_000_000.0, 1_700_010_000.0)
        .with_price_domain(0.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: TimeAxisLabelPolicy::UtcDateTime {
                show_seconds: false,
            },
        })
        .expect("set label config");

    let frame = engine.build_render_frame().expect("build frame");
    let time_labels: Vec<&str> = frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .map(|label| label.text.as_str())
        .collect();

    assert!(!time_labels.is_empty());
    assert!(
        time_labels
            .iter()
            .all(|text| text.contains('-') && text.contains(':'))
    );
}

#[test]
fn custom_time_label_formatter_overrides_policy() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(640, 300), 10.0, 20.0).with_price_domain(0.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_time_label_formatter(Arc::new(|value| format!("t={value:.0}")));
    let frame = engine.build_render_frame().expect("build frame");
    let time_labels: Vec<&str> = frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .map(|label| label.text.as_str())
        .collect();

    assert!(!time_labels.is_empty());
    assert!(time_labels.iter().all(|text| text.starts_with("t=")));
}

#[test]
fn utc_adaptive_policy_changes_label_detail_with_zoom() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(
        Viewport::new(900, 360),
        1_700_000_000.0,
        1_700_000_000.0 + 1_728_000.0,
    )
    .with_price_domain(0.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: TimeAxisLabelPolicy::UtcAdaptive,
        })
        .expect("set adaptive policy");

    let wide_frame = engine.build_render_frame().expect("wide frame");
    let wide_time_labels: Vec<&str> = wide_frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .map(|label| label.text.as_str())
        .collect();
    assert!(!wide_time_labels.is_empty());
    assert!(wide_time_labels.iter().all(|text| !text.contains(':')));

    engine
        .set_time_visible_range(1_700_000_000.0, 1_700_000_300.0)
        .expect("set narrow range");
    let narrow_frame = engine.build_render_frame().expect("narrow frame");
    let narrow_time_labels: Vec<&str> = narrow_frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Center)
        .map(|label| label.text.as_str())
        .collect();
    assert!(!narrow_time_labels.is_empty());
    assert!(
        narrow_time_labels
            .iter()
            .all(|text| text.matches(':').count() >= 2)
    );
}

#[test]
fn time_label_cache_reports_hits_for_repeated_frame_builds() {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(
        Viewport::new(900, 360),
        1_700_000_000.0,
        1_700_000_000.0 + 86_400.0,
    )
    .with_price_domain(0.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: TimeAxisLabelPolicy::UtcAdaptive,
        })
        .expect("set adaptive policy");
    engine.clear_time_label_cache();

    let _ = engine.build_render_frame().expect("first frame");
    let first_stats = engine.time_label_cache_stats();
    assert!(first_stats.misses > 0);

    let _ = engine.build_render_frame().expect("second frame");
    let second_stats = engine.time_label_cache_stats();
    assert!(second_stats.hits > first_stats.hits);
    assert_eq!(second_stats.misses, first_stats.misses);
}

#[test]
fn invalid_time_axis_precision_is_rejected() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(640, 300), 10.0, 20.0).with_price_domain(0.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let err = engine
        .set_time_axis_label_config(TimeAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: TimeAxisLabelPolicy::LogicalDecimal { precision: 32 },
        })
        .expect_err("precision should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}
