use crate::extensions::ChartPlugin;

use super::InvalidationTopics;

/// Legacy topic accumulator kept while migrating fully to LWC invalidation.
pub(super) struct PendingInvalidationTopics {
    topics: InvalidationTopics,
}

impl PendingInvalidationTopics {
    #[must_use]
    pub(super) fn with_all_topics() -> Self {
        Self {
            topics: InvalidationTopics::all(),
        }
    }

    #[must_use]
    pub(super) fn topics(&self) -> InvalidationTopics {
        self.topics
    }

    pub(super) fn merge_topics(&mut self, topics: InvalidationTopics) {
        self.topics = self.topics.union(topics);
    }

    pub(super) fn clear(&mut self) {
        self.topics = InvalidationTopics::none();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LwcTimeScaleInvalidationIntent {
    FitContent,
    Reset,
    ApplyRange,
    ApplyRightOffset,
    ApplyBarSpacingAndRightOffset,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct LwcTimeScaleStateSnapshot {
    pub(super) bar_spacing: f64,
    pub(super) right_offset: f64,
}

/// Runtime orchestration state grouped separately from model/behavior/presentation.
pub(super) struct ChartRuntimeState {
    pub(super) plugins: Vec<Box<dyn ChartPlugin>>,
    pub(super) pending_invalidation_topics: PendingInvalidationTopics,
    pub(super) pending_lwc_time_scale_invalidation_intent: Option<LwcTimeScaleInvalidationIntent>,
    pub(super) last_lwc_time_scale_state: Option<LwcTimeScaleStateSnapshot>,
}

impl ChartRuntimeState {
    #[must_use]
    pub(super) fn with_full_invalidation() -> Self {
        Self {
            plugins: Vec::new(),
            pending_invalidation_topics: PendingInvalidationTopics::with_all_topics(),
            pending_lwc_time_scale_invalidation_intent: None,
            last_lwc_time_scale_state: None,
        }
    }
}
