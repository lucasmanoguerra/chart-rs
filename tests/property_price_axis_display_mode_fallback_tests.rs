use chart_rs::api::{
    AxisLabelLocale, ChartEngine, ChartEngineConfig, PriceAxisDisplayMode, PriceAxisLabelConfig,
    PriceAxisLabelPolicy,
};
use chart_rs::core::{DataPoint, Viewport};
use chart_rs::render::{NullRenderer, TextHAlign};
use proptest::prelude::*;

fn build_labels(
    samples: [f64; 4],
    locale: AxisLabelLocale,
    display_mode: PriceAxisDisplayMode,
) -> Vec<String> {
    let min = samples.iter().copied().fold(f64::INFINITY, f64::min);
    let max = samples.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (max - min).abs();
    let padding = if span < 1e-9 { 1.0 } else { span * 0.25 };

    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(900, 420), 0.0, 3.0)
        .with_price_domain(min - padding, max + padding);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_data(vec![
        DataPoint::new(0.0, samples[0]),
        DataPoint::new(1.0, samples[1]),
        DataPoint::new(2.0, samples[2]),
        DataPoint::new(3.0, samples[3]),
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
    fn percentage_invalid_explicit_bases_match_base_one_fallback(
        samples in prop::array::uniform4(-1_000.0f64..1_000.0f64),
        use_es_locale in any::<bool>()
    ) {
        let locale = if use_es_locale {
            AxisLabelLocale::EsEs
        } else {
            AxisLabelLocale::EnUs
        };
        let baseline = build_labels(
            samples,
            locale,
            PriceAxisDisplayMode::Percentage {
                base_price: Some(1.0),
            },
        );
        prop_assert!(!baseline.is_empty());
        prop_assert!(baseline.iter().all(|label| label.ends_with('%')));

        for invalid_base in [0.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let candidate = build_labels(
                samples,
                locale,
                PriceAxisDisplayMode::Percentage {
                    base_price: Some(invalid_base),
                },
            );
            prop_assert_eq!(
                &candidate,
                &baseline,
                "percentage labels must fallback to explicit base=1 for invalid base={:?}",
                invalid_base
            );
        }
    }

    #[test]
    fn indexed_invalid_explicit_bases_match_base_one_fallback(
        samples in prop::array::uniform4(-1_000.0f64..1_000.0f64),
        use_es_locale in any::<bool>()
    ) {
        let locale = if use_es_locale {
            AxisLabelLocale::EsEs
        } else {
            AxisLabelLocale::EnUs
        };
        let baseline = build_labels(
            samples,
            locale,
            PriceAxisDisplayMode::IndexedTo100 {
                base_price: Some(1.0),
            },
        );
        prop_assert!(!baseline.is_empty());
        prop_assert!(baseline.iter().all(|label| !label.ends_with('%')));

        for invalid_base in [0.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let candidate = build_labels(
                samples,
                locale,
                PriceAxisDisplayMode::IndexedTo100 {
                    base_price: Some(invalid_base),
                },
            );
            prop_assert_eq!(
                &candidate,
                &baseline,
                "indexed labels must fallback to explicit base=1 for invalid base={:?}",
                invalid_base
            );
        }
    }
}
