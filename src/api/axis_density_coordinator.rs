use crate::error::ChartResult;
use crate::render::Renderer;

use super::ChartEngine;
use super::axis_ticks::{AXIS_TIME_MIN_SPACING_PX, density_scale_from_zoom_ratio};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn lwc_time_label_width_budget_px(font_size_px: f64) -> f64 {
        if !font_size_px.is_finite() {
            return AXIS_TIME_MIN_SPACING_PX;
        }
        // Lightweight Charts uses an 8-char budget to keep cadence stable even
        // when a few labels are much longer than the typical tick text.
        let pixels_per_eight_chars = (font_size_px.max(1.0) + 4.0) * 5.0;
        pixels_per_eight_chars.max(AXIS_TIME_MIN_SPACING_PX)
    }

    pub(super) fn resolve_time_axis_density_scale(&self) -> f64 {
        let (visible_start, visible_end) = self.core.model.time_scale.visible_range();
        let visible_span = (visible_end - visible_start).abs();
        let (full_start, full_end) = self.core.model.time_scale.full_range();
        let full_span = (full_end - full_start).abs();
        if !visible_span.is_finite() || !full_span.is_finite() || full_span <= 0.0 {
            return 1.0;
        }

        let zoom_ratio = visible_span / full_span;
        density_scale_from_zoom_ratio(zoom_ratio, 0.06, 0.70, 0.62, 0.45, 1.90)
    }

    fn resolve_series_price_span(&self) -> Option<f64> {
        let mut min_price = f64::INFINITY;
        let mut max_price = f64::NEG_INFINITY;

        for point in &self.core.model.points {
            if !point.y.is_finite() {
                continue;
            }
            min_price = min_price.min(point.y);
            max_price = max_price.max(point.y);
        }
        for candle in &self.core.model.candles {
            if candle.low.is_finite() {
                min_price = min_price.min(candle.low);
            }
            if candle.high.is_finite() {
                max_price = max_price.max(candle.high);
            }
        }

        let span = max_price - min_price;
        if span.is_finite() && span > 0.0 {
            Some(span.abs())
        } else {
            None
        }
    }

    pub(super) fn resolve_price_axis_density_scale(&self) -> f64 {
        let (domain_start, domain_end) = self.core.model.price_scale.domain();
        let domain_span = (domain_end - domain_start).abs();
        if !domain_span.is_finite() || domain_span <= 0.0 {
            return 1.0;
        }

        let Some(series_span) = self.resolve_series_price_span() else {
            return 1.0;
        };
        let zoom_ratio = domain_span / series_span;
        density_scale_from_zoom_ratio(zoom_ratio, 0.10, 0.75, 0.65, 0.55, 1.80)
    }

    pub(super) fn resolve_price_axis_span_px(&self, plot_bottom: f64) -> ChartResult<f64> {
        let (domain_start, domain_end) = self.core.model.price_scale.domain();
        let start_py = self
            .core
            .model
            .price_scale
            .price_to_pixel(domain_start, self.core.model.viewport)?;
        let end_py = self
            .core
            .model
            .price_scale
            .price_to_pixel(domain_end, self.core.model.viewport)?;
        let span = (start_py - end_py).abs();
        if span.is_finite() && span > 0.0 {
            Ok(span.min(plot_bottom).max(1.0))
        } else {
            Ok(plot_bottom.max(1.0))
        }
    }
}
