use chart_rs::api::{
    CROSSHAIR_DIAGNOSTICS_JSON_SCHEMA_V1, ChartEngine, ChartEngineConfig,
    CrosshairFormatterDiagnostics, CrosshairFormatterOverrideMode, ENGINE_SNAPSHOT_JSON_SCHEMA_V1,
    EngineSnapshot,
};
use chart_rs::core::{OhlcBar, Viewport};
use chart_rs::render::NullRenderer;
use serde_json::Value;
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

#[test]
fn crosshair_formatter_diagnostics_exposes_modes_generations_and_cache_stats() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(640, 480), 0.0, 5.0).with_price_domain(1.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    let initial = engine.crosshair_formatter_diagnostics();
    assert_eq!(
        initial.time_override_mode,
        CrosshairFormatterOverrideMode::None
    );
    assert_eq!(
        initial.price_override_mode,
        CrosshairFormatterOverrideMode::None
    );
    assert_eq!(initial.time_cache.size, 0);
    assert_eq!(initial.price_cache.size, 0);

    engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, _| {
        format!("TC:{value:.2}")
    }));
    engine.set_crosshair_price_label_formatter(Arc::new(|value| format!("PL:{value:.2}")));
    engine.pointer_move(100.0, 100.0);
    let _ = engine.build_render_frame().expect("build frame");

    let after_render = engine.crosshair_formatter_diagnostics();
    assert_eq!(
        after_render.time_override_mode,
        CrosshairFormatterOverrideMode::Context
    );
    assert_eq!(
        after_render.price_override_mode,
        CrosshairFormatterOverrideMode::Legacy
    );
    assert!(after_render.time_formatter_generation >= 1);
    assert!(after_render.price_formatter_generation >= 1);
    assert!(after_render.time_cache.misses >= 1);
    assert!(after_render.price_cache.misses >= 1);

    engine.clear_crosshair_formatter_caches();
    let cleared = engine.crosshair_formatter_diagnostics();
    assert_eq!(cleared.time_cache.size, 0);
    assert_eq!(cleared.price_cache.size, 0);
}

#[test]
fn snapshot_and_diagnostics_stay_coherent_across_formatter_lifecycle_transitions() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(640, 480), 0.0, 5.0).with_price_domain(1.0, 10.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_crosshair_time_label_formatter(Arc::new(|value| format!("TL:{value:.2}")));
    engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, _| {
        format!("TC:{value:.2}")
    }));
    engine.clear_crosshair_time_label_formatter_with_context();
    engine.set_crosshair_price_label_formatter(Arc::new(|value| format!("PL:{value:.2}")));
    engine.pointer_move(180.0, 120.0);
    let _ = engine.build_render_frame().expect("build frame");

    let snapshot = engine.snapshot(6.0).expect("snapshot should build");
    let diagnostics = engine.crosshair_formatter_diagnostics();
    assert_eq!(
        snapshot.crosshair_formatter.time_override_mode,
        diagnostics.time_override_mode
    );
    assert_eq!(
        snapshot.crosshair_formatter.price_override_mode,
        diagnostics.price_override_mode
    );
    assert_eq!(
        snapshot.crosshair_formatter.time_formatter_generation,
        diagnostics.time_formatter_generation
    );
    assert_eq!(
        snapshot.crosshair_formatter.price_formatter_generation,
        diagnostics.price_formatter_generation
    );

    engine.clear_crosshair_formatter_caches();
    let snapshot_after_clear = engine.snapshot(6.0).expect("snapshot should build");
    let diagnostics_after_clear = engine.crosshair_formatter_diagnostics();
    assert_eq!(diagnostics_after_clear.time_cache.size, 0);
    assert_eq!(diagnostics_after_clear.price_cache.size, 0);
    assert_eq!(
        snapshot_after_clear.crosshair_formatter.time_override_mode,
        diagnostics_after_clear.time_override_mode
    );
    assert_eq!(
        snapshot_after_clear.crosshair_formatter.price_override_mode,
        diagnostics_after_clear.price_override_mode
    );
    assert_eq!(
        snapshot_after_clear
            .crosshair_formatter
            .time_formatter_generation,
        diagnostics_after_clear.time_formatter_generation
    );
    assert_eq!(
        snapshot_after_clear
            .crosshair_formatter
            .price_formatter_generation,
        diagnostics_after_clear.price_formatter_generation
    );
}

#[test]
fn snapshot_json_crosshair_formatter_matches_diagnostics_after_mixed_mode_switches() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 10.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");

    engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, _| {
        format!("TC:{value:.2}")
    }));
    engine.set_crosshair_price_label_formatter_with_context(Arc::new(|value, _| {
        format!("PC:{value:.2}")
    }));
    engine.clear_crosshair_price_label_formatter_with_context();
    engine.set_crosshair_price_label_formatter(Arc::new(|value| format!("PL:{value:.2}")));
    engine.pointer_move(250.0, 150.0);
    let _ = engine.build_render_frame().expect("build frame");

    let json = engine
        .snapshot_json_pretty(8.0)
        .expect("snapshot should serialize");
    let decoded: EngineSnapshot =
        serde_json::from_str(&json).expect("snapshot json should deserialize");
    let diagnostics = engine.crosshair_formatter_diagnostics();

    assert_eq!(
        decoded.crosshair_formatter.time_override_mode,
        diagnostics.time_override_mode
    );
    assert_eq!(
        decoded.crosshair_formatter.price_override_mode,
        diagnostics.price_override_mode
    );
    assert_eq!(
        decoded.crosshair_formatter.time_formatter_generation,
        diagnostics.time_formatter_generation
    );
    assert_eq!(
        decoded.crosshair_formatter.price_formatter_generation,
        diagnostics.price_formatter_generation
    );
}

#[test]
fn snapshot_json_contract_v1_supports_backward_compatible_parse() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 10.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_crosshair_time_label_formatter(Arc::new(|value| format!("TL:{value:.2}")));

    let raw_snapshot_json = engine
        .snapshot_json_pretty(8.0)
        .expect("raw snapshot should serialize");
    let contract_json = engine
        .snapshot_json_contract_v1_pretty(8.0)
        .expect("contract snapshot should serialize");

    let from_raw = EngineSnapshot::from_json_compat_str(&raw_snapshot_json)
        .expect("compat parse should accept raw snapshot");
    let from_contract = EngineSnapshot::from_json_compat_str(&contract_json)
        .expect("compat parse should accept contract snapshot");
    assert_eq!(
        from_raw.crosshair_formatter,
        from_contract.crosshair_formatter
    );

    let payload: Value = serde_json::from_str(&contract_json).expect("contract json");
    assert_eq!(
        payload
            .get("schema_version")
            .and_then(Value::as_u64)
            .expect("schema_version should be u64"),
        u64::from(ENGINE_SNAPSHOT_JSON_SCHEMA_V1)
    );
    let snapshot = payload.get("snapshot").expect("snapshot payload");
    assert!(snapshot.get("crosshair_formatter").is_some());
}

#[test]
fn diagnostics_json_contract_v1_supports_backward_compatible_parse() {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 10.0).with_price_domain(0.0, 100.0);
    let mut engine = ChartEngine::new(renderer, config).expect("engine init");
    engine.set_crosshair_price_label_formatter_with_context(Arc::new(|value, _| {
        format!("PC:{value:.2}")
    }));
    engine.pointer_move(310.0, 140.0);
    let _ = engine.build_render_frame().expect("build frame");

    let raw_json = engine
        .crosshair_formatter_diagnostics_json_pretty()
        .expect("diagnostics json");
    let contract_json = engine
        .crosshair_formatter_diagnostics_json_contract_v1_pretty()
        .expect("diagnostics contract json");

    let from_raw = CrosshairFormatterDiagnostics::from_json_compat_str(&raw_json)
        .expect("compat parse should accept raw diagnostics");
    let from_contract = CrosshairFormatterDiagnostics::from_json_compat_str(&contract_json)
        .expect("compat parse should accept contract diagnostics");
    assert_eq!(from_raw, from_contract);

    let payload: Value = serde_json::from_str(&contract_json).expect("contract json");
    assert_eq!(
        payload
            .get("schema_version")
            .and_then(Value::as_u64)
            .expect("schema_version should be u64"),
        u64::from(CROSSHAIR_DIAGNOSTICS_JSON_SCHEMA_V1)
    );
    let diagnostics = payload.get("diagnostics").expect("diagnostics payload");
    assert!(diagnostics.get("time_override_mode").is_some());
    assert!(diagnostics.get("price_override_mode").is_some());
    assert!(diagnostics.get("time_formatter_generation").is_some());
    assert!(diagnostics.get("price_formatter_generation").is_some());
}
