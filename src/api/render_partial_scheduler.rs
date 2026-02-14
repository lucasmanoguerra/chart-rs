use crate::core::PaneId;
use crate::render::{CanvasLayerKind, LayeredRenderFrame};

use super::{InvalidationLevel, InvalidationMask, InvalidationTopic};

const CURSOR_ONLY_PLOT_LAYERS: [CanvasLayerKind; 3] = [
    CanvasLayerKind::Background,
    CanvasLayerKind::Overlay,
    CanvasLayerKind::Crosshair,
];

const LIGHT_PLOT_LAYERS: [CanvasLayerKind; 5] = [
    CanvasLayerKind::Background,
    CanvasLayerKind::Grid,
    CanvasLayerKind::Series,
    CanvasLayerKind::Overlay,
    CanvasLayerKind::Crosshair,
];

pub(super) struct PartialCairoRenderPlan {
    plot_layers: &'static [CanvasLayerKind],
    pane_targets: PaneTargets,
}

#[derive(Debug, Clone)]
enum PaneTargets {
    All,
    Some(Vec<PaneId>),
}

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

        let pane_targets =
            resolve_api_pane_targets(api_pane_targets, layered).unwrap_or(PaneTargets::All);
        let plot_layers: &'static [CanvasLayerKind] = if Self::is_cursor_only_invalidation(pending)
        {
            &CURSOR_ONLY_PLOT_LAYERS
        } else {
            &LIGHT_PLOT_LAYERS
        };

        Some(Self {
            plot_layers,
            pane_targets,
        })
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

    #[must_use]
    pub(super) fn plot_layers(&self) -> &'static [CanvasLayerKind] {
        self.plot_layers
    }

    #[must_use]
    pub(super) fn targets_pane(&self, pane_id: PaneId) -> bool {
        self.pane_targets.contains(pane_id)
    }

    fn is_cursor_only_invalidation(pending: InvalidationMask) -> bool {
        pending.level() == InvalidationLevel::Cursor
    }

    fn build_from_lwc(
        api_pending: InvalidationMask,
        api_pane_targets: &[PaneId],
        lwc_pending: Option<&crate::lwc::model::InvalidateMask>,
        lwc_pane_ids: &[PaneId],
        layered: &LayeredRenderFrame,
    ) -> Option<Self> {
        let pending = lwc_pending?;
        if layered.panes.len() <= 1 {
            return None;
        }
        // Time-scale topic requests must remain full redraw unless the LWC model
        // explicitly reports a lightweight/cursor mutation without time-scale effects.
        if pending_has_time_scale_topic(api_pending) {
            return None;
        }

        let level = pending.full_invalidation();
        if !matches!(
            level,
            crate::lwc::model::InvalidationLevel::Cursor
                | crate::lwc::model::InvalidationLevel::Light
        ) {
            return None;
        }
        if !pending.time_scale_invalidations().is_empty() {
            return None;
        }

        let explicit = pending.explicit_pane_invalidations();
        if explicit
            .iter()
            .any(|(_, invalidation)| invalidation.auto_scale)
        {
            return None;
        }
        let pane_targets = if explicit.is_empty() {
            resolve_api_pane_targets(api_pane_targets, layered).unwrap_or(PaneTargets::All)
        } else {
            let mut targets = explicit
                .iter()
                .filter_map(|(index, _)| lwc_pane_ids.get(*index).copied())
                .filter(|pane_id| layered.panes.iter().any(|pane| pane.pane_id == *pane_id))
                .collect::<Vec<_>>();
            targets.sort_by_key(|pane_id| pane_id.raw());
            targets.dedup();
            if targets.is_empty() {
                return None;
            }
            PaneTargets::Some(targets)
        };

        let plot_layers: &'static [CanvasLayerKind] =
            if Self::is_cursor_only_lwc_invalidation(pending) {
                &CURSOR_ONLY_PLOT_LAYERS
            } else {
                &LIGHT_PLOT_LAYERS
            };

        Some(Self {
            plot_layers,
            pane_targets,
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
}

fn pending_has_time_scale_topic(pending: InvalidationMask) -> bool {
    pending.has_topic(InvalidationTopic::TimeScale)
}

fn resolve_api_pane_targets(
    api_pane_targets: &[PaneId],
    layered: &LayeredRenderFrame,
) -> Option<PaneTargets> {
    if !api_pane_targets.is_empty() {
        let mut targets = api_pane_targets
            .iter()
            .copied()
            .filter(|pane_id| layered.panes.iter().any(|pane| pane.pane_id == *pane_id))
            .collect::<Vec<_>>();
        targets.sort_by_key(|pane_id| pane_id.raw());
        targets.dedup();
        return if targets.is_empty() {
            None
        } else {
            Some(PaneTargets::Some(targets))
        };
    }

    None
}

impl PaneTargets {
    fn contains(&self, pane_id: PaneId) -> bool {
        match self {
            Self::All => true,
            Self::Some(targets) => targets.contains(&pane_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CURSOR_ONLY_PLOT_LAYERS, LIGHT_PLOT_LAYERS, PartialCairoRenderPlan};
    use crate::api::{InvalidationLevel, InvalidationMask, InvalidationTopic, InvalidationTopics};
    use crate::core::{PaneId, Viewport};
    use crate::render::{LayeredRenderFrame, PaneLayerStack};

    fn layered_with_panes(panes: &[PaneId]) -> LayeredRenderFrame {
        let stacks = panes
            .iter()
            .copied()
            .map(PaneLayerStack::canonical_for_pane)
            .collect::<Vec<_>>();
        LayeredRenderFrame::from_stacks(Viewport::new(800, 500), stacks)
    }

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
    fn partial_plan_uses_cursor_only_plot_layers_for_pure_cursor_invalidations() {
        let multi_pane = layered_with_panes(&[PaneId::new(0), PaneId::new(1)]);
        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Cursor,
            InvalidationTopics::from_topic(InvalidationTopic::Cursor),
        );
        let plan = PartialCairoRenderPlan::build(pending, &[], &multi_pane).expect("plan");
        assert_eq!(plan.plot_layers(), &CURSOR_ONLY_PLOT_LAYERS);
    }

    #[test]
    fn partial_plan_uses_full_plot_layers_for_non_cursor_light_invalidations() {
        let multi_pane = layered_with_panes(&[PaneId::new(0), PaneId::new(1)]);
        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::Series),
        );
        let plan = PartialCairoRenderPlan::build(pending, &[], &multi_pane).expect("plan");
        assert_eq!(plan.plot_layers(), &LIGHT_PLOT_LAYERS);
    }

    #[test]
    fn partial_plan_uses_api_targets_and_ignores_unknown_targets() {
        let main = PaneId::new(0);
        let aux = PaneId::new(3);
        let multi_pane = layered_with_panes(&[main, aux]);
        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::Series),
        );

        let invalid_hint_plan =
            PartialCairoRenderPlan::build(pending, &[PaneId::new(99)], &multi_pane).expect("plan");
        assert!(invalid_hint_plan.targets_pane(main));
        assert!(invalid_hint_plan.targets_pane(aux));

        let aux_only_plan =
            PartialCairoRenderPlan::build(pending, &[aux], &multi_pane).expect("plan");
        assert!(!aux_only_plan.targets_pane(main));
        assert!(aux_only_plan.targets_pane(aux));
    }

    #[test]
    fn partial_plan_prefers_lwc_cursor_invalidation_when_available() {
        let main = PaneId::new(0);
        let aux = PaneId::new(1);
        let multi_pane = layered_with_panes(&[main, aux]);
        let legacy_pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Full,
            InvalidationTopics::from_topic(InvalidationTopic::Series),
        );
        let lwc_pane_ids = vec![main, aux];

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
    fn partial_plan_uses_single_lwc_pane_invalidation_as_target() {
        let main = PaneId::new(0);
        let aux = PaneId::new(1);
        let multi_pane = layered_with_panes(&[main, aux]);
        let legacy_pending = InvalidationMask::light();
        let lwc_pane_ids = vec![main, aux];

        let mut lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Cursor);
        lwc_pending.invalidate_pane(
            1,
            crate::lwc::model::PaneInvalidation {
                level: crate::lwc::model::InvalidationLevel::Light,
                auto_scale: false,
            },
        );

        let plan = PartialCairoRenderPlan::build_from_masks(
            legacy_pending,
            &[],
            Some(&lwc_pending),
            &lwc_pane_ids,
            &multi_pane,
        )
        .expect("plan");
        assert!(!plan.targets_pane(main));
        assert!(plan.targets_pane(aux));
    }

    #[test]
    fn partial_plan_uses_multiple_lwc_pane_invalidations_as_targets() {
        let main = PaneId::new(0);
        let pane_a = PaneId::new(1);
        let pane_b = PaneId::new(2);
        let multi_pane = layered_with_panes(&[main, pane_a, pane_b]);
        let legacy_pending = InvalidationMask::light();
        let lwc_pane_ids = vec![main, pane_a, pane_b];

        let mut lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Light);
        lwc_pending.invalidate_pane(
            1,
            crate::lwc::model::PaneInvalidation {
                level: crate::lwc::model::InvalidationLevel::Light,
                auto_scale: false,
            },
        );
        lwc_pending.invalidate_pane(
            2,
            crate::lwc::model::PaneInvalidation {
                level: crate::lwc::model::InvalidationLevel::Light,
                auto_scale: false,
            },
        );

        let plan = PartialCairoRenderPlan::build_from_masks(
            legacy_pending,
            &[],
            Some(&lwc_pending),
            &lwc_pane_ids,
            &multi_pane,
        )
        .expect("plan");
        assert!(!plan.targets_pane(main));
        assert!(plan.targets_pane(pane_a));
        assert!(plan.targets_pane(pane_b));
    }

    #[test]
    fn partial_plan_falls_back_to_full_for_lwc_cursor_with_time_scale_mutation() {
        let main = PaneId::new(0);
        let aux = PaneId::new(1);
        let multi_pane = layered_with_panes(&[main, aux]);
        let legacy_pending = InvalidationMask::cursor();
        let lwc_pane_ids = vec![main, aux];

        let mut lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Cursor);
        lwc_pending.set_bar_spacing(8.0);

        let plan = PartialCairoRenderPlan::build_from_masks(
            legacy_pending,
            &[],
            Some(&lwc_pending),
            &lwc_pane_ids,
            &multi_pane,
        );
        assert!(plan.is_none());
    }

    #[test]
    fn partial_plan_falls_back_to_full_for_lwc_autoscale_invalidation() {
        let main = PaneId::new(0);
        let aux = PaneId::new(1);
        let multi_pane = layered_with_panes(&[main, aux]);
        let legacy_pending = InvalidationMask::light();
        let lwc_pane_ids = vec![main, aux];

        let mut lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Light);
        lwc_pending.invalidate_pane(
            1,
            crate::lwc::model::PaneInvalidation {
                level: crate::lwc::model::InvalidationLevel::Light,
                auto_scale: true,
            },
        );

        let plan = PartialCairoRenderPlan::build_from_masks(
            legacy_pending,
            &[],
            Some(&lwc_pending),
            &lwc_pane_ids,
            &multi_pane,
        );
        assert!(plan.is_none());
    }

    #[test]
    fn partial_plan_falls_back_to_full_when_api_mask_contains_time_scale_topic() {
        let main = PaneId::new(0);
        let aux = PaneId::new(1);
        let multi_pane = layered_with_panes(&[main, aux]);
        let legacy_pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::TimeScale),
        );
        let lwc_pane_ids = vec![main, aux];
        let lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Cursor);

        let plan = PartialCairoRenderPlan::build_from_masks(
            legacy_pending,
            &[],
            Some(&lwc_pending),
            &lwc_pane_ids,
            &multi_pane,
        );
        assert!(plan.is_none());
    }

    #[test]
    fn partial_plan_uses_api_pane_targets_when_provided() {
        let main = PaneId::new(0);
        let pane_a = PaneId::new(1);
        let pane_b = PaneId::new(2);
        let multi_pane = layered_with_panes(&[main, pane_a, pane_b]);
        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::Series),
        );

        let plan =
            PartialCairoRenderPlan::build(pending, &[pane_a, pane_b], &multi_pane).expect("plan");
        assert!(!plan.targets_pane(main));
        assert!(plan.targets_pane(pane_a));
        assert!(plan.targets_pane(pane_b));
    }
}
