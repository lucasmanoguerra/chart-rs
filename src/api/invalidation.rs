use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::core::PaneId;
use crate::error::ChartResult;
use crate::render::{RenderFrame, Renderer};

use super::{ChartEngine, invalidation_render_gate};

/// Ordered invalidation levels aligned with Lightweight Charts repaint classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum InvalidationLevel {
    #[default]
    None,
    Cursor,
    Light,
    Full,
}

impl InvalidationLevel {
    #[must_use]
    pub const fn max(self, other: Self) -> Self {
        if self as u8 >= other as u8 {
            self
        } else {
            other
        }
    }
}

/// Domain-oriented invalidation topic used to classify repaint requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InvalidationTopic {
    General,
    Cursor,
    TimeScale,
    PriceScale,
    Series,
    PaneLayout,
    Axis,
    Style,
    Plugin,
}

impl InvalidationTopic {
    const fn bit(self) -> u16 {
        match self {
            Self::General => 1 << 0,
            Self::Cursor => 1 << 1,
            Self::TimeScale => 1 << 2,
            Self::PriceScale => 1 << 3,
            Self::Series => 1 << 4,
            Self::PaneLayout => 1 << 5,
            Self::Axis => 1 << 6,
            Self::Style => 1 << 7,
            Self::Plugin => 1 << 8,
        }
    }
}

/// Bitmask of invalidation topics used by schedulers for selective redraw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InvalidationTopics {
    bits: u16,
}

impl InvalidationTopics {
    const ALL_BITS: u16 = InvalidationTopic::General.bit()
        | InvalidationTopic::Cursor.bit()
        | InvalidationTopic::TimeScale.bit()
        | InvalidationTopic::PriceScale.bit()
        | InvalidationTopic::Series.bit()
        | InvalidationTopic::PaneLayout.bit()
        | InvalidationTopic::Axis.bit()
        | InvalidationTopic::Style.bit()
        | InvalidationTopic::Plugin.bit();

    #[must_use]
    pub const fn none() -> Self {
        Self { bits: 0 }
    }

    #[must_use]
    pub const fn all() -> Self {
        Self {
            bits: Self::ALL_BITS,
        }
    }

    #[must_use]
    pub const fn from_topic(topic: InvalidationTopic) -> Self {
        Self { bits: topic.bit() }
    }

    #[must_use]
    pub const fn with_topic(self, topic: InvalidationTopic) -> Self {
        Self {
            bits: self.bits | topic.bit(),
        }
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        (self.bits & other.bits) != 0
    }

    #[must_use]
    pub const fn contains_topic(self, topic: InvalidationTopic) -> bool {
        self.intersects(Self::from_topic(topic))
    }

    #[must_use]
    pub const fn is_none(self) -> bool {
        self.bits == 0
    }
}

/// Coalesced invalidation request consumed by frame scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InvalidationMask {
    level: InvalidationLevel,
    #[serde(default)]
    topics: InvalidationTopics,
}

impl InvalidationMask {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            level: InvalidationLevel::None,
            topics: InvalidationTopics::none(),
        }
    }

    #[must_use]
    pub const fn cursor() -> Self {
        Self {
            level: InvalidationLevel::Cursor,
            topics: InvalidationTopics::from_topic(InvalidationTopic::Cursor),
        }
    }

    #[must_use]
    pub const fn light() -> Self {
        Self {
            level: InvalidationLevel::Light,
            topics: InvalidationTopics::from_topic(InvalidationTopic::General),
        }
    }

    #[must_use]
    pub const fn full() -> Self {
        Self {
            level: InvalidationLevel::Full,
            topics: InvalidationTopics::all(),
        }
    }

    #[must_use]
    pub const fn with_level_and_topics(
        level: InvalidationLevel,
        topics: InvalidationTopics,
    ) -> Self {
        Self { level, topics }
    }

    #[must_use]
    pub const fn level(self) -> InvalidationLevel {
        self.level
    }

    #[must_use]
    pub const fn topics(self) -> InvalidationTopics {
        self.topics
    }

    #[must_use]
    pub const fn has_topic(self, topic: InvalidationTopic) -> bool {
        self.topics.contains_topic(topic)
    }

    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self.level, InvalidationLevel::None)
    }

    #[must_use]
    pub const fn with_topics(mut self, topics: InvalidationTopics) -> Self {
        self.topics = topics;
        self
    }

    pub fn merge(&mut self, other: Self) {
        self.level = self.level.max(other.level);
        self.topics = self.topics.union(other.topics);
    }

    pub fn merge_topics(&mut self, topics: InvalidationTopics) {
        self.topics = self.topics.union(topics);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LwcPaneInvalidationSnapshot {
    pub pane_id: PaneId,
    pub level: InvalidationLevel,
    pub auto_scale: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LwcPendingInvalidationSnapshot {
    pub level: InvalidationLevel,
    pub pane_invalidations: Vec<LwcPaneInvalidationSnapshot>,
    pub time_scale_invalidation_count: usize,
}

fn map_invalidation_level(level: InvalidationLevel) -> crate::lwc::model::InvalidationLevel {
    match level {
        InvalidationLevel::None => crate::lwc::model::InvalidationLevel::None,
        InvalidationLevel::Cursor => crate::lwc::model::InvalidationLevel::Cursor,
        InvalidationLevel::Light => crate::lwc::model::InvalidationLevel::Light,
        InvalidationLevel::Full => crate::lwc::model::InvalidationLevel::Full,
    }
}

fn map_lwc_invalidation_level(level: crate::lwc::model::InvalidationLevel) -> InvalidationLevel {
    match level {
        crate::lwc::model::InvalidationLevel::None => InvalidationLevel::None,
        crate::lwc::model::InvalidationLevel::Cursor => InvalidationLevel::Cursor,
        crate::lwc::model::InvalidationLevel::Light => InvalidationLevel::Light,
        crate::lwc::model::InvalidationLevel::Full => InvalidationLevel::Full,
    }
}

impl<R: Renderer> ChartEngine<R> {
    fn approx_eq(left: f64, right: f64) -> bool {
        (left - right).abs() <= 1e-9
    }

    fn record_lwc_time_scale_apply_range_invalidation(
        &mut self,
        topics: InvalidationTopics,
    ) -> ChartResult<()> {
        if !topics.contains_topic(InvalidationTopic::TimeScale) {
            return Ok(());
        }

        if let Some(intent) = self
            .core
            .runtime
            .pending_lwc_time_scale_invalidation_intent
            .take()
        {
            let mut mask = crate::lwc::model::InvalidateMask::light();
            match intent {
                super::chart_runtime::LwcTimeScaleInvalidationIntent::FitContent => {
                    mask.set_fit_content();
                }
                super::chart_runtime::LwcTimeScaleInvalidationIntent::Reset => {
                    mask.reset_time_scale();
                }
                super::chart_runtime::LwcTimeScaleInvalidationIntent::ApplyRange => {
                    let Some(visible_range) =
                        self.core.lwc_model.time_scale_mut().visible_logical_range()
                    else {
                        return Ok(());
                    };
                    mask.apply_range(visible_range);
                }
                super::chart_runtime::LwcTimeScaleInvalidationIntent::ApplyRightOffset => {
                    mask.set_right_offset(self.core.lwc_model.time_scale().right_offset());
                }
                super::chart_runtime::LwcTimeScaleInvalidationIntent::ApplyBarSpacingAndRightOffset => {
                    mask.set_bar_spacing(self.core.lwc_model.time_scale().bar_spacing());
                    mask.set_right_offset(self.core.lwc_model.time_scale().right_offset());
                }
            }
            self.core.lwc_model.invalidate(mask);
            self.core.runtime.last_lwc_time_scale_state =
                Some(super::chart_runtime::LwcTimeScaleStateSnapshot {
                    bar_spacing: self.core.lwc_model.time_scale().bar_spacing(),
                    right_offset: self.core.lwc_model.time_scale().right_offset(),
                });
            return Ok(());
        }

        let current = super::chart_runtime::LwcTimeScaleStateSnapshot {
            bar_spacing: self.core.lwc_model.time_scale().bar_spacing(),
            right_offset: self.core.lwc_model.time_scale().right_offset(),
        };
        let previous = self
            .core
            .runtime
            .last_lwc_time_scale_state
            .unwrap_or(current);

        let bar_spacing_changed = !Self::approx_eq(current.bar_spacing, previous.bar_spacing);
        let right_offset_changed = !Self::approx_eq(current.right_offset, previous.right_offset);

        let mut mask = crate::lwc::model::InvalidateMask::light();
        match (bar_spacing_changed, right_offset_changed) {
            (true, false) => mask.set_bar_spacing(current.bar_spacing),
            (false, true) => mask.set_right_offset(current.right_offset),
            (true, true) | (false, false) => {
                if let Some(visible_range) =
                    self.core.lwc_model.time_scale_mut().visible_logical_range()
                {
                    mask.apply_range(visible_range);
                } else {
                    return Ok(());
                }
            }
        }
        self.core.lwc_model.invalidate(mask);
        self.core.runtime.last_lwc_time_scale_state = Some(current);
        Ok(())
    }

    fn adapt_lwc_pending_to_api(
        &self,
        pending: &crate::lwc::model::InvalidateMask,
    ) -> InvalidationMask {
        let level = map_lwc_invalidation_level(pending.full_invalidation());
        InvalidationMask::with_level_and_topics(level, InvalidationTopics::none())
    }

    fn effective_pending_invalidation(&self) -> InvalidationMask {
        let mut effective = if let Some(lwc_pending) = self.core.lwc_model.pending_invalidation() {
            self.adapt_lwc_pending_to_api(lwc_pending)
        } else {
            InvalidationMask::none()
        };
        effective.merge_topics(self.core.runtime.pending_invalidation_topics.topics());
        effective
    }

    #[must_use]
    pub fn pending_invalidation(&self) -> InvalidationMask {
        self.effective_pending_invalidation()
    }

    #[must_use]
    pub fn pending_invalidation_level(&self) -> InvalidationLevel {
        self.effective_pending_invalidation().level()
    }

    #[must_use]
    pub fn pending_invalidation_topics(&self) -> InvalidationTopics {
        self.effective_pending_invalidation().topics()
    }

    #[must_use]
    pub fn pending_invalidation_pane_targets(&self) -> Vec<PaneId> {
        let mut targets = self
            .lwc_pending_invalidation_snapshot()
            .map(|snapshot| {
                snapshot
                    .pane_invalidations
                    .into_iter()
                    .map(|entry| entry.pane_id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        targets.sort_by_key(|pane_id| pane_id.raw());
        targets.dedup();
        targets
    }

    #[must_use]
    pub fn has_pending_invalidation_topic(&self, topic: InvalidationTopic) -> bool {
        self.effective_pending_invalidation().has_topic(topic)
    }

    #[must_use]
    pub fn has_pending_invalidation(&self) -> bool {
        !self.effective_pending_invalidation().is_none()
    }

    #[must_use]
    pub fn lwc_pending_invalidation(&self) -> Option<&crate::lwc::model::InvalidateMask> {
        self.core.lwc_model.pending_invalidation()
    }

    #[must_use]
    pub fn lwc_pending_invalidation_snapshot(&self) -> Option<LwcPendingInvalidationSnapshot> {
        let pending = self.core.lwc_model.pending_invalidation()?;
        let pane_invalidations = pending
            .explicit_pane_invalidations()
            .into_iter()
            .filter_map(|(pane_index, pane_invalidation)| {
                self.core.lwc_model.panes().get(pane_index).map(|pane| {
                    LwcPaneInvalidationSnapshot {
                        pane_id: pane.id(),
                        level: map_lwc_invalidation_level(pane_invalidation.level),
                        auto_scale: pane_invalidation.auto_scale,
                    }
                })
            })
            .collect::<Vec<_>>();

        Some(LwcPendingInvalidationSnapshot {
            level: map_lwc_invalidation_level(pending.full_invalidation()),
            pane_invalidations,
            time_scale_invalidation_count: pending.time_scale_invalidations().len(),
        })
    }

    pub fn clear_lwc_pending_invalidation(&mut self) {
        let _ = self.core.lwc_model.take_pending_invalidation();
    }

    pub fn clear_pending_invalidation(&mut self) {
        self.core.runtime.pending_invalidation_topics.clear();
        self.clear_lwc_pending_invalidation();
    }

    #[must_use]
    pub fn take_pending_invalidation(&mut self) -> InvalidationMask {
        let pending = self.effective_pending_invalidation();
        self.clear_pending_invalidation();
        pending
    }

    pub fn build_render_frame_if_invalidated(&mut self) -> ChartResult<Option<RenderFrame>> {
        invalidation_render_gate::build_render_frame_if_invalidated(self)
    }

    pub fn render_if_invalidated(&mut self) -> ChartResult<bool> {
        invalidation_render_gate::render_if_invalidated(self)
    }

    pub(super) fn invalidate_with_detail(
        &mut self,
        level: InvalidationLevel,
        topics: InvalidationTopics,
        pane_target: Option<PaneId>,
    ) {
        if let Err(err) = self.sync_lwc_model_for_invalidation_topics(topics) {
            warn!(
                error = %err,
                "failed to synchronize LWC parity model prior to invalidation"
            );
        }
        if let Err(err) = self.record_lwc_time_scale_apply_range_invalidation(topics) {
            warn!(
                error = %err,
                "failed to register LWC time-scale apply-range invalidation"
            );
        }

        self.core
            .runtime
            .pending_invalidation_topics
            .merge_topics(topics);

        match level {
            InvalidationLevel::Full => self.core.lwc_model.full_update(),
            InvalidationLevel::Light => self.core.lwc_model.light_update(),
            InvalidationLevel::Cursor => self.core.lwc_model.cursor_update(),
            InvalidationLevel::None => {}
        }

        if let Some(target_pane_id) = pane_target
            && let Some(index) = self
                .core
                .lwc_model
                .panes()
                .iter()
                .position(|pane| pane.id() == target_pane_id)
        {
            self.core
                .lwc_model
                .invalidate_pane(index, map_invalidation_level(level), false);
        }
    }

    pub(super) fn invalidate_cursor(&mut self) {
        self.invalidate_with_detail(
            InvalidationLevel::Cursor,
            InvalidationTopics::from_topic(InvalidationTopic::Cursor),
            None,
        );
    }

    pub(super) fn invalidate_full(&mut self) {
        self.invalidate_with_detail(InvalidationLevel::Full, InvalidationTopics::all(), None);
    }

    pub(super) fn invalidate_price_scale(&mut self) {
        self.invalidate_with_detail(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::PriceScale),
            None,
        );
    }

    pub(super) fn invalidate_axis(&mut self) {
        self.invalidate_with_detail(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::Axis),
            None,
        );
    }

    pub(super) fn invalidate_pane_layout(&mut self) {
        self.invalidate_with_detail(
            InvalidationLevel::Full,
            InvalidationTopics::from_topic(InvalidationTopic::PaneLayout),
            None,
        );
    }

    pub(super) fn invalidate_pane_content(&mut self, pane_id: PaneId) {
        self.invalidate_with_detail(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::Series)
                .with_topic(InvalidationTopic::PaneLayout),
            Some(pane_id),
        );
    }

    pub(super) fn set_lwc_time_scale_invalidation_intent(
        &mut self,
        intent: super::chart_runtime::LwcTimeScaleInvalidationIntent,
    ) {
        self.core.runtime.pending_lwc_time_scale_invalidation_intent = Some(intent);
    }
}

#[cfg(test)]
mod tests {
    use super::{InvalidationLevel, InvalidationMask, InvalidationTopic, InvalidationTopics};

    #[test]
    fn invalidation_mask_merge_preserves_highest_level() {
        let mut mask = InvalidationMask::none();
        mask.merge(InvalidationMask::cursor());
        assert_eq!(mask.level(), InvalidationLevel::Cursor);

        mask.merge(InvalidationMask::light());
        assert_eq!(mask.level(), InvalidationLevel::Light);

        mask.merge(InvalidationMask::cursor());
        assert_eq!(mask.level(), InvalidationLevel::Light);

        mask.merge(InvalidationMask::full());
        assert_eq!(mask.level(), InvalidationLevel::Full);
    }

    #[test]
    fn invalidation_topics_union_and_contains_work() {
        let topics = InvalidationTopics::from_topic(InvalidationTopic::TimeScale)
            .with_topic(InvalidationTopic::Axis);
        assert!(topics.contains_topic(InvalidationTopic::TimeScale));
        assert!(topics.contains_topic(InvalidationTopic::Axis));
        assert!(!topics.contains_topic(InvalidationTopic::PriceScale));
    }

    #[test]
    fn invalidation_mask_merge_unions_topics() {
        let mut mask = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::Series),
        );

        mask.merge(InvalidationMask::with_level_and_topics(
            InvalidationLevel::Cursor,
            InvalidationTopics::from_topic(InvalidationTopic::Cursor),
        ));

        assert_eq!(mask.level(), InvalidationLevel::Light);
        assert!(mask.has_topic(InvalidationTopic::Series));
        assert!(mask.has_topic(InvalidationTopic::Cursor));
        assert!(!mask.has_topic(InvalidationTopic::PaneLayout));
    }
}
