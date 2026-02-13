use std::sync::Arc;

use chart_rs::ChartError;
use chart_rs::api::{
    AxisLabelLocale, ChartEngine, ChartEngineConfig, PriceAxisDisplayMode, PriceAxisLabelConfig,
    PriceAxisLabelPolicy,
};
use chart_rs::core::{DataPoint, PriceScaleMode, Viewport};
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

fn is_log_125_ladder(value: f64) -> bool {
    if !value.is_finite() || value <= 0.0 {
        return false;
    }
    let exponent = value.log10().floor();
    let base = 10_f64.powf(exponent);
    let mantissa = value / base;
    (mantissa - 1.0).abs() <= 1e-9
        || (mantissa - 2.0).abs() <= 1e-9
        || (mantissa - 5.0).abs() <= 1e-9
}

#[derive(Clone, Copy, Debug)]
enum FallbackCacheScenario {
    PercentageInvalidExplicitBase,
    PercentageNoneBaseResolvedFromZeroData,
    IndexedNoneBaseResolvedFromZeroDomainWithoutData,
}

fn build_engine_for_fallback_cache_scenario(
    scenario: FallbackCacheScenario,
) -> ChartEngine<NullRenderer> {
    let (domain_min, domain_max, locale, display_mode, points): (
        f64,
        f64,
        AxisLabelLocale,
        PriceAxisDisplayMode,
        Vec<DataPoint>,
    ) = match scenario {
        FallbackCacheScenario::PercentageInvalidExplicitBase => (
            95.0,
            105.0,
            AxisLabelLocale::EnUs,
            PriceAxisDisplayMode::Percentage {
                base_price: Some(f64::NAN),
            },
            vec![
                DataPoint::new(0.0, 100.0),
                DataPoint::new(1.0, 101.0),
                DataPoint::new(2.0, 99.0),
                DataPoint::new(3.0, 102.0),
            ],
        ),
        FallbackCacheScenario::PercentageNoneBaseResolvedFromZeroData => (
            -20.0,
            120.0,
            AxisLabelLocale::EsEs,
            PriceAxisDisplayMode::Percentage { base_price: None },
            vec![
                DataPoint::new(0.0, 0.0),
                DataPoint::new(1.0, 100.0),
                DataPoint::new(2.0, 99.5),
                DataPoint::new(3.0, 101.0),
            ],
        ),
        FallbackCacheScenario::IndexedNoneBaseResolvedFromZeroDomainWithoutData => (
            0.0,
            20.0,
            AxisLabelLocale::EnUs,
            PriceAxisDisplayMode::IndexedTo100 { base_price: None },
            Vec::new(),
        ),
    };

    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 3.0)
        .with_price_domain(domain_min, domain_max);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    if !points.is_empty() {
        engine.set_data(points);
    }
    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale,
            policy: PriceAxisLabelPolicy::FixedDecimals { precision: 2 },
            display_mode,
        })
        .expect("set fallback mode");
    engine
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
            ..PriceAxisLabelConfig::default()
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
            ..PriceAxisLabelConfig::default()
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
            ..PriceAxisLabelConfig::default()
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
            ..PriceAxisLabelConfig::default()
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
            ..PriceAxisLabelConfig::default()
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
            ..PriceAxisLabelConfig::default()
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
            ..PriceAxisLabelConfig::default()
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
            ..PriceAxisLabelConfig::default()
        })
        .expect_err("min_move should fail");
    assert!(matches!(err, ChartError::InvalidData(_)));
}

#[test]
fn percentage_display_mode_uses_percent_suffix() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(95.0, 105.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::FixedDecimals { precision: 2 },
            display_mode: PriceAxisDisplayMode::Percentage {
                base_price: Some(100.0),
            },
        })
        .expect("set percentage mode");

    let frame = engine.build_render_frame().expect("build frame");
    let labels = price_labels(&frame);
    assert!(!labels.is_empty());
    assert!(labels.iter().all(|label| label.ends_with('%')));
}

#[test]
fn indexed_to_100_display_mode_applies_base_transform() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(95.0, 105.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::FixedDecimals { precision: 2 },
            display_mode: PriceAxisDisplayMode::IndexedTo100 {
                base_price: Some(50.0),
            },
        })
        .expect("set indexed mode");

    let frame = engine.build_render_frame().expect("build frame");
    let labels = price_labels(&frame);
    assert!(!labels.is_empty());
    assert!(labels.iter().all(|label| {
        let value = label.parse::<f64>().expect("parse indexed label");
        (180.0..=220.0).contains(&value)
    }));
}

#[test]
fn invalid_price_axis_display_base_falls_back_to_one() {
    fn build_labels(mode: PriceAxisDisplayMode) -> Vec<String> {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0)
            .with_price_domain(0.0, 10.0);
        let mut engine = ChartEngine::new(renderer, config).expect("engine init");
        engine
            .set_price_axis_label_config(PriceAxisLabelConfig {
                locale: AxisLabelLocale::EnUs,
                policy: PriceAxisLabelPolicy::Adaptive,
                display_mode: mode,
            })
            .expect("set display mode");
        let frame = engine.build_render_frame().expect("build frame");
        price_labels(&frame)
            .into_iter()
            .map(ToOwned::to_owned)
            .collect()
    }

    let invalid_bases = [0.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY];

    let percentage_baseline = build_labels(PriceAxisDisplayMode::Percentage { base_price: None });
    assert!(percentage_baseline.iter().all(|label| label.ends_with('%')));
    for base in invalid_bases {
        let labels = build_labels(PriceAxisDisplayMode::Percentage {
            base_price: Some(base),
        });
        assert_eq!(
            labels, percentage_baseline,
            "percentage labels should fallback to 1 for invalid base={base:?}"
        );
    }

    let indexed_baseline = build_labels(PriceAxisDisplayMode::IndexedTo100 { base_price: None });
    for base in invalid_bases {
        let labels = build_labels(PriceAxisDisplayMode::IndexedTo100 {
            base_price: Some(base),
        });
        assert_eq!(
            labels, indexed_baseline,
            "indexed labels should fallback to 1 for invalid base={base:?}"
        );
    }
}

#[test]
fn log_mode_price_axis_labels_follow_125_ladder() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(1.0, 1_000.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine
        .set_price_scale_mode(PriceScaleMode::Log)
        .expect("set log mode");
    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::FixedDecimals { precision: 0 },
            ..PriceAxisLabelConfig::default()
        })
        .expect("set fixed policy");

    let frame = engine.build_render_frame().expect("build frame");
    let labels = price_labels(&frame);
    assert!(!labels.is_empty());
    assert!(labels.iter().all(|label| {
        let value = label.parse::<f64>().expect("parse label");
        is_log_125_ladder(value)
    }));
}

#[test]
fn price_label_cache_reports_hits_for_repeated_frame_builds() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(95.0, 105.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale: AxisLabelLocale::EnUs,
            policy: PriceAxisLabelPolicy::Adaptive,
            ..PriceAxisLabelConfig::default()
        })
        .expect("set adaptive policy");

    engine.clear_price_label_cache();
    let before = engine.price_label_cache_stats();
    assert_eq!(before.hits, 0);
    assert_eq!(before.misses, 0);

    let _ = engine.build_render_frame().expect("first frame");
    let after_first = engine.price_label_cache_stats();
    assert!(after_first.misses > 0);
    assert!(after_first.size > 0);

    let _ = engine.build_render_frame().expect("second frame");
    let after_second = engine.price_label_cache_stats();
    assert!(after_second.hits > after_first.hits);
}

#[test]
fn changing_price_axis_config_clears_price_label_cache_entries() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(820, 420), 0.0, 100.0).with_price_domain(95.0, 105.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let _ = engine.build_render_frame().expect("seed frame");
    assert!(engine.price_label_cache_stats().size > 0);

    let mut config = engine.price_axis_label_config();
    config.policy = PriceAxisLabelPolicy::FixedDecimals { precision: 3 };
    engine
        .set_price_axis_label_config(config)
        .expect("set policy");
    assert_eq!(engine.price_label_cache_stats().size, 0);
}

#[test]
fn price_label_cache_stats_report_hot_hits_for_mixed_fallback_routes() {
    for scenario in [
        FallbackCacheScenario::PercentageInvalidExplicitBase,
        FallbackCacheScenario::PercentageNoneBaseResolvedFromZeroData,
        FallbackCacheScenario::IndexedNoneBaseResolvedFromZeroDomainWithoutData,
    ] {
        let engine = build_engine_for_fallback_cache_scenario(scenario);
        engine.clear_price_label_cache();
        let before = engine.price_label_cache_stats();

        let _ = engine.build_render_frame().expect("first frame");
        let after_first = engine.price_label_cache_stats();
        assert!(
            after_first.misses > before.misses,
            "expected first-pass cache misses for scenario={scenario:?}"
        );
        assert!(
            after_first.size > 0,
            "expected non-empty cache after first frame for scenario={scenario:?}"
        );

        let _ = engine.build_render_frame().expect("second frame");
        let after_second = engine.price_label_cache_stats();
        assert!(
            after_second.hits > after_first.hits,
            "expected cache-hot hits on second frame for scenario={scenario:?}"
        );
    }
}

#[test]
fn price_label_cache_stats_cold_rebuild_penalty_exceeds_hot_second_pass_miss_delta() {
    for scenario in [
        FallbackCacheScenario::PercentageInvalidExplicitBase,
        FallbackCacheScenario::PercentageNoneBaseResolvedFromZeroData,
        FallbackCacheScenario::IndexedNoneBaseResolvedFromZeroDomainWithoutData,
    ] {
        let engine = build_engine_for_fallback_cache_scenario(scenario);

        engine.clear_price_label_cache();
        let _ = engine.build_render_frame().expect("hot first frame");
        let hot_after_first = engine.price_label_cache_stats();
        let _ = engine.build_render_frame().expect("hot second frame");
        let hot_after_second = engine.price_label_cache_stats();
        let hot_second_miss_delta = hot_after_second
            .misses
            .saturating_sub(hot_after_first.misses);

        engine.clear_price_label_cache();
        let _ = engine.build_render_frame().expect("cold first frame");
        let cold_after_first = engine.price_label_cache_stats();
        engine.clear_price_label_cache();
        let _ = engine.build_render_frame().expect("cold second frame");
        let cold_after_second = engine.price_label_cache_stats();
        let cold_second_miss_delta = cold_after_second
            .misses
            .saturating_sub(cold_after_first.misses);

        assert!(
            cold_second_miss_delta > hot_second_miss_delta,
            "expected cold rebuild miss penalty above hot second-pass misses for scenario={scenario:?} (cold={cold_second_miss_delta}, hot={hot_second_miss_delta})"
        );
    }
}
