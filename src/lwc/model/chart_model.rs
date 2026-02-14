use crate::core::PaneId;
use crate::error::{ChartError, ChartResult};
use std::collections::HashMap;

use super::{
    InvalidateMask, InvalidationLevel, LogicalRange, Pane, PaneInvalidation, PriceScaleOptions,
    TimeScale, TimeScaleAnimation, TimeScaleInvalidation, TimeScaleOptions,
};

#[derive(Debug, Clone)]
pub struct ChartModel {
    width: f64,
    time_scale: TimeScale,
    panes: Vec<Pane>,
    pending_invalidation: Option<InvalidateMask>,
}

impl ChartModel {
    #[must_use]
    pub fn with_default_pane(width: f64) -> Self {
        let mut time_scale = TimeScale::new(TimeScaleOptions::default());
        if width.is_finite() && width > 0.0 {
            let _ = time_scale.set_width(width);
        }
        let pane = Pane::new(
            PaneId::new(0),
            PriceScaleOptions::default(),
            PriceScaleOptions {
                auto_scale: true,
                ..PriceScaleOptions::default()
            },
        );
        Self {
            width,
            time_scale,
            panes: vec![pane],
            pending_invalidation: Some(InvalidateMask::full()),
        }
    }

    #[must_use]
    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn set_width(&mut self, width: f64) -> ChartResult<()> {
        if !width.is_finite() || width <= 0.0 {
            return Err(ChartError::InvalidData(
                "chart model width must be finite and > 0".to_owned(),
            ));
        }
        self.width = width;
        self.time_scale.set_width(width)?;
        self.invalidate(InvalidateMask::full());
        Ok(())
    }

    #[must_use]
    pub fn time_scale(&self) -> &TimeScale {
        &self.time_scale
    }

    #[must_use]
    pub fn time_scale_mut(&mut self) -> &mut TimeScale {
        &mut self.time_scale
    }

    #[must_use]
    pub fn panes(&self) -> &[Pane] {
        &self.panes
    }

    #[must_use]
    pub fn panes_mut(&mut self) -> &mut [Pane] {
        &mut self.panes
    }

    pub fn invalidate(&mut self, mask: InvalidateMask) {
        if let Some(pending) = &mut self.pending_invalidation {
            pending.merge(&mask);
        } else {
            self.pending_invalidation = Some(mask);
        }
    }

    #[must_use]
    pub fn pending_invalidation(&self) -> Option<&InvalidateMask> {
        self.pending_invalidation.as_ref()
    }

    pub fn take_pending_invalidation(&mut self) -> Option<InvalidateMask> {
        self.pending_invalidation.take()
    }

    pub fn full_update(&mut self) {
        self.invalidate(InvalidateMask::full());
    }

    pub fn light_update(&mut self) {
        self.invalidate(InvalidateMask::light());
    }

    pub fn cursor_update(&mut self) {
        self.invalidate(InvalidateMask::new(InvalidationLevel::Cursor));
    }

    pub fn fit_content(&mut self) -> ChartResult<()> {
        let mut mask = InvalidateMask::light();
        mask.set_fit_content();
        self.apply_time_scale_invalidations(&mask)?;
        self.invalidate(mask);
        Ok(())
    }

    pub fn set_target_logical_range(&mut self, range: LogicalRange) -> ChartResult<()> {
        let mut mask = InvalidateMask::light();
        mask.apply_range(range);
        self.apply_time_scale_invalidations(&mask)?;
        self.invalidate(mask);
        Ok(())
    }

    pub fn set_bar_spacing(&mut self, bar_spacing: f64) -> ChartResult<()> {
        let mut mask = InvalidateMask::light();
        mask.set_bar_spacing(bar_spacing);
        self.apply_time_scale_invalidations(&mask)?;
        self.invalidate(mask);
        Ok(())
    }

    pub fn set_right_offset(&mut self, right_offset: f64) -> ChartResult<()> {
        let mut mask = InvalidateMask::light();
        mask.set_right_offset(right_offset);
        self.apply_time_scale_invalidations(&mask)?;
        self.invalidate(mask);
        Ok(())
    }

    pub fn reset_time_scale(&mut self) -> ChartResult<()> {
        let mut mask = InvalidateMask::light();
        mask.reset_time_scale();
        self.apply_time_scale_invalidations(&mask)?;
        self.invalidate(mask);
        Ok(())
    }

    pub fn set_time_scale_animation(&mut self, animation: TimeScaleAnimation) {
        let mut mask = InvalidateMask::light();
        mask.set_time_scale_animation(animation);
        self.invalidate(mask);
    }

    pub fn stop_time_scale_animation(&mut self) {
        let mut mask = InvalidateMask::light();
        mask.stop_time_scale_animation();
        self.invalidate(mask);
    }

    pub fn invalidate_pane(
        &mut self,
        pane_index: usize,
        level: InvalidationLevel,
        auto_scale: bool,
    ) {
        let mut mask = InvalidateMask::new(level);
        mask.invalidate_pane(pane_index, PaneInvalidation { level, auto_scale });
        self.invalidate(mask);
    }

    pub fn sync_panes(&mut self, pane_ids: &[PaneId]) {
        if pane_ids.is_empty() {
            return;
        }

        let current = self.panes.iter().map(Pane::id).collect::<Vec<_>>();
        if current == pane_ids {
            return;
        }

        let mut existing = self
            .panes
            .drain(..)
            .map(|pane| (pane.id(), pane))
            .collect::<HashMap<_, _>>();
        let mut rebuilt = Vec::with_capacity(pane_ids.len());

        for pane_id in pane_ids {
            if let Some(existing_pane) = existing.remove(pane_id) {
                rebuilt.push(existing_pane);
            } else {
                rebuilt.push(Pane::new(
                    *pane_id,
                    PriceScaleOptions::default(),
                    PriceScaleOptions {
                        auto_scale: true,
                        ..PriceScaleOptions::default()
                    },
                ));
            }
        }

        self.panes = rebuilt;
        self.full_update();
    }

    pub fn pane_by_id_mut(&mut self, pane_id: PaneId) -> Option<&mut Pane> {
        self.panes.iter_mut().find(|pane| pane.id() == pane_id)
    }

    fn apply_time_scale_invalidations(&mut self, mask: &InvalidateMask) -> ChartResult<()> {
        for invalidation in mask.time_scale_invalidations() {
            match invalidation {
                TimeScaleInvalidation::FitContent => self.time_scale.fit_content()?,
                TimeScaleInvalidation::ApplyRange(range) => {
                    self.time_scale.set_logical_range(*range)?
                }
                TimeScaleInvalidation::ApplyBarSpacing(spacing) => {
                    self.time_scale.set_bar_spacing(*spacing)?
                }
                TimeScaleInvalidation::ApplyRightOffset(offset) => {
                    self.time_scale.set_right_offset(*offset)?
                }
                TimeScaleInvalidation::Reset => self.time_scale.restore_default()?,
                TimeScaleInvalidation::Animation(animation) => {
                    if !animation.finished(animation.start_time) {
                        self.time_scale
                            .set_right_offset(animation.position(animation.start_time))?;
                    }
                }
                TimeScaleInvalidation::StopAnimation => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ChartModel;
    use crate::lwc::model::InvalidationLevel;

    #[test]
    fn default_chart_model_starts_with_full_invalidation() {
        let model = ChartModel::with_default_pane(800.0);
        let pending = model.pending_invalidation().expect("pending");
        assert_eq!(pending.full_invalidation(), InvalidationLevel::Full);
    }

    #[test]
    fn invalidate_merges_masks() {
        let mut model = ChartModel::with_default_pane(800.0);
        model.take_pending_invalidation();
        model.light_update();
        model.cursor_update();
        let pending = model.pending_invalidation().expect("pending");
        assert_eq!(pending.full_invalidation(), InvalidationLevel::Light);
    }
}
