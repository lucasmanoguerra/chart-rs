use crate::core::PaneId;
use crate::render::LayeredRenderFrame;

use super::InvalidationMask;
use super::render_partial_pane_targets_resolver;
use super::render_partial_plot_layers_resolver::pending_has_time_scale_topic;

pub(super) struct LwcPartialRenderPolicy {
    pub(super) explicit_pane_targets: Option<Vec<PaneId>>,
    pub(super) cursor_only: bool,
}

#[must_use]
pub(super) fn resolve_lwc_partial_render_policy(
    api_pending: InvalidationMask,
    lwc_pending: &crate::lwc::model::InvalidateMask,
    lwc_pane_ids: &[PaneId],
    layered: &LayeredRenderFrame,
) -> Option<LwcPartialRenderPolicy> {
    if layered.panes.len() <= 1 {
        return None;
    }
    // Time-scale topic requests must remain full redraw unless the LWC model
    // explicitly reports a lightweight/cursor mutation without time-scale effects.
    if pending_has_time_scale_topic(api_pending) {
        return None;
    }

    let level = lwc_pending.full_invalidation();
    if !matches!(
        level,
        crate::lwc::model::InvalidationLevel::Cursor | crate::lwc::model::InvalidationLevel::Light
    ) {
        return None;
    }
    if !lwc_pending.time_scale_invalidations().is_empty() {
        return None;
    }

    let explicit = lwc_pending.explicit_pane_invalidations();
    if explicit
        .iter()
        .any(|(_, invalidation)| invalidation.auto_scale)
    {
        return None;
    }

    let explicit_pane_targets = if explicit.is_empty() {
        None
    } else {
        Some(
            render_partial_pane_targets_resolver::collect_known_pane_targets(
                explicit
                    .iter()
                    .filter_map(|(index, _)| lwc_pane_ids.get(*index).copied()),
                layered,
            ),
        )
    };

    Some(LwcPartialRenderPolicy {
        explicit_pane_targets,
        cursor_only: is_cursor_only_lwc_invalidation(lwc_pending),
    })
}

fn is_cursor_only_lwc_invalidation(pending: &crate::lwc::model::InvalidateMask) -> bool {
    if pending.full_invalidation() != crate::lwc::model::InvalidationLevel::Cursor {
        return false;
    }
    if !pending.time_scale_invalidations().is_empty() {
        return false;
    }

    !pending
        .explicit_pane_invalidations()
        .iter()
        .any(|(_, invalidation)| {
            invalidation.auto_scale
                || invalidation.level != crate::lwc::model::InvalidationLevel::Cursor
        })
}

#[cfg(test)]
mod tests {
    use super::resolve_lwc_partial_render_policy;
    use crate::api::render_partial_test_support::layered_with_panes;
    use crate::api::{InvalidationLevel, InvalidationMask, InvalidationTopic, InvalidationTopics};
    use crate::core::PaneId;

    #[test]
    fn policy_rejects_single_pane_layered_frames() {
        let layered = layered_with_panes(&[PaneId::new(0)]);
        let api_pending = InvalidationMask::light();
        let lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Cursor);
        let lwc_pane_ids = vec![PaneId::new(0)];

        let policy =
            resolve_lwc_partial_render_policy(api_pending, &lwc_pending, &lwc_pane_ids, &layered);
        assert!(policy.is_none());
    }

    #[test]
    fn policy_rejects_api_time_scale_topic_requests() {
        let layered = layered_with_panes(&[PaneId::new(0), PaneId::new(1)]);
        let api_pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::TimeScale),
        );
        let lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Cursor);
        let lwc_pane_ids = vec![PaneId::new(0), PaneId::new(1)];

        let policy =
            resolve_lwc_partial_render_policy(api_pending, &lwc_pending, &lwc_pane_ids, &layered);
        assert!(policy.is_none());
    }

    #[test]
    fn policy_rejects_time_scale_mutations_inside_lwc_pending() {
        let layered = layered_with_panes(&[PaneId::new(0), PaneId::new(1)]);
        let api_pending = InvalidationMask::light();
        let mut lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Cursor);
        lwc_pending.set_bar_spacing(8.0);
        let lwc_pane_ids = vec![PaneId::new(0), PaneId::new(1)];

        let policy =
            resolve_lwc_partial_render_policy(api_pending, &lwc_pending, &lwc_pane_ids, &layered);
        assert!(policy.is_none());
    }

    #[test]
    fn policy_rejects_lwc_autoscale_pane_invalidations() {
        let layered = layered_with_panes(&[PaneId::new(0), PaneId::new(1)]);
        let api_pending = InvalidationMask::light();
        let mut lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Light);
        lwc_pending.invalidate_pane(
            1,
            crate::lwc::model::PaneInvalidation {
                level: crate::lwc::model::InvalidationLevel::Light,
                auto_scale: true,
            },
        );
        let lwc_pane_ids = vec![PaneId::new(0), PaneId::new(1)];

        let policy =
            resolve_lwc_partial_render_policy(api_pending, &lwc_pending, &lwc_pane_ids, &layered);
        assert!(policy.is_none());
    }

    #[test]
    fn policy_detects_cursor_only_lwc_invalidations() {
        let layered = layered_with_panes(&[PaneId::new(0), PaneId::new(1)]);
        let api_pending = InvalidationMask::light();
        let lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Cursor);
        let lwc_pane_ids = vec![PaneId::new(0), PaneId::new(1)];

        let policy =
            resolve_lwc_partial_render_policy(api_pending, &lwc_pending, &lwc_pane_ids, &layered)
                .expect("policy");
        assert!(policy.cursor_only);
        assert!(policy.explicit_pane_targets.is_none());
    }

    #[test]
    fn policy_collects_explicit_known_pane_targets() {
        let main = PaneId::new(0);
        let pane = PaneId::new(1);
        let layered = layered_with_panes(&[main, pane]);
        let api_pending = InvalidationMask::light();
        let mut lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Light);
        lwc_pending.invalidate_pane(
            1,
            crate::lwc::model::PaneInvalidation {
                level: crate::lwc::model::InvalidationLevel::Light,
                auto_scale: false,
            },
        );
        let lwc_pane_ids = vec![main, pane];

        let policy =
            resolve_lwc_partial_render_policy(api_pending, &lwc_pending, &lwc_pane_ids, &layered)
                .expect("policy");

        assert_eq!(policy.explicit_pane_targets, Some(vec![pane]));
        assert!(!policy.cursor_only);
    }
}
