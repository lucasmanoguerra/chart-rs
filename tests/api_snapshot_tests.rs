use chart_rs::api::{
    ChartEngine, ChartEngineConfig, CrosshairFormatterOverrideMode, EngineSnapshot,
};
use chart_rs::core::{OhlcBar, Viewport};
use chart_rs::render::NullRenderer;
use std::sync::Arc;

#[test]
fn chart_engine_config_json_roundtrip() {
    let config = ChartEngineConfig::new(Viewport::new(1024, 768), 100.0, 200.0)
        .with_price_domain(10.5, 88.25);

    let json = config
        .to_json_pretty()
        .expect("config should serialize to json");
    let restored = ChartEngineConfig::from_json_str(&json).expect("config should deserialize");

    assert_eq!(restored, config);
}

#[test]
fn snapshot_preserves_metadata_order_and_geometry() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 600), 0.0, 10.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_series_metadata("id", "candles-main");
    engine.set_series_metadata("style", "candlestick");
    engine.set_candles(vec![
        OhlcBar::new(1.0, 20.0, 25.0, 19.0, 24.0).expect("valid candle"),
        OhlcBar::new(2.0, 24.0, 28.0, 22.0, 23.0).expect("valid candle"),
    ]);

    let snapshot = engine.snapshot(8.0).expect("snapshot should build");
    let keys: Vec<&str> = snapshot
        .series_metadata
        .keys()
        .map(std::string::String::as_str)
        .collect();

    assert_eq!(keys, vec!["id", "style"]);
    assert_eq!(snapshot.candle_geometry.len(), 2);
}

#[test]
fn snapshot_json_roundtrip() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(640, 480), 0.0, 5.0).with_price_domain(1.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_series_metadata("symbol", "BTCUSD");
    engine.set_candles(vec![
        OhlcBar::new(1.0, 3.0, 5.0, 2.0, 4.0).expect("valid candle"),
    ]);

    let json = engine
        .snapshot_json_pretty(6.0)
        .expect("snapshot should serialize");
    let decoded: EngineSnapshot =
        serde_json::from_str(&json).expect("snapshot json should deserialize");

    assert_eq!(decoded.candle_geometry.len(), 1);
    assert_eq!(
        decoded.series_metadata.get("symbol").map(String::as_str),
        Some("BTCUSD")
    );
}

#[test]
fn snapshot_exports_crosshair_formatter_state() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(640, 480), 0.0, 5.0).with_price_domain(1.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_crosshair_time_label_formatter(Arc::new(|value| format!("T:{value:.2}")));
    engine.set_crosshair_price_label_formatter_with_context(Arc::new(|value, _| {
        format!("P:{value:.2}")
    }));

    let snapshot = engine.snapshot(6.0).expect("snapshot should build");
    assert_eq!(
        snapshot.crosshair_formatter.time_override_mode,
        CrosshairFormatterOverrideMode::Legacy
    );
    assert_eq!(
        snapshot.crosshair_formatter.price_override_mode,
        CrosshairFormatterOverrideMode::Context
    );
    assert!(snapshot.crosshair_formatter.time_formatter_generation >= 1);
    assert!(snapshot.crosshair_formatter.price_formatter_generation >= 1);
}

#[test]
fn crosshair_formatter_override_mode_accessors_follow_contract() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(640, 480), 0.0, 5.0).with_price_domain(1.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    assert_eq!(
        engine.crosshair_time_label_formatter_override_mode(),
        CrosshairFormatterOverrideMode::None
    );
    assert_eq!(
        engine.crosshair_price_label_formatter_override_mode(),
        CrosshairFormatterOverrideMode::None
    );
    let generations_before = engine.crosshair_label_formatter_generations();

    engine.set_crosshair_time_label_formatter(Arc::new(|value| format!("T:{value:.2}")));
    assert_eq!(
        engine.crosshair_time_label_formatter_override_mode(),
        CrosshairFormatterOverrideMode::Legacy
    );
    engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, _| {
        format!("TC:{value:.2}")
    }));
    assert_eq!(
        engine.crosshair_time_label_formatter_override_mode(),
        CrosshairFormatterOverrideMode::Context
    );

    engine.set_crosshair_price_label_formatter_with_context(Arc::new(|value, _| {
        format!("PC:{value:.2}")
    }));
    assert_eq!(
        engine.crosshair_price_label_formatter_override_mode(),
        CrosshairFormatterOverrideMode::Context
    );
    engine.set_crosshair_price_label_formatter(Arc::new(|value| format!("P:{value:.2}")));
    assert_eq!(
        engine.crosshair_price_label_formatter_override_mode(),
        CrosshairFormatterOverrideMode::Legacy
    );

    let generations_after = engine.crosshair_label_formatter_generations();
    assert!(generations_after.0 > generations_before.0);
    assert!(generations_after.1 > generations_before.1);
}
