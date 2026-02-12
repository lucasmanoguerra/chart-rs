use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::{ChartEngine, CrosshairFormatterSnapshot, EngineSnapshot};

impl<R: Renderer> ChartEngine<R> {
    /// Builds a deterministic snapshot useful for regression tests.
    pub fn snapshot(&self, body_width_px: f64) -> ChartResult<EngineSnapshot> {
        Ok(EngineSnapshot {
            viewport: self.viewport,
            time_full_range: self.time_scale.full_range(),
            time_visible_range: self.time_scale.visible_range(),
            price_domain: self.price_scale.domain(),
            crosshair: self.interaction.crosshair(),
            points: self.points.clone(),
            candle_geometry: self.project_candles(body_width_px)?,
            series_metadata: self.series_metadata.clone(),
            crosshair_formatter: {
                let (time_gen, price_gen) = self.crosshair_label_formatter_generations();
                CrosshairFormatterSnapshot {
                    time_override_mode: self.crosshair_time_label_formatter_override_mode(),
                    price_override_mode: self.crosshair_price_label_formatter_override_mode(),
                    time_formatter_generation: time_gen,
                    price_formatter_generation: price_gen,
                }
            },
        })
    }

    /// Serializes snapshot as pretty JSON for fixture-based regression checks.
    pub fn snapshot_json_pretty(&self, body_width_px: f64) -> ChartResult<String> {
        let snapshot = self.snapshot(body_width_px)?;
        serde_json::to_string_pretty(&snapshot)
            .map_err(|e| ChartError::InvalidData(format!("failed to serialize snapshot: {e}")))
    }
}
