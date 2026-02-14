use crate::core::{PaneDescriptor, PaneId, PaneLayoutRegion};
use crate::error::ChartResult;
use crate::render::{PaneLayerStack, Renderer};

use super::ChartEngine;
use super::layout_helpers::resolve_axis_layout;

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn panes(&self) -> &[PaneDescriptor] {
        self.core.model.pane_collection.panes()
    }

    #[must_use]
    pub fn main_pane_id(&self) -> PaneId {
        self.core.model.pane_collection.main_pane_id()
    }

    #[must_use]
    pub fn points_pane_id(&self) -> PaneId {
        self.core.model.points_pane_id
    }

    #[must_use]
    pub fn candles_pane_id(&self) -> PaneId {
        self.core.model.candles_pane_id
    }

    pub fn set_points_pane(&mut self, pane_id: PaneId) -> ChartResult<()> {
        if !self.core.model.pane_collection.contains(pane_id) {
            return Err(crate::error::ChartError::InvalidData(
                "points pane does not exist".to_owned(),
            ));
        }
        if self.core.model.points_pane_id != pane_id {
            self.core.model.points_pane_id = pane_id;
            self.invalidate_pane_content(pane_id);
        }
        Ok(())
    }

    pub fn set_candles_pane(&mut self, pane_id: PaneId) -> ChartResult<()> {
        if !self.core.model.pane_collection.contains(pane_id) {
            return Err(crate::error::ChartError::InvalidData(
                "candles pane does not exist".to_owned(),
            ));
        }
        if self.core.model.candles_pane_id != pane_id {
            self.core.model.candles_pane_id = pane_id;
            self.invalidate_pane_content(pane_id);
        }
        Ok(())
    }

    pub fn create_pane(&mut self, stretch_factor: f64) -> ChartResult<PaneId> {
        let pane_id = self
            .core
            .model
            .pane_collection
            .create_pane(stretch_factor)?;
        self.invalidate_pane_layout();
        Ok(pane_id)
    }

    pub fn remove_pane(&mut self, pane_id: PaneId) -> ChartResult<bool> {
        let removed = self.core.model.pane_collection.remove_pane(pane_id)?;
        if removed {
            let main_pane_id = self.main_pane_id();
            if self.core.model.points_pane_id == pane_id {
                self.core.model.points_pane_id = main_pane_id;
            }
            if self.core.model.candles_pane_id == pane_id {
                self.core.model.candles_pane_id = main_pane_id;
            }
            self.invalidate_pane_layout();
        }
        Ok(removed)
    }

    pub fn set_pane_stretch_factor(
        &mut self,
        pane_id: PaneId,
        stretch_factor: f64,
    ) -> ChartResult<bool> {
        let changed = self
            .core
            .model
            .pane_collection
            .set_stretch_factor(pane_id, stretch_factor)?;
        if changed {
            self.core.model.pane_collection.normalize_stretch_factors();
            self.invalidate_pane_layout();
        }
        Ok(changed)
    }

    #[must_use]
    pub fn pane_layer_stack(&self, pane_id: PaneId) -> Option<PaneLayerStack> {
        if !self.core.model.pane_collection.contains(pane_id) {
            return None;
        }
        Some(PaneLayerStack::canonical_for_pane(pane_id))
    }

    #[must_use]
    pub fn pane_layer_stacks(&self) -> Vec<PaneLayerStack> {
        self.core
            .model
            .pane_collection
            .panes()
            .iter()
            .map(|pane| PaneLayerStack::canonical_for_pane(pane.id))
            .collect()
    }

    #[must_use]
    pub fn pane_layout_regions(&self, plot_top: f64, plot_bottom: f64) -> Vec<PaneLayoutRegion> {
        self.core
            .model
            .pane_collection
            .layout_regions(plot_top, plot_bottom)
    }

    #[must_use]
    pub fn pane_plot_regions_for_current_viewport(&self) -> Vec<PaneLayoutRegion> {
        let viewport_width = f64::from(self.core.model.viewport.width);
        let viewport_height = f64::from(self.core.model.viewport.height);
        let layout = resolve_axis_layout(
            viewport_width,
            viewport_height,
            self.core.presentation.render_style.price_axis_width_px,
            self.core.presentation.render_style.time_axis_height_px,
        );
        self.pane_layout_regions(0.0, layout.plot_bottom)
    }
}
