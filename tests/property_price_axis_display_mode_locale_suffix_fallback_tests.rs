use chart_rs::api::{
    AxisLabelLocale, ChartEngine, ChartEngineConfig, PriceAxisDisplayMode, PriceAxisLabelConfig,
    PriceAxisLabelPolicy,
};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::{NullRenderer, TextHAlign};
use proptest::prelude::*;

#[derive(Clone, Copy, Debug)]
enum DisplayModeKind {
    Percentage,
    IndexedTo100,
}

impl DisplayModeKind {
    fn display_mode(self, base_price: Option<f64>) -> PriceAxisDisplayMode {
        match self {
            Self::Percentage => PriceAxisDisplayMode::Percentage { base_price },
            Self::IndexedTo100 => PriceAxisDisplayMode::IndexedTo100 { base_price },
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum FallbackRoute {
    ExplicitInvalidBase,
    NoneResolvedBaseZeroFromData,
    NoneResolvedBaseNonZeroFromData,
    NoneResolvedBaseZeroFromDomainWithoutData,
    NoneResolvedBaseNonZeroFromDomainWithoutData,
}

fn build_labels(
    domain_min: f64,
    domain_max: f64,
    data: Vec<DataPoint>,
    locale: AxisLabelLocale,
    display_mode: PriceAxisDisplayMode,
) -> Vec<String> {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 3.0)
        .with_price_domain(domain_min, domain_max);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    if !data.is_empty() {
        engine.set_data(data);
    }
    engine
        .set_price_axis_label_config(PriceAxisLabelConfig {
            locale,
            policy: PriceAxisLabelPolicy::FixedDecimals { precision: 2 },
            display_mode,
        })
        .expect("set price axis config");

    let frame = engine.build_render_frame().expect("build frame");
    frame
        .texts
        .iter()
        .filter(|label| label.h_align == TextHAlign::Right)
        .map(|label| label.text.clone())
        .collect()
}

fn build_labels_for_route(
    mode: DisplayModeKind,
    route: FallbackRoute,
    locale: AxisLabelLocale,
    tail: [f64; 3],
    non_zero_resolved_base: f64,
    non_zero_domain_min: f64,
    domain_span: f64,
) -> Vec<String> {
    match route {
        FallbackRoute::ExplicitInvalidBase => build_labels(
            -2_000.0,
            2_000.0,
            vec![
                DataPoint::new(0.0, non_zero_resolved_base),
                DataPoint::new(1.0, tail[0]),
                DataPoint::new(2.0, tail[1]),
                DataPoint::new(3.0, tail[2]),
            ],
            locale,
            mode.display_mode(Some(f64::NAN)),
        ),
        FallbackRoute::NoneResolvedBaseZeroFromData => build_labels(
            -2_000.0,
            2_000.0,
            vec![
                DataPoint::new(0.0, 0.0),
                DataPoint::new(1.0, tail[0]),
                DataPoint::new(2.0, tail[1]),
                DataPoint::new(3.0, tail[2]),
            ],
            locale,
            mode.display_mode(None),
        ),
        FallbackRoute::NoneResolvedBaseNonZeroFromData => build_labels(
            -2_000.0,
            2_000.0,
            vec![
                DataPoint::new(0.0, non_zero_resolved_base),
                DataPoint::new(1.0, tail[0]),
                DataPoint::new(2.0, tail[1]),
                DataPoint::new(3.0, tail[2]),
            ],
            locale,
            mode.display_mode(None),
        ),
        FallbackRoute::NoneResolvedBaseZeroFromDomainWithoutData => build_labels(
            0.0,
            domain_span,
            Vec::new(),
            locale,
            mode.display_mode(None),
        ),
        FallbackRoute::NoneResolvedBaseNonZeroFromDomainWithoutData => build_labels(
            non_zero_domain_min,
            non_zero_domain_min + domain_span,
            Vec::new(),
            locale,
            mode.display_mode(None),
        ),
    }
}

fn assert_locale_separator_and_suffix(
    labels: &[String],
    locale: AxisLabelLocale,
    expect_percent_suffix: bool,
) {
    assert!(!labels.is_empty(), "labels must not be empty");

    for label in labels {
        let numeric_part = if expect_percent_suffix {
            assert!(
                label.ends_with('%'),
                "expected percent suffix for label={label}"
            );
            label.trim_end_matches('%')
        } else {
            assert!(
                !label.ends_with('%'),
                "indexed mode must not append percent suffix label={label}"
            );
            label.as_str()
        };

        match locale {
            AxisLabelLocale::EnUs => {
                assert!(
                    numeric_part.contains('.'),
                    "EnUs labels must use decimal point label={label}"
                );
                assert!(
                    !numeric_part.contains(','),
                    "EnUs labels must not use decimal comma label={label}"
                );
            }
            AxisLabelLocale::EsEs => {
                assert!(
                    numeric_part.contains(','),
                    "EsEs labels must use decimal comma label={label}"
                );
                assert!(
                    !numeric_part.contains('.'),
                    "EsEs labels must not use decimal point label={label}"
                );
            }
        }
    }
}

proptest! {
    #[test]
    fn percentage_fallback_routes_keep_percent_suffix_and_locale_separator(
        tail in prop::array::uniform3(-1_000.0f64..1_000.0f64),
        non_zero_resolved_base in -1_000.0f64..1_000.0f64,
        non_zero_domain_min in -1_000.0f64..1_000.0f64,
        domain_span in 1.0f64..2_000.0f64,
        use_es_locale in any::<bool>(),
    ) {
        prop_assume!(non_zero_resolved_base.abs() > 1e-9);
        prop_assume!(non_zero_domain_min.abs() > 1e-9);
        let locale = if use_es_locale { AxisLabelLocale::EsEs } else { AxisLabelLocale::EnUs };

        for route in [
            FallbackRoute::ExplicitInvalidBase,
            FallbackRoute::NoneResolvedBaseZeroFromData,
            FallbackRoute::NoneResolvedBaseNonZeroFromData,
            FallbackRoute::NoneResolvedBaseZeroFromDomainWithoutData,
            FallbackRoute::NoneResolvedBaseNonZeroFromDomainWithoutData,
        ] {
            let labels = build_labels_for_route(
                DisplayModeKind::Percentage,
                route,
                locale,
                tail,
                non_zero_resolved_base,
                non_zero_domain_min,
                domain_span,
            );
            assert_locale_separator_and_suffix(&labels, locale, true);
        }
    }

    #[test]
    fn indexed_fallback_routes_keep_non_percent_suffix_and_locale_separator(
        tail in prop::array::uniform3(-1_000.0f64..1_000.0f64),
        non_zero_resolved_base in -1_000.0f64..1_000.0f64,
        non_zero_domain_min in -1_000.0f64..1_000.0f64,
        domain_span in 1.0f64..2_000.0f64,
        use_es_locale in any::<bool>(),
    ) {
        prop_assume!(non_zero_resolved_base.abs() > 1e-9);
        prop_assume!(non_zero_domain_min.abs() > 1e-9);
        let locale = if use_es_locale { AxisLabelLocale::EsEs } else { AxisLabelLocale::EnUs };

        for route in [
            FallbackRoute::ExplicitInvalidBase,
            FallbackRoute::NoneResolvedBaseZeroFromData,
            FallbackRoute::NoneResolvedBaseNonZeroFromData,
            FallbackRoute::NoneResolvedBaseZeroFromDomainWithoutData,
            FallbackRoute::NoneResolvedBaseNonZeroFromDomainWithoutData,
        ] {
            let labels = build_labels_for_route(
                DisplayModeKind::IndexedTo100,
                route,
                locale,
                tail,
                non_zero_resolved_base,
                non_zero_domain_min,
                domain_span,
            );
            assert_locale_separator_and_suffix(&labels, locale, false);
        }
    }
}
