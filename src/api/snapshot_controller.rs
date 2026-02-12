use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::{
    ChartEngine, CrosshairFormatterOverrideMode, CrosshairFormatterSnapshot, EngineSnapshot,
};

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
            crosshair_formatter: CrosshairFormatterSnapshot {
                time_override_mode: if self.crosshair_time_label_formatter_with_context.is_some() {
                    CrosshairFormatterOverrideMode::Context
                } else if self.crosshair_time_label_formatter.is_some() {
                    CrosshairFormatterOverrideMode::Legacy
                } else {
                    CrosshairFormatterOverrideMode::None
                },
                price_override_mode: if self.crosshair_price_label_formatter_with_context.is_some()
                {
                    CrosshairFormatterOverrideMode::Context
                } else if self.crosshair_price_label_formatter.is_some() {
                    CrosshairFormatterOverrideMode::Legacy
                } else {
                    CrosshairFormatterOverrideMode::None
                },
                time_formatter_generation: self.crosshair_time_label_formatter_generation,
                price_formatter_generation: self.crosshair_price_label_formatter_generation,
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
