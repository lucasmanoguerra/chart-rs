use crate::core::PaneId;
use crate::render::{CanvasLayerKind, LayeredRenderFrame};

pub(super) use super::render_partial_plan::PartialCairoRenderPlan;

use super::{
    InvalidationLevel, InvalidationMask, render_partial_lwc_policy_resolver,
    render_partial_plan_pane_targets_resolver::{
        resolve_api_or_all_targets, resolve_lwc_explicit_or_api_or_all_targets,
    },
    render_partial_plot_layers_resolver::{
        CURSOR_ONLY_PLOT_LAYERS, select_plot_layers_for_pending,
    },
};

impl PartialCairoRenderPlan {
    #[must_use]
    pub(super) fn build(
        pending: InvalidationMask,
        api_pane_targets: &[PaneId],
        layered: &LayeredRenderFrame,
    ) -> Option<Self> {
        if layered.panes.len() <= 1
            || !matches!(
                pending.level(),
                InvalidationLevel::Cursor | InvalidationLevel::Light
            )
        {
            return None;
        }

        let pane_targets = resolve_api_or_all_targets(api_pane_targets, layered);
        let plot_layers: &'static [CanvasLayerKind] = select_plot_layers_for_pending(pending);

        Some(Self::new(plot_layers, pane_targets))
    }

    #[must_use]
    pub(super) fn build_from_masks(
        pending: InvalidationMask,
        api_pane_targets: &[PaneId],
        lwc_pending: Option<&crate::lwc::model::InvalidateMask>,
        lwc_pane_ids: &[PaneId],
        layered: &LayeredRenderFrame,
    ) -> Option<Self> {
        if lwc_pending.is_some() {
            return Self::build_from_lwc(
                pending,
                api_pane_targets,
                lwc_pending,
                lwc_pane_ids,
                layered,
            );
        }
        Self::build(pending, api_pane_targets, layered)
    }

    fn build_from_lwc(
        api_pending: InvalidationMask,
        api_pane_targets: &[PaneId],
        lwc_pending: Option<&crate::lwc::model::InvalidateMask>,
        lwc_pane_ids: &[PaneId],
        layered: &LayeredRenderFrame,
    ) -> Option<Self> {
        let pending = lwc_pending?;
        let policy = render_partial_lwc_policy_resolver::resolve_lwc_partial_render_policy(
            api_pending,
            pending,
            lwc_pane_ids,
            layered,
        )?;
        let pane_targets = resolve_lwc_explicit_or_api_or_all_targets(
            policy.explicit_pane_targets,
            api_pane_targets,
            layered,
        );

        let plot_layers: &'static [CanvasLayerKind] = if policy.cursor_only {
            &CURSOR_ONLY_PLOT_LAYERS
        } else {
            select_plot_layers_for_pending(api_pending)
        };

        Some(Self::new(plot_layers, pane_targets))
    }
}

#[cfg(test)]
mod tests {
    use super::{CURSOR_ONLY_PLOT_LAYERS, PartialCairoRenderPlan};
    use crate::api::render_partial_test_support::layered_with_panes;
    use crate::api::{InvalidationLevel, InvalidationMask, InvalidationTopic, InvalidationTopics};
    use crate::core::PaneId;

    #[test]
    fn partial_plan_requires_multiple_panes_and_lightweight_invalidation_levels() {
        let single_pane = layered_with_panes(&[PaneId::new(0)]);
        let pending_cursor = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Cursor,
            InvalidationTopics::from_topic(InvalidationTopic::Cursor),
        );
        assert!(PartialCairoRenderPlan::build(pending_cursor, &[], &single_pane).is_none());

        let multi_pane = layered_with_panes(&[PaneId::new(0), PaneId::new(1)]);
        let pending_full = InvalidationMask::full();
        assert!(PartialCairoRenderPlan::build(pending_full, &[], &multi_pane).is_none());
    }

    #[test]
    fn partial_plan_prefers_lwc_cursor_invalidation_when_available() {
        let main = PaneId::new(0);
        let pane_a = PaneId::new(1);
        let multi_pane = layered_with_panes(&[main, pane_a]);
        let legacy_pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Full,
            InvalidationTopics::from_topic(InvalidationTopic::Series),
        );
        let lwc_pane_ids = vec![main, pane_a];

        let lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Cursor);

        let plan = PartialCairoRenderPlan::build_from_masks(
            legacy_pending,
            &[],
            Some(&lwc_pending),
            &lwc_pane_ids,
            &multi_pane,
        )
        .expect("plan");
        assert_eq!(plan.plot_layers(), &CURSOR_ONLY_PLOT_LAYERS);
    }

    #[test]
    fn partial_plan_uses_api_targets_when_lwc_explicit_panes_are_unknown() {
        let main = PaneId::new(0);
        let pane_a = PaneId::new(1);
        let pane_b = PaneId::new(2);
        let multi_pane = layered_with_panes(&[main, pane_a, pane_b]);
        let legacy_pending = InvalidationMask::light();
        let lwc_pane_ids = vec![main, pane_a];

        let mut lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Light);
        lwc_pending.invalidate_pane(
            99,
            crate::lwc::model::PaneInvalidation {
                level: crate::lwc::model::InvalidationLevel::Light,
                auto_scale: false,
            },
        );

        let plan = PartialCairoRenderPlan::build_from_masks(
            legacy_pending,
            &[pane_b],
            Some(&lwc_pending),
            &lwc_pane_ids,
            &multi_pane,
        )
        .expect("plan");
        assert!(!plan.targets_pane(main));
        assert!(!plan.targets_pane(pane_a));
        assert!(plan.targets_pane(pane_b));
    }
}
