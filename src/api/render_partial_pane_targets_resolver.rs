use crate::core::PaneId;
use crate::render::LayeredRenderFrame;

#[must_use]
pub(super) fn resolve_api_pane_targets(
    api_pane_targets: &[PaneId],
    layered: &LayeredRenderFrame,
) -> Option<Vec<PaneId>> {
    if api_pane_targets.is_empty() {
        return None;
    }
    let targets = collect_known_pane_targets(api_pane_targets.iter().copied(), layered);
    if targets.is_empty() {
        None
    } else {
        Some(targets)
    }
}

#[must_use]
pub(super) fn collect_known_pane_targets(
    pane_ids: impl Iterator<Item = PaneId>,
    layered: &LayeredRenderFrame,
) -> Vec<PaneId> {
    let mut targets = pane_ids
        .filter(|pane_id| layered.panes.iter().any(|pane| pane.pane_id == *pane_id))
        .collect::<Vec<_>>();
    targets.sort_by_key(|pane_id| pane_id.raw());
    targets.dedup();
    targets
}
