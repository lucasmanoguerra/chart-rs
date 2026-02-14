use indexmap::IndexMap;

use crate::core::{DataPoint, OhlcBar, Viewport};
use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::ChartEngine;

impl<R: Renderer> ChartEngine<R> {
    /// Sets or updates deterministic series metadata.
    ///
    /// `IndexMap` is used to preserve insertion order for stable snapshots.
    pub fn set_series_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.core
            .model
            .series_metadata
            .insert(key.into(), value.into());
    }

    #[must_use]
    pub fn series_metadata(&self) -> &IndexMap<String, String> {
        &self.core.model.series_metadata
    }

    #[must_use]
    pub fn points(&self) -> &[DataPoint] {
        &self.core.model.points
    }

    #[must_use]
    pub fn candles(&self) -> &[OhlcBar] {
        &self.core.model.candles
    }

    #[must_use]
    pub fn viewport(&self) -> Viewport {
        self.core.model.viewport
    }

    /// Updates viewport dimensions used by scale mapping and render layout.
    pub fn set_viewport(&mut self, viewport: Viewport) -> ChartResult<()> {
        if !viewport.is_valid() {
            return Err(ChartError::InvalidViewport {
                width: viewport.width,
                height: viewport.height,
            });
        }
        let previous_width = self.core.model.viewport.width;
        self.core.model.viewport = viewport;
        self.invalidate_full();

        let mut changed = self.apply_time_scale_resize_behavior(previous_width)?;
        changed |= self.apply_time_scale_zoom_limit_behavior()?;
        changed |= self.apply_time_scale_edge_behavior()?;
        if changed {
            self.emit_visible_range_changed();
        }
        Ok(())
    }
}
