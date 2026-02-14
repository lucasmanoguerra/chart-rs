use crate::core::PaneLayoutRegion;
use crate::render::{LayeredRenderFrame, Renderer};

use super::ChartEngine;

#[derive(Debug, Clone, Copy)]
pub(super) struct PaneSceneContext {
    pub plot_top: f64,
    pub plot_bottom: f64,
}

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub(super) fn resolve_pane_scene_regions(
        &self,
        ctx: PaneSceneContext,
    ) -> Vec<PaneLayoutRegion> {
        self.pane_layout_regions(ctx.plot_top, ctx.plot_bottom)
    }

    #[must_use]
    pub(super) fn apply_pane_scene_regions(
        &self,
        layered: LayeredRenderFrame,
        regions: &[PaneLayoutRegion],
    ) -> LayeredRenderFrame {
        let pane_region_tuples: Vec<_> = regions
            .iter()
            .map(|region| (region.pane_id, region.plot_top, region.plot_bottom))
            .collect();
        layered.with_pane_regions(&pane_region_tuples)
    }

    pub(super) fn remap_plot_layers_into_pane_regions(
        &self,
        layered: &mut LayeredRenderFrame,
        regions: &[PaneLayoutRegion],
        source_plot_top: f64,
        source_plot_bottom: f64,
    ) {
        if source_plot_bottom <= source_plot_top {
            return;
        }
        for pane in regions {
            layered.remap_plot_layers_to_pane_region(
                pane.pane_id,
                source_plot_top,
                source_plot_bottom,
            );
        }
    }
}
