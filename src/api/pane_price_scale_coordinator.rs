use crate::core::{PaneId, PriceScale};
use crate::render::Renderer;

use super::ChartEngine;

impl<R: Renderer> ChartEngine<R> {
    fn pane_data_price_extents(
        &self,
        pane_id: PaneId,
        visible_range: Option<(f64, f64)>,
    ) -> Option<(f64, f64)> {
        let mut min_price = f64::INFINITY;
        let mut max_price = f64::NEG_INFINITY;

        let is_visible = |time: f64| {
            if let Some((start, end)) = visible_range {
                let lo = start.min(end);
                let hi = start.max(end);
                time >= lo && time <= hi
            } else {
                true
            }
        };

        if self.core.model.points_pane_id == pane_id {
            for point in &self.core.model.points {
                if !point.y.is_finite() || !is_visible(point.x) {
                    continue;
                }
                min_price = min_price.min(point.y);
                max_price = max_price.max(point.y);
            }
        }

        if self.core.model.candles_pane_id == pane_id {
            for candle in &self.core.model.candles {
                if !is_visible(candle.time) {
                    continue;
                }
                if candle.low.is_finite() {
                    min_price = min_price.min(candle.low);
                }
                if candle.high.is_finite() {
                    max_price = max_price.max(candle.high);
                }
            }
        }

        if !min_price.is_finite() || !max_price.is_finite() {
            return None;
        }

        if (max_price - min_price).abs() <= f64::EPSILON {
            let center = min_price;
            let pad = center.abs().max(1.0) * 1e-6;
            Some((center - pad, center + pad))
        } else {
            Some((min_price, max_price))
        }
    }

    pub(super) fn resolve_render_price_scale_for_pane(
        &self,
        pane_id: PaneId,
        visible_start: f64,
        visible_end: f64,
    ) -> PriceScale {
        let pane_extents = self
            .pane_data_price_extents(pane_id, Some((visible_start, visible_end)))
            .or_else(|| self.pane_data_price_extents(pane_id, None));
        let Some((domain_start, domain_end)) = pane_extents else {
            return self.core.model.price_scale;
        };

        let base_value = self.core.model.price_scale.base_value();
        let keep_inverted = self.core.model.price_scale.is_inverted();
        let keep_margins = self.core.model.price_scale.margins();
        let Ok(scale) = PriceScale::new_with_mode_and_base(
            domain_start,
            domain_end,
            self.core.model.price_scale_mode,
            base_value,
        ) else {
            return self.core.model.price_scale;
        };
        let Ok(scale) = scale.with_margins(keep_margins.0, keep_margins.1) else {
            return self.core.model.price_scale;
        };
        scale.with_inverted(keep_inverted)
    }
}
