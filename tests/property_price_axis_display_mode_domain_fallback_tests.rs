use chart_rs::api::{
    AxisLabelLocale, ChartEngine, ChartEngineConfig, PriceAxisDisplayMode, PriceAxisLabelConfig,
    PriceAxisLabelPolicy,
};
use chart_rs::core::Viewport;
use chart_rs::render::{NullRenderer, TextHAlign};
use proptest::prelude::*;

fn build_labels_without_data(
    domain_min: f64,
    domain_max: f64,
    locale: AxisLabelLocale,
    display_mode: PriceAxisDisplayMode,
) -> Vec<String> {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 3.0)
        .with_price_domain(domain_min, domain_max);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

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

proptest! {
    #[test]
    fn percentage_none_base_without_data_matches_explicit_domain_min_when_non_zero(
        domain_min in -1_000.0f64..1_000.0f64,
        domain_span in 1.0f64..2_000.0f64,
        use_es_locale in any::<bool>(),
    ) {
        prop_assume!(domain_min.abs() > 1e-9);
        let locale = if use_es_locale {
            AxisLabelLocale::EsEs
        } else {
            AxisLabelLocale::EnUs
        };
        let domain_max = domain_min + domain_span;

        let with_none = build_labels_without_data(
            domain_min,
            domain_max,
            locale,
            PriceAxisDisplayMode::Percentage { base_price: None },
        );
        let with_explicit = build_labels_without_data(
            domain_min,
            domain_max,
            locale,
            PriceAxisDisplayMode::Percentage {
                base_price: Some(domain_min),
            },
        );

        prop_assert!(!with_none.is_empty());
        prop_assert!(with_none.iter().all(|label| label.ends_with('%')));
        prop_assert_eq!(&with_none, &with_explicit);
    }

    #[test]
    fn indexed_none_base_without_data_matches_explicit_domain_min_when_non_zero(
        domain_min in -1_000.0f64..1_000.0f64,
        domain_span in 1.0f64..2_000.0f64,
        use_es_locale in any::<bool>(),
    ) {
        prop_assume!(domain_min.abs() > 1e-9);
        let locale = if use_es_locale {
            AxisLabelLocale::EsEs
        } else {
            AxisLabelLocale::EnUs
        };
        let domain_max = domain_min + domain_span;

        let with_none = build_labels_without_data(
            domain_min,
            domain_max,
            locale,
            PriceAxisDisplayMode::IndexedTo100 { base_price: None },
        );
        let with_explicit = build_labels_without_data(
            domain_min,
            domain_max,
            locale,
            PriceAxisDisplayMode::IndexedTo100 {
                base_price: Some(domain_min),
            },
        );

        prop_assert!(!with_none.is_empty());
        prop_assert!(with_none.iter().all(|label| !label.ends_with('%')));
        prop_assert_eq!(&with_none, &with_explicit);
    }

    #[test]
    fn percentage_none_base_without_data_zero_domain_min_matches_explicit_one(
        domain_span in 1.0f64..2_000.0f64,
        use_es_locale in any::<bool>(),
    ) {
        let locale = if use_es_locale {
            AxisLabelLocale::EsEs
        } else {
            AxisLabelLocale::EnUs
        };
        let domain_min = 0.0;
        let domain_max = domain_span;

        let with_none = build_labels_without_data(
            domain_min,
            domain_max,
            locale,
            PriceAxisDisplayMode::Percentage { base_price: None },
        );
        let with_one = build_labels_without_data(
            domain_min,
            domain_max,
            locale,
            PriceAxisDisplayMode::Percentage {
                base_price: Some(1.0),
            },
        );

        prop_assert!(!with_none.is_empty());
        prop_assert_eq!(&with_none, &with_one);
    }

    #[test]
    fn indexed_none_base_without_data_zero_domain_min_matches_explicit_one(
        domain_span in 1.0f64..2_000.0f64,
        use_es_locale in any::<bool>(),
    ) {
        let locale = if use_es_locale {
            AxisLabelLocale::EsEs
        } else {
            AxisLabelLocale::EnUs
        };
        let domain_min = 0.0;
        let domain_max = domain_span;

        let with_none = build_labels_without_data(
            domain_min,
            domain_max,
            locale,
            PriceAxisDisplayMode::IndexedTo100 { base_price: None },
        );
        let with_one = build_labels_without_data(
            domain_min,
            domain_max,
            locale,
            PriceAxisDisplayMode::IndexedTo100 {
                base_price: Some(1.0),
            },
        );

        prop_assert!(!with_none.is_empty());
        prop_assert_eq!(&with_none, &with_one);
    }
}
