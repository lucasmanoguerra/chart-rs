use crate::core::{OhlcBar, PaneId, PriceScale, project_candles};
use crate::error::ChartResult;
use crate::render::{
    CanvasLayerKind, Color, LayeredRenderFrame, LinePrimitive, RectPrimitive, RenderFrame, Renderer,
};

use super::{CandlestickBodyMode, ChartEngine, RenderStyle};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn append_candlestick_series_primitives(
        &self,
        frame: &mut RenderFrame,
        layered: &mut LayeredRenderFrame,
        pane_and_scale: (PaneId, PriceScale),
        visible_range: (f64, f64),
        plot_right: f64,
        style: RenderStyle,
    ) -> ChartResult<()> {
        let (candles_pane_id, candles_scale) = pane_and_scale;
        let (visible_start, visible_end) = visible_range;
        let visible_candle_indices =
            self.visible_candle_indices_in_time_window(visible_start, visible_end);
        if visible_candle_indices.is_empty() {
            return Ok(());
        }

        let visible_candles: Vec<OhlcBar> = visible_candle_indices
            .iter()
            .map(|&idx| self.core.model.candles[idx])
            .collect();
        let candle_bar_spacing =
            self.resolve_candlestick_bar_spacing_px(&visible_candles, plot_right);
        let candle_body_width =
            self.resolve_candlestick_body_width_px(&visible_candles, plot_right);
        let wick_width = Self::resolve_effective_candlestick_wick_width_px(
            style.candlestick_wick_width_px,
            candle_bar_spacing,
            candle_body_width,
            1.0,
        );
        let border_width = Self::resolve_effective_candlestick_border_width_px(
            style.candlestick_border_width_px,
            candle_body_width,
            1.0,
        );
        let render_border_only_body = style.show_candlestick_borders
            && border_width > 0.0
            && candle_body_width <= 2.0 * border_width;
        let candle_geometries = project_candles(
            &visible_candles,
            self.core.model.time_scale,
            candles_scale,
            self.core.model.viewport,
            candle_body_width,
        )?;
        let mut prev_wick_edge: Option<i64> = None;
        let mut prev_border_edge: Option<i64> = None;
        for (candle, source_index) in candle_geometries
            .into_iter()
            .zip(visible_candle_indices.into_iter())
        {
            let style_override = self
                .core
                .model
                .candle_style_overrides
                .get(source_index)
                .copied()
                .flatten();
            let fallback_body_color = if candle.is_bullish {
                style.candlestick_up_color
            } else {
                style.candlestick_down_color
            };
            let body_color = style_override
                .and_then(|entry| entry.color)
                .unwrap_or(fallback_body_color);
            let fallback_wick_color = if candle.is_bullish {
                style.candlestick_wick_up_color
            } else {
                style.candlestick_wick_down_color
            };
            let wick_color = style_override
                .and_then(|entry| entry.wick_color)
                .unwrap_or(fallback_wick_color);
            let fallback_border_color = if candle.is_bullish {
                style.candlestick_border_up_color
            } else {
                style.candlestick_border_down_color
            };
            let border_color = style_override
                .and_then(|entry| entry.border_color)
                .unwrap_or(fallback_border_color);
            let body_fill_color = match style.candlestick_body_mode {
                CandlestickBodyMode::Solid => body_color,
                CandlestickBodyMode::HollowUp if candle.is_bullish => {
                    Color::rgba(body_color.red, body_color.green, body_color.blue, 0.0)
                }
                CandlestickBodyMode::HollowUp => body_color,
            };

            if style.show_candlestick_wicks {
                let (wick_left_px, wick_right_px, wick_draw_width) =
                    Self::resolve_lwc_horizontal_draw_bounds(
                        candle.center_x,
                        wick_width,
                        prev_wick_edge,
                    );
                let wick_center_x = wick_left_px as f64 + (wick_draw_width as f64 - 1.0) * 0.5;
                let line = LinePrimitive::new(
                    wick_center_x,
                    candle.wick_top,
                    wick_center_x,
                    candle.wick_bottom,
                    wick_draw_width as f64,
                    wick_color,
                );
                frame.lines.push(line);
                layered.push_line(candles_pane_id, CanvasLayerKind::Series, line);
                prev_wick_edge = Some(wick_right_px);
            }

            let rect_fill_color = if render_border_only_body {
                border_color
            } else {
                body_fill_color
            };
            let (body_left_px, _body_right_px, body_draw_width) = if style.show_candlestick_borders
            {
                let bounds = Self::resolve_lwc_horizontal_draw_bounds(
                    candle.center_x,
                    candle_body_width,
                    prev_border_edge,
                );
                prev_border_edge = Some(bounds.1);
                bounds
            } else {
                Self::resolve_lwc_horizontal_draw_bounds(candle.center_x, candle_body_width, None)
            };
            let mut body = RectPrimitive::new(
                body_left_px as f64,
                candle.body_top.min(candle.body_bottom),
                body_draw_width as f64,
                (candle.body_bottom - candle.body_top).abs().max(1.0),
                rect_fill_color,
            );
            if !render_border_only_body && style.show_candlestick_borders && border_width > 0.0 {
                body = body.with_border(border_width, border_color);
            }
            frame.rects.push(body);
            layered.push_rect(candles_pane_id, CanvasLayerKind::Series, body);
        }

        Ok(())
    }

    fn resolve_candlestick_body_width_px(
        &self,
        visible_candles: &[OhlcBar],
        plot_width_px: f64,
    ) -> f64 {
        let bar_spacing_px =
            self.resolve_candlestick_bar_spacing_px(visible_candles, plot_width_px);
        let mut body_width = Self::lwc_optimal_candlestick_width_px(bar_spacing_px, 1.0);
        if body_width >= 2.0 {
            // Lightweight keeps candlestick body parity aligned with 1px wick/grid
            // width to preserve symmetric crosshair overlap.
            let wick_width = 1_i64;
            let body_width_i64 = body_width.floor() as i64;
            if (wick_width % 2) != (body_width_i64 % 2) {
                body_width = (body_width - 1.0).max(1.0);
            }
        }
        body_width
    }

    fn resolve_candlestick_bar_spacing_px(
        &self,
        visible_candles: &[OhlcBar],
        plot_width_px: f64,
    ) -> f64 {
        let plot_width_px = plot_width_px.max(1.0);
        if let Some(reference_step) = self.resolve_reference_time_step() {
            if let Ok((bar_spacing_px, _)) = self
                .core
                .model
                .time_scale
                .derive_visible_bar_spacing_and_right_offset(reference_step, plot_width_px)
            {
                if bar_spacing_px.is_finite() && bar_spacing_px > 0.0 {
                    return bar_spacing_px;
                }
            }
        }

        if visible_candles.len() >= 2 {
            let mut deltas = Vec::with_capacity(visible_candles.len().saturating_sub(1));
            for pair in visible_candles.windows(2) {
                let left_x = match self
                    .core
                    .model
                    .time_scale
                    .time_to_pixel(pair[0].time, self.core.model.viewport)
                {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let right_x = match self
                    .core
                    .model
                    .time_scale
                    .time_to_pixel(pair[1].time, self.core.model.viewport)
                {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let dx = (right_x - left_x).abs();
                if dx.is_finite() && dx > 0.0 {
                    deltas.push(dx);
                }
            }
            if !deltas.is_empty() {
                deltas.sort_by(|left, right| left.total_cmp(right));
                let mid = deltas.len() / 2;
                if deltas.len() % 2 == 1 {
                    return deltas[mid];
                }
                return (deltas[mid - 1] + deltas[mid]) * 0.5;
            }
        }

        // Lightweight default time-scale spacing fallback.
        6.0
    }

    fn resolve_reference_time_step(&self) -> Option<f64> {
        if let Some(step) =
            Self::estimate_positive_time_step(self.core.model.candles.iter().map(|bar| bar.time))
        {
            return Some(step);
        }
        Self::estimate_positive_time_step(self.core.model.points.iter().map(|point| point.x))
    }

    fn estimate_positive_time_step<I>(times: I) -> Option<f64>
    where
        I: IntoIterator<Item = f64>,
    {
        let mut ordered = times
            .into_iter()
            .filter(|value| value.is_finite())
            .collect::<Vec<_>>();
        if ordered.len() < 2 {
            return None;
        }

        ordered.sort_by(|left, right| left.total_cmp(right));

        let mut deltas = Vec::with_capacity(ordered.len().saturating_sub(1));
        for window in ordered.windows(2) {
            let delta = window[1] - window[0];
            if delta.is_finite() && delta > 0.0 {
                deltas.push(delta);
            }
        }

        if !deltas.is_empty() {
            deltas.sort_by(|left, right| left.total_cmp(right));
            let mid = deltas.len() / 2;
            if deltas.len() % 2 == 1 {
                return Some(deltas[mid]);
            }
            return Some((deltas[mid - 1] + deltas[mid]) * 0.5);
        }

        let span = ordered.last().copied().unwrap_or(0.0) - ordered.first().copied().unwrap_or(0.0);
        if span > 0.0 {
            let count = ordered.len().saturating_sub(1) as f64;
            return Some(span / count.max(1.0));
        }

        None
    }

    fn visible_candle_indices_in_time_window(&self, start: f64, end: f64) -> Vec<usize> {
        self.core
            .model
            .candles
            .iter()
            .enumerate()
            .filter_map(|(idx, candle)| {
                if candle.time >= start && candle.time <= end {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    fn lwc_optimal_candlestick_width_px(bar_spacing_px: f64, pixel_ratio: f64) -> f64 {
        let min_width = pixel_ratio.floor().max(1.0);
        if !bar_spacing_px.is_finite() || bar_spacing_px <= 0.0 || !pixel_ratio.is_finite() {
            return min_width;
        }

        let special_from = 2.5;
        let special_to = 4.0;
        let special_coeff = 3.0;
        if bar_spacing_px >= special_from && bar_spacing_px <= special_to {
            return (special_coeff * pixel_ratio).floor().max(min_width);
        }

        let reducing_coeff = 0.2;
        let coeff = 1.0
            - reducing_coeff * (bar_spacing_px.max(special_to) - special_to).atan()
                / (std::f64::consts::PI * 0.5);
        let res = (bar_spacing_px * coeff * pixel_ratio).floor();
        let scaled_spacing = (bar_spacing_px * pixel_ratio).floor();
        let optimal = res.min(scaled_spacing);
        optimal.max(min_width)
    }

    fn resolve_effective_candlestick_wick_width_px(
        requested_width_px: f64,
        bar_spacing_px: f64,
        body_width_px: f64,
        pixel_ratio: f64,
    ) -> f64 {
        let min_width = pixel_ratio.floor().max(1.0);
        if !requested_width_px.is_finite() || requested_width_px <= 0.0 {
            return min_width;
        }
        let spacing_cap = (bar_spacing_px * pixel_ratio).floor().max(min_width);
        let body_cap = body_width_px.floor().max(min_width);
        requested_width_px
            .min(spacing_cap)
            .min(body_cap)
            .max(min_width)
    }

    fn resolve_effective_candlestick_border_width_px(
        requested_width_px: f64,
        body_width_px: f64,
        pixel_ratio: f64,
    ) -> f64 {
        if !requested_width_px.is_finite() || requested_width_px <= 0.0 {
            return 0.0;
        }

        let min_width = pixel_ratio.floor().max(1.0);
        let default_border_width = requested_width_px.max(min_width);

        let mut border_width = default_border_width;
        if body_width_px <= 2.0 * border_width {
            border_width = ((body_width_px - 1.0) * 0.5).floor().max(0.0);
        }
        let constrained_border_width = border_width.max(min_width);

        if body_width_px <= constrained_border_width * 2.0 {
            return default_border_width;
        }
        constrained_border_width
    }

    fn resolve_lwc_horizontal_draw_bounds(
        center_x: f64,
        nominal_width_px: f64,
        prev_edge_inclusive_px: Option<i64>,
    ) -> (i64, i64, i64) {
        let width_px = nominal_width_px.floor().max(1.0) as i64;
        let mut left = center_x.round() as i64 - ((width_px as f64 * 0.5).floor() as i64);
        let right = left + width_px - 1;
        if let Some(prev_edge) = prev_edge_inclusive_px {
            left = left.max(prev_edge + 1);
            left = left.min(right);
        }
        let adjusted_width = (right - left + 1).max(1);
        (left, right, adjusted_width)
    }
}
