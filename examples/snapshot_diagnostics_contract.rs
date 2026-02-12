use chart_rs::api::ChartEngine;
use chart_rs::api::ChartEngineConfig;
use chart_rs::core::Viewport;
use chart_rs::render::NullRenderer;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let renderer = NullRenderer::default();
    let config =
        ChartEngineConfig::new(Viewport::new(900, 520), 0.0, 20.0).with_price_domain(10.0, 200.0);
    let mut engine = ChartEngine::new(renderer, config)?;

    engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, context| {
        format!("time={value:.1},span={:.1}", context.visible_span_abs)
    }));
    engine.set_crosshair_price_label_formatter(Arc::new(|value| format!("price={value:.3}")));

    engine.pointer_move(300.0, 240.0);
    let _ = engine.build_render_frame()?;

    let snapshot = engine.snapshot(7.0)?;
    let diagnostics = engine.crosshair_formatter_diagnostics();
    let snapshot_contract_json = engine.snapshot_json_contract_v1_pretty(7.0)?;
    let diagnostics_contract_json =
        engine.crosshair_formatter_diagnostics_json_contract_v1_pretty()?;

    println!(
        "snapshot formatter state: {:?}",
        snapshot.crosshair_formatter
    );
    println!("diagnostics formatter state: {:?}", diagnostics);
    println!("snapshot contract bytes: {}", snapshot_contract_json.len());
    println!(
        "diagnostics contract bytes: {}",
        diagnostics_contract_json.len()
    );

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
    let diagnostics_after_clear = engine.crosshair_formatter_diagnostics();
    println!(
        "diagnostics after cache clear: {:?}",
        diagnostics_after_clear
    );

    Ok(())
}
