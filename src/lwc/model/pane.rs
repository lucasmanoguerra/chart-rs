use std::collections::BTreeMap;

use crate::core::PaneId;

use super::{PriceScale, PriceScaleOptions};

#[derive(Debug, Clone)]
pub struct Pane {
    id: PaneId,
    stretch_factor: f64,
    preserve_empty_pane: bool,
    left_price_scale: PriceScale,
    right_price_scale: PriceScale,
    overlay_price_scales: BTreeMap<String, PriceScale>,
}

impl Pane {
    #[must_use]
    pub fn new(
        id: PaneId,
        left_options: PriceScaleOptions,
        right_options: PriceScaleOptions,
    ) -> Self {
        Self {
            id,
            stretch_factor: 1.0,
            preserve_empty_pane: false,
            left_price_scale: PriceScale::new("left", left_options),
            right_price_scale: PriceScale::new("right", right_options),
            overlay_price_scales: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn id(&self) -> PaneId {
        self.id
    }

    #[must_use]
    pub fn stretch_factor(&self) -> f64 {
        self.stretch_factor
    }

    pub fn set_stretch_factor(&mut self, stretch_factor: f64) {
        self.stretch_factor = stretch_factor;
    }

    #[must_use]
    pub fn preserve_empty_pane(&self) -> bool {
        self.preserve_empty_pane
    }

    pub fn set_preserve_empty_pane(&mut self, preserve: bool) {
        self.preserve_empty_pane = preserve;
    }

    pub fn set_height(&mut self, height: f64) {
        self.left_price_scale.set_height(height);
        self.right_price_scale.set_height(height);
        for scale in self.overlay_price_scales.values_mut() {
            scale.set_height(height);
        }
    }

    #[must_use]
    pub fn left_price_scale(&self) -> &PriceScale {
        &self.left_price_scale
    }

    #[must_use]
    pub fn left_price_scale_mut(&mut self) -> &mut PriceScale {
        &mut self.left_price_scale
    }

    #[must_use]
    pub fn right_price_scale(&self) -> &PriceScale {
        &self.right_price_scale
    }

    #[must_use]
    pub fn right_price_scale_mut(&mut self) -> &mut PriceScale {
        &mut self.right_price_scale
    }

    #[must_use]
    pub fn overlay_price_scale(&self, id: &str) -> Option<&PriceScale> {
        self.overlay_price_scales.get(id)
    }

    #[must_use]
    pub fn overlay_price_scale_mut(&mut self, id: &str) -> Option<&mut PriceScale> {
        self.overlay_price_scales.get_mut(id)
    }

    pub fn ensure_overlay_price_scale(
        &mut self,
        id: impl Into<String>,
        options: PriceScaleOptions,
    ) -> &mut PriceScale {
        let id = id.into();
        self.overlay_price_scales
            .entry(id.clone())
            .or_insert_with(|| PriceScale::new(id, options))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::PaneId;

    use super::Pane;
    use crate::lwc::model::PriceScaleOptions;

    #[test]
    fn pane_creates_left_right_price_scales() {
        let pane = Pane::new(
            PaneId::new(0),
            PriceScaleOptions::default(),
            PriceScaleOptions::default(),
        );
        assert_eq!(pane.left_price_scale().id(), "left");
        assert_eq!(pane.right_price_scale().id(), "right");
    }
}
