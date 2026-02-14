use crate::render::{Color, Renderer};

use super::{ChartEngine, LastPriceSourceMode};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn resolve_price_display_base_price(&self) -> f64 {
        let mut candidate: Option<(f64, f64)> = None;

        for point in &self.core.model.points {
            if !point.x.is_finite() || !point.y.is_finite() {
                continue;
            }
            candidate = match candidate {
                Some((best_time, best_price)) if best_time <= point.x => {
                    Some((best_time, best_price))
                }
                _ => Some((point.x, point.y)),
            };
        }

        for candle in &self.core.model.candles {
            if !candle.time.is_finite() || !candle.close.is_finite() {
                continue;
            }
            candidate = match candidate {
                Some((best_time, best_price)) if best_time <= candle.time => {
                    Some((best_time, best_price))
                }
                _ => Some((candle.time, candle.close)),
            };
        }

        if let Some((_, base_price)) = candidate {
            return base_price;
        }

        let domain = self.core.model.price_scale.domain();
        if domain.0.is_finite() { domain.0 } else { 1.0 }
    }

    fn resolve_latest_price_sample_with_window(
        &self,
        window: Option<(f64, f64)>,
    ) -> Option<(f64, f64)> {
        let normalized_window = window.map(|(start, end)| {
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        });
        let mut candidate: Option<(f64, f64)> = None;

        for point in &self.core.model.points {
            if !point.x.is_finite() || !point.y.is_finite() {
                continue;
            }
            if let Some((window_start, window_end)) = normalized_window
                && (point.x < window_start || point.x > window_end)
            {
                continue;
            }
            candidate = match candidate {
                Some((best_time, best_price)) if best_time >= point.x => {
                    Some((best_time, best_price))
                }
                _ => Some((point.x, point.y)),
            };
        }

        for candle in &self.core.model.candles {
            if !candle.time.is_finite() || !candle.close.is_finite() {
                continue;
            }
            if let Some((window_start, window_end)) = normalized_window
                && (candle.time < window_start || candle.time > window_end)
            {
                continue;
            }
            candidate = match candidate {
                Some((best_time, best_price)) if best_time >= candle.time => {
                    Some((best_time, best_price))
                }
                _ => Some((candle.time, candle.close)),
            };
        }

        candidate
    }

    fn resolve_previous_price_before_time_with_window(
        &self,
        latest_time: f64,
        window: Option<(f64, f64)>,
    ) -> Option<f64> {
        let normalized_window = window.map(|(start, end)| {
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        });
        let mut candidate: Option<(f64, f64)> = None;

        for point in &self.core.model.points {
            if !point.x.is_finite() || !point.y.is_finite() || point.x >= latest_time {
                continue;
            }
            if let Some((window_start, window_end)) = normalized_window
                && (point.x < window_start || point.x > window_end)
            {
                continue;
            }
            // Preserve first-seen winner for equal timestamps to keep frame snapshots stable.
            candidate = match candidate {
                Some((best_time, best_price)) if best_time >= point.x => {
                    Some((best_time, best_price))
                }
                _ => Some((point.x, point.y)),
            };
        }

        for candle in &self.core.model.candles {
            if !candle.time.is_finite() || !candle.close.is_finite() || candle.time >= latest_time {
                continue;
            }
            if let Some((window_start, window_end)) = normalized_window
                && (candle.time < window_start || candle.time > window_end)
            {
                continue;
            }
            candidate = match candidate {
                Some((best_time, best_price)) if best_time >= candle.time => {
                    Some((best_time, best_price))
                }
                _ => Some((candle.time, candle.close)),
            };
        }

        candidate.map(|(_, price)| price)
    }

    pub(super) fn resolve_latest_and_previous_price_values(
        &self,
        source_mode: LastPriceSourceMode,
        visible_start: f64,
        visible_end: f64,
    ) -> Option<(f64, Option<f64>)> {
        let window = match source_mode {
            LastPriceSourceMode::LatestData => None,
            LastPriceSourceMode::LatestVisible => Some((visible_start, visible_end)),
        };
        let (latest_time, latest_price) = self.resolve_latest_price_sample_with_window(window)?;
        let previous_price =
            self.resolve_previous_price_before_time_with_window(latest_time, window);
        Some((latest_price, previous_price))
    }

    pub(super) fn resolve_last_price_marker_colors(
        &self,
        latest_price: f64,
        previous_price: Option<f64>,
    ) -> (Color, Color) {
        let style = self.core.presentation.render_style;
        if !style.last_price_use_trend_color {
            return (style.last_price_line_color, style.last_price_label_color);
        }

        let trend_color = match previous_price {
            Some(previous) if latest_price > previous => style.last_price_up_color,
            Some(previous) if latest_price < previous => style.last_price_down_color,
            _ => style.last_price_neutral_color,
        };
        (trend_color, trend_color)
    }

    pub(super) fn resolve_last_price_label_box_fill_color(
        &self,
        marker_label_color: Color,
    ) -> Color {
        let style = self.core.presentation.render_style;
        if style.last_price_label_box_use_marker_color {
            marker_label_color
        } else {
            style.last_price_label_box_color
        }
    }

    pub(super) fn resolve_last_price_label_box_text_color(
        &self,
        box_fill_color: Color,
        marker_label_color: Color,
    ) -> Color {
        let style = self.core.presentation.render_style;
        if !style.show_last_price_label_box {
            return marker_label_color;
        }
        if !style.last_price_label_box_auto_text_contrast {
            return style.last_price_label_box_text_color;
        }

        Self::resolve_auto_contrast_text_color(box_fill_color)
    }

    pub(super) fn resolve_crosshair_label_box_text_color(
        &self,
        fallback_text_color: Color,
        box_fill_color: Color,
        per_axis_text_color: Option<Color>,
        per_axis_auto_contrast: Option<bool>,
    ) -> Color {
        let style = self.core.presentation.render_style;
        let auto_contrast =
            per_axis_auto_contrast.unwrap_or(style.crosshair_label_box_auto_text_contrast);
        if !auto_contrast {
            return per_axis_text_color.unwrap_or(style.crosshair_label_box_text_color);
        }
        if !style.show_crosshair_time_label_box && !style.show_crosshair_price_label_box {
            return fallback_text_color;
        }

        Self::resolve_auto_contrast_text_color(box_fill_color)
    }

    pub(super) fn resolve_auto_contrast_text_color(box_fill_color: Color) -> Color {
        // WCAG-inspired luminance gate keeps axis text readable on dynamic marker fills.
        let luminance = 0.2126 * box_fill_color.red
            + 0.7152 * box_fill_color.green
            + 0.0722 * box_fill_color.blue;
        if luminance >= 0.56 {
            Color::rgb(0.06, 0.08, 0.11)
        } else {
            Color::rgb(1.0, 1.0, 1.0)
        }
    }
}
