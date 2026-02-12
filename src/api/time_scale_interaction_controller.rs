use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::ChartEngine;
use super::layout_helpers::resolve_axis_layout;

impl<R: Renderer> ChartEngine<R> {
    /// Applies time-axis drag scaling around a pixel anchor.
    ///
    /// Conventions:
    /// - `drag_delta_x_px < 0` zooms out (larger visible time span)
    /// - `drag_delta_x_px > 0` zooms in (smaller visible time span)
    /// - one drag notch is normalized as `120` pixels
    ///
    /// Returns the effective zoom factor applied to visible span.
    pub fn axis_drag_scale_time(
        &mut self,
        drag_delta_x_px: f64,
        anchor_x_px: f64,
        zoom_step_ratio: f64,
        min_span_absolute: f64,
    ) -> ChartResult<f64> {
        if !self.interaction_input_behavior.allows_axis_drag_scale() {
            return Ok(1.0);
        }

        if !drag_delta_x_px.is_finite() {
            return Err(ChartError::InvalidData(
                "axis drag delta must be finite".to_owned(),
            ));
        }
        if !anchor_x_px.is_finite() {
            return Err(ChartError::InvalidData(
                "axis drag anchor x must be finite".to_owned(),
            ));
        }
        if !zoom_step_ratio.is_finite() || zoom_step_ratio <= 0.0 {
            return Err(ChartError::InvalidData(
                "axis drag zoom step ratio must be finite and > 0".to_owned(),
            ));
        }
        if !min_span_absolute.is_finite() || min_span_absolute <= 0.0 {
            return Err(ChartError::InvalidData(
                "axis drag minimum span must be finite and > 0".to_owned(),
            ));
        }
        if drag_delta_x_px == 0.0 {
            return Ok(1.0);
        }

        let normalized_steps = drag_delta_x_px / 120.0;
        let base = 1.0 + zoom_step_ratio;
        let factor = base.powf(normalized_steps);
        if !factor.is_finite() || factor <= 0.0 {
            return Err(ChartError::InvalidData(
                "computed axis drag zoom factor must be finite and > 0".to_owned(),
            ));
        }

        if self
            .time_scale_scroll_zoom_behavior
            .right_bar_stays_on_scroll
        {
            let (_, right_edge) = self.time_scale.visible_range();
            self.zoom_time_visible_around_time(factor, right_edge, min_span_absolute)?;
        } else {
            let viewport_width = f64::from(self.viewport.width);
            let viewport_height = f64::from(self.viewport.height);
            let layout = resolve_axis_layout(
                viewport_width,
                viewport_height,
                self.render_style.price_axis_width_px,
                self.render_style.time_axis_height_px,
            );
            let anchor_x = anchor_x_px.clamp(0.0, layout.plot_right);
            self.zoom_time_visible_around_pixel(factor, anchor_x, min_span_absolute)?;
        }
        Ok(factor)
    }
}
