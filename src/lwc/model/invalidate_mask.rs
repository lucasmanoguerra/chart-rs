use std::collections::BTreeMap;

use super::LogicalRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(u8)]
pub enum InvalidationLevel {
    #[default]
    None = 0,
    Cursor = 1,
    Light = 2,
    Full = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PaneInvalidation {
    pub level: InvalidationLevel,
    pub auto_scale: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeScaleInvalidationType {
    FitContent,
    ApplyRange,
    ApplyBarSpacing,
    ApplyRightOffset,
    Reset,
    Animation,
    StopAnimation,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeScaleAnimation {
    pub from: f64,
    pub to: f64,
    pub start_time: f64,
    pub duration: f64,
}

impl TimeScaleAnimation {
    #[must_use]
    pub fn finished(self, now: f64) -> bool {
        if self.duration <= 0.0 {
            return true;
        }
        ((now - self.start_time) / self.duration) >= 1.0
    }

    #[must_use]
    pub fn position(self, now: f64) -> f64 {
        if self.finished(now) || self.duration <= 0.0 {
            return self.to;
        }
        let progress = ((now - self.start_time) / self.duration).clamp(0.0, 1.0);
        self.from + (self.to - self.from) * progress
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeScaleInvalidation {
    FitContent,
    ApplyRange(LogicalRange),
    ApplyBarSpacing(f64),
    ApplyRightOffset(f64),
    Reset,
    Animation(TimeScaleAnimation),
    StopAnimation,
}

impl TimeScaleInvalidation {
    #[must_use]
    pub fn kind(self) -> TimeScaleInvalidationType {
        match self {
            Self::FitContent => TimeScaleInvalidationType::FitContent,
            Self::ApplyRange(_) => TimeScaleInvalidationType::ApplyRange,
            Self::ApplyBarSpacing(_) => TimeScaleInvalidationType::ApplyBarSpacing,
            Self::ApplyRightOffset(_) => TimeScaleInvalidationType::ApplyRightOffset,
            Self::Reset => TimeScaleInvalidationType::Reset,
            Self::Animation(_) => TimeScaleInvalidationType::Animation,
            Self::StopAnimation => TimeScaleInvalidationType::StopAnimation,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InvalidateMask {
    invalidated_panes: BTreeMap<usize, PaneInvalidation>,
    global_level: InvalidationLevel,
    time_scale_invalidations: Vec<TimeScaleInvalidation>,
}

impl InvalidateMask {
    #[must_use]
    pub fn new(global_level: InvalidationLevel) -> Self {
        Self {
            invalidated_panes: BTreeMap::new(),
            global_level,
            time_scale_invalidations: Vec::new(),
        }
    }

    #[must_use]
    pub fn full() -> Self {
        Self::new(InvalidationLevel::Full)
    }

    #[must_use]
    pub fn light() -> Self {
        Self::new(InvalidationLevel::Light)
    }

    #[must_use]
    pub fn full_invalidation(&self) -> InvalidationLevel {
        self.global_level
    }

    pub fn invalidate_pane(&mut self, pane_index: usize, invalidation: PaneInvalidation) {
        let merged = if let Some(previous) = self.invalidated_panes.get(&pane_index) {
            PaneInvalidation {
                level: previous.level.max(invalidation.level),
                auto_scale: previous.auto_scale || invalidation.auto_scale,
            }
        } else {
            invalidation
        };
        self.invalidated_panes.insert(pane_index, merged);
    }

    #[must_use]
    pub fn invalidation_for_pane(&self, pane_index: usize) -> PaneInvalidation {
        if let Some(pane) = self.invalidated_panes.get(&pane_index) {
            PaneInvalidation {
                level: self.global_level.max(pane.level),
                auto_scale: pane.auto_scale,
            }
        } else {
            PaneInvalidation {
                level: self.global_level,
                auto_scale: false,
            }
        }
    }

    #[must_use]
    pub fn explicit_pane_invalidations(&self) -> Vec<(usize, PaneInvalidation)> {
        self.invalidated_panes
            .iter()
            .map(|(pane_index, invalidation)| (*pane_index, *invalidation))
            .collect()
    }

    #[must_use]
    pub fn time_scale_invalidations(&self) -> &[TimeScaleInvalidation] {
        &self.time_scale_invalidations
    }

    pub fn set_fit_content(&mut self) {
        self.stop_time_scale_animation();
        self.time_scale_invalidations = vec![TimeScaleInvalidation::FitContent];
    }

    pub fn apply_range(&mut self, range: LogicalRange) {
        self.stop_time_scale_animation();
        self.time_scale_invalidations = vec![TimeScaleInvalidation::ApplyRange(range)];
    }

    pub fn set_bar_spacing(&mut self, spacing: f64) {
        self.stop_time_scale_animation();
        self.time_scale_invalidations
            .push(TimeScaleInvalidation::ApplyBarSpacing(spacing));
    }

    pub fn set_right_offset(&mut self, offset: f64) {
        self.stop_time_scale_animation();
        self.time_scale_invalidations
            .push(TimeScaleInvalidation::ApplyRightOffset(offset));
    }

    pub fn reset_time_scale(&mut self) {
        self.stop_time_scale_animation();
        self.time_scale_invalidations = vec![TimeScaleInvalidation::Reset];
    }

    pub fn set_time_scale_animation(&mut self, animation: TimeScaleAnimation) {
        self.remove_time_scale_animation();
        self.time_scale_invalidations
            .push(TimeScaleInvalidation::Animation(animation));
    }

    pub fn stop_time_scale_animation(&mut self) {
        self.remove_time_scale_animation();
        self.time_scale_invalidations
            .push(TimeScaleInvalidation::StopAnimation);
    }

    pub fn merge(&mut self, other: &InvalidateMask) {
        for invalidation in &other.time_scale_invalidations {
            self.apply_time_scale_invalidation(*invalidation);
        }
        self.global_level = self.global_level.max(other.global_level);
        for (pane_index, pane) in &other.invalidated_panes {
            self.invalidate_pane(*pane_index, *pane);
        }
    }

    fn apply_time_scale_invalidation(&mut self, invalidation: TimeScaleInvalidation) {
        match invalidation {
            TimeScaleInvalidation::FitContent => self.set_fit_content(),
            TimeScaleInvalidation::ApplyRange(range) => self.apply_range(range),
            TimeScaleInvalidation::ApplyBarSpacing(spacing) => self.set_bar_spacing(spacing),
            TimeScaleInvalidation::ApplyRightOffset(offset) => self.set_right_offset(offset),
            TimeScaleInvalidation::Reset => self.reset_time_scale(),
            TimeScaleInvalidation::Animation(animation) => self.set_time_scale_animation(animation),
            TimeScaleInvalidation::StopAnimation => self.remove_time_scale_animation(),
        }
    }

    fn remove_time_scale_animation(&mut self) {
        if let Some(position) = self
            .time_scale_invalidations
            .iter()
            .position(|inv| matches!(inv, TimeScaleInvalidation::Animation(_)))
        {
            self.time_scale_invalidations.remove(position);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        InvalidateMask, InvalidationLevel, LogicalRange, PaneInvalidation, TimeScaleAnimation,
        TimeScaleInvalidation, TimeScaleInvalidationType,
    };

    #[test]
    fn pane_invalidation_merges_level_and_autoscale() {
        let mut mask = InvalidateMask::new(InvalidationLevel::None);
        mask.invalidate_pane(
            1,
            PaneInvalidation {
                level: InvalidationLevel::Cursor,
                auto_scale: false,
            },
        );
        mask.invalidate_pane(
            1,
            PaneInvalidation {
                level: InvalidationLevel::Light,
                auto_scale: true,
            },
        );
        let result = mask.invalidation_for_pane(1);
        assert_eq!(result.level, InvalidationLevel::Light);
        assert!(result.auto_scale);
    }

    #[test]
    fn set_fit_content_replaces_previous_time_scale_invalidations() {
        let mut mask = InvalidateMask::light();
        mask.set_bar_spacing(8.0);
        mask.apply_range(LogicalRange {
            from: 10.0,
            to: 20.0,
        });
        assert_eq!(
            mask.time_scale_invalidations(),
            &[TimeScaleInvalidation::ApplyRange(LogicalRange {
                from: 10.0,
                to: 20.0
            })]
        );
        mask.set_fit_content();
        assert_eq!(
            mask.time_scale_invalidations(),
            &[TimeScaleInvalidation::FitContent]
        );
    }

    #[test]
    fn animation_is_removed_before_pushing_new_animation_or_stop() {
        let mut mask = InvalidateMask::light();
        let a = TimeScaleAnimation {
            from: 0.0,
            to: 5.0,
            start_time: 0.0,
            duration: 100.0,
        };
        let b = TimeScaleAnimation {
            from: 5.0,
            to: 10.0,
            start_time: 10.0,
            duration: 100.0,
        };

        mask.set_time_scale_animation(a);
        mask.set_time_scale_animation(b);
        assert_eq!(mask.time_scale_invalidations().len(), 1);
        assert_eq!(
            mask.time_scale_invalidations()[0].kind(),
            TimeScaleInvalidationType::Animation
        );

        mask.stop_time_scale_animation();
        assert_eq!(mask.time_scale_invalidations().len(), 1);
        assert_eq!(
            mask.time_scale_invalidations()[0].kind(),
            TimeScaleInvalidationType::StopAnimation
        );
    }

    #[test]
    fn merge_preserves_stronger_global_level_and_combines_panes() {
        let mut a = InvalidateMask::new(InvalidationLevel::Cursor);
        let mut b = InvalidateMask::new(InvalidationLevel::Light);
        a.invalidate_pane(
            0,
            PaneInvalidation {
                level: InvalidationLevel::Cursor,
                auto_scale: false,
            },
        );
        b.invalidate_pane(
            0,
            PaneInvalidation {
                level: InvalidationLevel::Light,
                auto_scale: true,
            },
        );
        b.invalidate_pane(
            1,
            PaneInvalidation {
                level: InvalidationLevel::Cursor,
                auto_scale: false,
            },
        );
        a.merge(&b);
        assert_eq!(a.full_invalidation(), InvalidationLevel::Light);
        assert_eq!(a.invalidation_for_pane(0).level, InvalidationLevel::Light);
        assert!(a.invalidation_for_pane(0).auto_scale);
        assert_eq!(a.invalidation_for_pane(1).level, InvalidationLevel::Light);
    }
}
