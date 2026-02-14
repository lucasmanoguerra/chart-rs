use crate::error::ChartResult;
use crate::render::Renderer;

use super::{ChartEngine, scale_coordinator::ScaleCoordinator};

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
        ScaleCoordinator::axis_drag_scale_time(
            self,
            drag_delta_x_px,
            anchor_x_px,
            zoom_step_ratio,
            min_span_absolute,
        )
    }

    /// Resets time axis to full-range visible domain.
    ///
    /// This mirrors axis double-click reset semantics for time scale.
    ///
    /// Returns `true` when visible range changed.
    pub fn axis_double_click_reset_time_scale(&mut self) -> ChartResult<bool> {
        ScaleCoordinator::axis_double_click_reset_time_scale(self)
    }
}
