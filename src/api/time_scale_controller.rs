use crate::core::TimeScaleTuning;
use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::{ChartEngine, PluginEvent};

impl<R: Renderer> ChartEngine<R> {
    /// Overrides visible time range (zoom/pan style behavior).
    pub fn set_time_visible_range(&mut self, start: f64, end: f64) -> ChartResult<()> {
        self.time_scale.set_visible_range(start, end)?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Resets visible range to fitted full range.
    pub fn reset_time_visible_range(&mut self) {
        self.time_scale.reset_visible_range_to_full();
        self.emit_visible_range_changed();
    }

    /// Pans visible range by explicit time delta.
    pub fn pan_time_visible_by(&mut self, delta_time: f64) -> ChartResult<()> {
        self.time_scale.pan_visible_by_delta(delta_time)?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Pans visible range using pixel drag delta.
    ///
    /// Positive `delta_px` moves the range to earlier times, matching common
    /// drag-to-scroll chart behavior.
    pub fn pan_time_visible_by_pixels(&mut self, delta_px: f64) -> ChartResult<()> {
        if !delta_px.is_finite() {
            return Err(ChartError::InvalidData(
                "pan pixel delta must be finite".to_owned(),
            ));
        }

        let (start, end) = self.time_scale.visible_range();
        let span = end - start;
        let delta_time = -(delta_px / f64::from(self.viewport.width)) * span;
        self.time_scale.pan_visible_by_delta(delta_time)?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Applies wheel-driven horizontal pan.
    ///
    /// Conventions:
    /// - one wheel notch is normalized as `120` units
    /// - `wheel_delta_x > 0` pans to later times
    ///
    /// Returns the applied time displacement.
    pub fn wheel_pan_time_visible(
        &mut self,
        wheel_delta_x: f64,
        pan_step_ratio: f64,
    ) -> ChartResult<f64> {
        if !wheel_delta_x.is_finite() {
            return Err(ChartError::InvalidData(
                "wheel pan delta must be finite".to_owned(),
            ));
        }
        if !pan_step_ratio.is_finite() || pan_step_ratio <= 0.0 {
            return Err(ChartError::InvalidData(
                "wheel pan step ratio must be finite and > 0".to_owned(),
            ));
        }
        if wheel_delta_x == 0.0 {
            return Ok(0.0);
        }

        let (start, end) = self.time_scale.visible_range();
        let span = end - start;
        let normalized_steps = wheel_delta_x / 120.0;
        let delta_time = normalized_steps * span * pan_step_ratio;
        self.pan_time_visible_by(delta_time)?;
        Ok(delta_time)
    }

    /// Zooms visible range around a logical time anchor.
    pub fn zoom_time_visible_around_time(
        &mut self,
        factor: f64,
        anchor_time: f64,
        min_span_absolute: f64,
    ) -> ChartResult<()> {
        self.time_scale
            .zoom_visible_by_factor(factor, anchor_time, min_span_absolute)?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Zooms visible range around a pixel anchor.
    pub fn zoom_time_visible_around_pixel(
        &mut self,
        factor: f64,
        anchor_px: f64,
        min_span_absolute: f64,
    ) -> ChartResult<()> {
        let anchor_time = self.map_pixel_to_x(anchor_px)?;
        self.time_scale
            .zoom_visible_by_factor(factor, anchor_time, min_span_absolute)?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Applies wheel-driven zoom around a pixel anchor.
    ///
    /// Conventions:
    /// - `wheel_delta_y < 0` zooms in
    /// - `wheel_delta_y > 0` zooms out
    /// - one wheel notch is normalized as `120` units
    ///
    /// Returns the effective zoom factor applied to the visible range.
    pub fn wheel_zoom_time_visible(
        &mut self,
        wheel_delta_y: f64,
        anchor_px: f64,
        zoom_step_ratio: f64,
        min_span_absolute: f64,
    ) -> ChartResult<f64> {
        if !wheel_delta_y.is_finite() {
            return Err(ChartError::InvalidData(
                "wheel delta must be finite".to_owned(),
            ));
        }
        if !zoom_step_ratio.is_finite() || zoom_step_ratio <= 0.0 {
            return Err(ChartError::InvalidData(
                "wheel zoom step ratio must be finite and > 0".to_owned(),
            ));
        }
        if wheel_delta_y == 0.0 {
            return Ok(1.0);
        }

        let normalized_steps = wheel_delta_y / 120.0;
        let base = 1.0 + zoom_step_ratio;
        let factor = base.powf(-normalized_steps);
        if !factor.is_finite() || factor <= 0.0 {
            return Err(ChartError::InvalidData(
                "computed wheel zoom factor must be finite and > 0".to_owned(),
            ));
        }

        self.zoom_time_visible_around_pixel(factor, anchor_px, min_span_absolute)?;
        Ok(factor)
    }

    /// Advances kinetic pan by a deterministic simulation step.
    ///
    /// Returns `true` when a displacement was applied.
    pub fn step_kinetic_pan(&mut self, delta_seconds: f64) -> ChartResult<bool> {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return Err(ChartError::InvalidData(
                "kinetic pan delta seconds must be finite and > 0".to_owned(),
            ));
        }

        let was_active = self.interaction.kinetic_pan_state().active;
        let Some(displacement) = self.interaction.step_kinetic_pan(delta_seconds) else {
            return Ok(false);
        };

        self.pan_time_visible_by(displacement)?;

        if was_active && !self.interaction.kinetic_pan_state().active {
            self.emit_plugin_event(PluginEvent::PanEnded);
        }
        Ok(true)
    }

    /// Fits time scale against available point/candle data.
    pub fn fit_time_to_data(&mut self, tuning: TimeScaleTuning) -> ChartResult<()> {
        if self.points.is_empty() && self.candles.is_empty() {
            return Ok(());
        }

        self.time_scale
            .fit_to_mixed_data(&self.points, &self.candles, tuning)?;
        self.emit_visible_range_changed();
        Ok(())
    }
}
