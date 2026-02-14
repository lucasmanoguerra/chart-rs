use crate::core::{PaneId, Viewport};
use crate::render::{LayeredRenderFrame, PaneLayerStack};

#[must_use]
pub(crate) fn layered_with_panes(panes: &[PaneId]) -> LayeredRenderFrame {
    let stacks = panes
        .iter()
        .copied()
        .map(PaneLayerStack::canonical_for_pane)
        .collect::<Vec<_>>();
    LayeredRenderFrame::from_stacks(Viewport::new(800, 500), stacks)
}
