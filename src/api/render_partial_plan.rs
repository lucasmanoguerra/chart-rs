use crate::core::PaneId;
use crate::render::CanvasLayerKind;

use super::render_partial_plan_pane_targets_resolver::PartialPaneTargets;

pub(super) struct PartialCairoRenderPlan {
    plot_layers: &'static [CanvasLayerKind],
    pane_targets: PartialPaneTargets,
}

impl PartialCairoRenderPlan {
    #[must_use]
    pub(super) fn new(
        plot_layers: &'static [CanvasLayerKind],
        pane_targets: PartialPaneTargets,
    ) -> Self {
        Self {
            plot_layers,
            pane_targets,
        }
    }

    #[must_use]
    pub(super) fn plot_layers(&self) -> &'static [CanvasLayerKind] {
        self.plot_layers
    }

    #[must_use]
    pub(super) fn targets_pane(&self, pane_id: PaneId) -> bool {
        self.pane_targets.contains(pane_id)
    }
}
