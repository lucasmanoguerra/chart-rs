use crate::error::ChartResult;
use crate::render::Renderer;

use super::{ChartEngine, scale_coordinator::ScaleCoordinator};

impl<R: Renderer> ChartEngine<R> {
    /// Applies vertical panning on the price axis around a pixel anchor.
    ///
    /// Conventions:
    /// - `drag_delta_y_px > 0` shifts the price domain upward on screen
    /// - `drag_delta_y_px < 0` shifts the price domain downward on screen
    ///
    /// Returns `true` when domain changed.
    pub fn axis_drag_pan_price(
        &mut self,
        drag_delta_y_px: f64,
        anchor_y_px: f64,
    ) -> ChartResult<bool> {
        ScaleCoordinator::axis_drag_pan_price(self, drag_delta_y_px, anchor_y_px)
    }

    /// Applies price-axis drag scaling around a pixel anchor.
    ///
    /// Conventions:
    /// - `drag_delta_y_px < 0` zooms in (smaller visible price span)
    /// - `drag_delta_y_px > 0` zooms out (larger visible price span)
    /// - one drag notch is normalized as `120` pixels
    ///
    /// Returns the effective zoom factor applied to price span.
    pub fn axis_drag_scale_price(
        &mut self,
        drag_delta_y_px: f64,
        anchor_y_px: f64,
        zoom_step_ratio: f64,
        min_span_absolute: f64,
    ) -> ChartResult<f64> {
        ScaleCoordinator::axis_drag_scale_price(
            self,
            drag_delta_y_px,
            anchor_y_px,
            zoom_step_ratio,
            min_span_absolute,
        )
    }

    /// Resets price axis to data-driven autoscale domain.
    ///
    /// This mirrors axis double-click reset semantics. Candles have priority
    /// when both candle and point data are present.
    ///
    /// Returns `true` when price domain changed.
    pub fn axis_double_click_reset_price_scale(&mut self) -> ChartResult<bool> {
        ScaleCoordinator::axis_double_click_reset_price_scale(self)
    }
}
