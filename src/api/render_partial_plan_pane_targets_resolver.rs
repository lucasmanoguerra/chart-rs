use crate::core::PaneId;
use crate::render::LayeredRenderFrame;

use super::render_partial_pane_targets_resolver;

#[derive(Debug, Clone)]
pub(super) enum PartialPaneTargets {
    All,
    Some(Vec<PaneId>),
}

impl PartialPaneTargets {
    #[must_use]
    pub(super) fn contains(&self, pane_id: PaneId) -> bool {
        match self {
            Self::All => true,
            Self::Some(targets) => targets.contains(&pane_id),
        }
    }
}

#[must_use]
pub(super) fn resolve_api_or_all_targets(
    api_pane_targets: &[PaneId],
    layered: &LayeredRenderFrame,
) -> PartialPaneTargets {
    render_partial_pane_targets_resolver::resolve_api_pane_targets(api_pane_targets, layered)
        .map(PartialPaneTargets::Some)
        .unwrap_or(PartialPaneTargets::All)
}

#[must_use]
pub(super) fn resolve_lwc_explicit_or_api_or_all_targets(
    explicit_pane_targets: Option<Vec<PaneId>>,
    api_pane_targets: &[PaneId],
    layered: &LayeredRenderFrame,
) -> PartialPaneTargets {
    match explicit_pane_targets {
        Some(explicit_targets) if explicit_targets.is_empty() => {
            resolve_api_or_all_targets(api_pane_targets, layered)
        }
        Some(explicit_targets) => PartialPaneTargets::Some(explicit_targets),
        None => resolve_api_or_all_targets(api_pane_targets, layered),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PartialPaneTargets, resolve_api_or_all_targets, resolve_lwc_explicit_or_api_or_all_targets,
    };
    use crate::api::render_partial_test_support::layered_with_panes;
    use crate::core::PaneId;

    #[test]
    fn resolve_api_or_all_returns_all_when_api_targets_are_unknown() {
        let layered = layered_with_panes(&[PaneId::new(0), PaneId::new(1)]);
        let targets = resolve_api_or_all_targets(&[PaneId::new(99)], &layered);
        assert!(matches!(targets, PartialPaneTargets::All));
    }

    #[test]
    fn resolve_api_or_all_returns_some_for_known_api_targets() {
        let pane = PaneId::new(1);
        let layered = layered_with_panes(&[PaneId::new(0), pane]);
        let targets = resolve_api_or_all_targets(&[pane], &layered);
        assert!(matches!(targets, PartialPaneTargets::Some(values) if values == vec![pane]));
    }

    #[test]
    fn resolve_lwc_explicit_or_api_or_all_prioritizes_non_empty_explicit_targets() {
        let pane = PaneId::new(2);
        let layered = layered_with_panes(&[PaneId::new(0), pane]);
        let targets = resolve_lwc_explicit_or_api_or_all_targets(Some(vec![pane]), &[], &layered);
        assert!(matches!(targets, PartialPaneTargets::Some(values) if values == vec![pane]));
    }

    #[test]
    fn resolve_lwc_explicit_or_api_or_all_falls_back_to_api_when_explicit_is_empty() {
        let pane = PaneId::new(1);
        let layered = layered_with_panes(&[PaneId::new(0), pane]);
        let targets =
            resolve_lwc_explicit_or_api_or_all_targets(Some(Vec::new()), &[pane], &layered);
        assert!(matches!(targets, PartialPaneTargets::Some(values) if values == vec![pane]));
    }

    #[test]
    fn resolve_lwc_explicit_or_api_or_all_falls_back_to_all_without_explicit_or_api() {
        let layered = layered_with_panes(&[PaneId::new(0), PaneId::new(1)]);
        let targets = resolve_lwc_explicit_or_api_or_all_targets(None, &[], &layered);
        assert!(matches!(targets, PartialPaneTargets::All));
    }
}
