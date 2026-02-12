use chart_rs::api::{
    AxisLabelLocale, ChartEngine, ChartEngineConfig, PriceAxisDisplayMode, PriceAxisLabelConfig,
    PriceAxisLabelPolicy, TimeAxisLabelConfig, TimeAxisLabelPolicy, TimeAxisSessionConfig,
    TimeAxisTimeZone,
};
use chart_rs::core::Viewport;
use chart_rs::render::NullRenderer;

#[test]
fn chart_engine_config_defaults_axis_label_bootstrap_fields() {
    let config =
        ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0).with_price_domain(0.0, 1.0);

    assert_eq!(
        config.time_axis_label_config,
        TimeAxisLabelConfig::default()
    );
    assert_eq!(
        config.price_axis_label_config,
        PriceAxisLabelConfig::default()
    );
    assert_eq!(
        config.time_axis_label_config.policy,
        TimeAxisLabelPolicy::UtcAdaptive
    );
}

#[test]
fn chart_engine_config_applies_axis_label_configs_on_init() {
    let time_config = TimeAxisLabelConfig {
        locale: AxisLabelLocale::EsEs,
        policy: TimeAxisLabelPolicy::UtcAdaptive,
        timezone: TimeAxisTimeZone::FixedOffsetMinutes { minutes: -180 },
        session: Some(TimeAxisSessionConfig {
            start_hour: 9,
            start_minute: 30,
            end_hour: 16,
            end_minute: 0,
        }),
    };
    let price_config = PriceAxisLabelConfig {
        locale: AxisLabelLocale::EsEs,
        policy: PriceAxisLabelPolicy::MinMove {
            min_move: 0.25,
            trim_trailing_zeros: true,
        },
        display_mode: PriceAxisDisplayMode::Percentage {
            base_price: Some(100.0),
        },
    };

    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_time_axis_label_config(time_config)
        .with_price_axis_label_config(price_config);
    let renderer = NullRenderer::default();
    let engine = ChartEngine::new(renderer, config).expect("engine");

    assert_eq!(engine.time_axis_label_config(), time_config);
    assert_eq!(engine.price_axis_label_config(), price_config);
}

#[test]
fn chart_engine_config_rejects_invalid_time_axis_label_config() {
    let invalid_time_config = TimeAxisLabelConfig {
        locale: AxisLabelLocale::EnUs,
        policy: TimeAxisLabelPolicy::UtcDateTime {
            show_seconds: false,
        },
        timezone: TimeAxisTimeZone::Utc,
        session: Some(TimeAxisSessionConfig {
            start_hour: 9,
            start_minute: 0,
            end_hour: 9,
            end_minute: 0,
        }),
    };

    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_time_axis_label_config(invalid_time_config);
    let renderer = NullRenderer::default();

    match ChartEngine::new(renderer, config) {
        Ok(_) => panic!("invalid time-axis config must fail"),
        Err(err) => assert!(matches!(err, chart_rs::ChartError::InvalidData(_))),
    }
}

#[test]
fn chart_engine_config_rejects_invalid_price_axis_label_config() {
    let invalid_price_config = PriceAxisLabelConfig {
        locale: AxisLabelLocale::EnUs,
        policy: PriceAxisLabelPolicy::MinMove {
            min_move: 0.0,
            trim_trailing_zeros: true,
        },
        display_mode: PriceAxisDisplayMode::Normal,
    };

    let config = ChartEngineConfig::new(Viewport::new(1000, 500), 0.0, 100.0)
        .with_price_domain(0.0, 1.0)
        .with_price_axis_label_config(invalid_price_config);
    let renderer = NullRenderer::default();

    match ChartEngine::new(renderer, config) {
        Ok(_) => panic!("invalid price-axis config must fail"),
        Err(err) => assert!(matches!(err, chart_rs::ChartError::InvalidData(_))),
    }
}

#[test]
fn chart_engine_config_json_without_axis_label_fields_uses_defaults() {
    let json = r#"{
  "viewport": { "width": 1000, "height": 500 },
  "time_start": 0.0,
  "time_end": 100.0,
  "price_min": 0.0,
  "price_max": 1.0
}"#;

    let config = ChartEngineConfig::from_json_str(json).expect("parse config");

    assert_eq!(
        config.time_axis_label_config,
        TimeAxisLabelConfig::default()
    );
    assert_eq!(
        config.price_axis_label_config,
        PriceAxisLabelConfig::default()
    );
}
