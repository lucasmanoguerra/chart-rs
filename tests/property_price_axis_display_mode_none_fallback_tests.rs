use chart_rs::api::{
    AxisLabelLocale, ChartEngine, ChartEngineConfig, PriceAxisDisplayMode, PriceAxisLabelConfig,
    PriceAxisLabelPolicy,
};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::{NullRenderer, TextHAlign};
use proptest::prelude::*;

fn build_labels(
    points: [f64; 4],
    locale: AxisLabelLocale,
    display_mode: PriceAxisDisplayMode,
) -> Vec<String> {
    let min = points.iter().copied().fold(f64::INFINITY, f64::min);
    let max = points.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (max - min).abs();
    let padding = if span < 1e-9 { 1.0 } else { span * 0.25 };

    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 3.0)
        .with_price_domain(min - padding, max + padding);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_data(vec![
        DataPoint::new(0.0, points[0]),
        DataPoint::new(1.0, points[1]),
        DataPoint::new(2.0, points[2]),
        DataPoint::new(3.0, points[3]),
    ]);
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
    fn percentage_none_base_matches_explicit_resolved_non_zero_base(
        resolved_base in -1_000.0f64..1_000.0f64,
        rest in prop::array::uniform3(-1_000.0f64..1_000.0f64),
        use_es_locale in any::<bool>()
    ) {
        prop_assume!(resolved_base.abs() > 1e-9);
        let locale = if use_es_locale {
            AxisLabelLocale::EsEs
        } else {
            AxisLabelLocale::EnUs
        };
        let points = [resolved_base, rest[0], rest[1], rest[2]];

        let with_none = build_labels(
            points,
            locale,
            PriceAxisDisplayMode::Percentage {
                base_price: None,
            },
        );
        let with_explicit = build_labels(
            points,
            locale,
            PriceAxisDisplayMode::Percentage {
                base_price: Some(resolved_base),
            },
        );

        prop_assert!(!with_none.is_empty());
        prop_assert!(with_none.iter().all(|label| label.ends_with('%')));
        prop_assert_eq!(&with_none, &with_explicit);
    }

    #[test]
    fn indexed_none_base_matches_explicit_resolved_non_zero_base(
        resolved_base in -1_000.0f64..1_000.0f64,
        rest in prop::array::uniform3(-1_000.0f64..1_000.0f64),
        use_es_locale in any::<bool>()
    ) {
        prop_assume!(resolved_base.abs() > 1e-9);
        let locale = if use_es_locale {
            AxisLabelLocale::EsEs
        } else {
            AxisLabelLocale::EnUs
        };
        let points = [resolved_base, rest[0], rest[1], rest[2]];

        let with_none = build_labels(
            points,
            locale,
            PriceAxisDisplayMode::IndexedTo100 {
                base_price: None,
            },
        );
        let with_explicit = build_labels(
            points,
            locale,
            PriceAxisDisplayMode::IndexedTo100 {
                base_price: Some(resolved_base),
            },
        );

        prop_assert!(!with_none.is_empty());
        prop_assert!(with_none.iter().all(|label| !label.ends_with('%')));
        prop_assert_eq!(&with_none, &with_explicit);
    }

    #[test]
    fn percentage_none_base_with_zero_resolved_base_matches_explicit_one(
        rest in prop::array::uniform3(-1_000.0f64..1_000.0f64),
        use_es_locale in any::<bool>()
    ) {
        let locale = if use_es_locale {
            AxisLabelLocale::EsEs
        } else {
            AxisLabelLocale::EnUs
        };
        let points = [0.0, rest[0], rest[1], rest[2]];

        let with_none = build_labels(
            points,
            locale,
            PriceAxisDisplayMode::Percentage {
                base_price: None,
            },
        );
        let with_one = build_labels(
            points,
            locale,
            PriceAxisDisplayMode::Percentage {
                base_price: Some(1.0),
            },
        );

        prop_assert!(!with_none.is_empty());
        prop_assert_eq!(&with_none, &with_one);
    }

    #[test]
    fn indexed_none_base_with_zero_resolved_base_matches_explicit_one(
        rest in prop::array::uniform3(-1_000.0f64..1_000.0f64),
        use_es_locale in any::<bool>()
    ) {
        let locale = if use_es_locale {
            AxisLabelLocale::EsEs
        } else {
            AxisLabelLocale::EnUs
        };
        let points = [0.0, rest[0], rest[1], rest[2]];

        let with_none = build_labels(
            points,
            locale,
            PriceAxisDisplayMode::IndexedTo100 {
                base_price: None,
            },
        );
        let with_one = build_labels(
            points,
            locale,
            PriceAxisDisplayMode::IndexedTo100 {
                base_price: Some(1.0),
            },
        );

        prop_assert!(!with_none.is_empty());
        prop_assert_eq!(&with_none, &with_one);
    }
}
