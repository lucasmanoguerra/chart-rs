use crate::core::TimeScaleTuning;
use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::{
    ChartEngine, TimeScaleEdgeBehavior, TimeScaleNavigationBehavior,
    TimeScaleRealtimeAppendBehavior, TimeScaleResizeBehavior, TimeScaleScrollZoomBehavior,
    TimeScaleZoomLimitBehavior, interaction_coordinator::InteractionCoordinator,
    time_scale_coordinator::TimeScaleCoordinator, time_scale_validation,
};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn time_scale_edge_behavior(&self) -> TimeScaleEdgeBehavior {
        self.core.behavior.time_scale_edge_behavior
    }

    pub fn set_time_scale_edge_behavior(
        &mut self,
        behavior: TimeScaleEdgeBehavior,
    ) -> ChartResult<()> {
        self.core.behavior.time_scale_edge_behavior = behavior;
        if self.apply_time_scale_constraints()? {
            self.emit_visible_range_changed();
        }
        Ok(())
    }

    #[must_use]
    pub fn time_scale_navigation_behavior(&self) -> TimeScaleNavigationBehavior {
        self.core.behavior.time_scale_navigation_behavior
    }

    pub fn set_time_scale_navigation_behavior(
        &mut self,
        behavior: TimeScaleNavigationBehavior,
    ) -> ChartResult<()> {
        time_scale_validation::validate_time_scale_navigation_behavior(behavior)?;
        self.core.behavior.time_scale_navigation_behavior = behavior;
        if self.apply_time_scale_constraints()? {
            self.emit_visible_range_changed();
        }
        Ok(())
    }

    #[must_use]
    pub fn time_scale_zoom_limit_behavior(&self) -> TimeScaleZoomLimitBehavior {
        self.core.behavior.time_scale_zoom_limit_behavior
    }

    pub fn set_time_scale_zoom_limit_behavior(
        &mut self,
        behavior: TimeScaleZoomLimitBehavior,
    ) -> ChartResult<()> {
        time_scale_validation::validate_time_scale_zoom_limit_behavior(behavior)?;
        self.core.behavior.time_scale_zoom_limit_behavior = behavior;
        if self.apply_time_scale_constraints()? {
            self.emit_visible_range_changed();
        }
        Ok(())
    }

    #[must_use]
    pub fn time_scale_right_offset_px(&self) -> Option<f64> {
        self.core.behavior.time_scale_right_offset_px
    }

    pub fn set_time_scale_right_offset_px(
        &mut self,
        right_offset_px: Option<f64>,
    ) -> ChartResult<()> {
        if let Some(px) = right_offset_px {
            if !px.is_finite() || px < 0.0 {
                return Err(ChartError::InvalidData(
                    "time scale right offset px must be finite and >= 0".to_owned(),
                ));
            }
        }
        self.core.behavior.time_scale_right_offset_px = right_offset_px;
        if self.apply_time_scale_constraints()? {
            self.emit_visible_range_changed();
        }
        Ok(())
    }

    #[must_use]
    pub fn time_scale_scroll_zoom_behavior(&self) -> TimeScaleScrollZoomBehavior {
        self.core.behavior.time_scale_scroll_zoom_behavior
    }

    pub fn set_time_scale_scroll_zoom_behavior(
        &mut self,
        behavior: TimeScaleScrollZoomBehavior,
    ) -> ChartResult<()> {
        self.core.behavior.time_scale_scroll_zoom_behavior = behavior;
        Ok(())
    }

    #[must_use]
    pub fn time_scale_resize_behavior(&self) -> TimeScaleResizeBehavior {
        self.core.behavior.time_scale_resize_behavior
    }

    pub fn set_time_scale_resize_behavior(
        &mut self,
        behavior: TimeScaleResizeBehavior,
    ) -> ChartResult<()> {
        self.core.behavior.time_scale_resize_behavior = behavior;
        Ok(())
    }

    #[must_use]
    pub fn time_scale_realtime_append_behavior(&self) -> TimeScaleRealtimeAppendBehavior {
        self.core.behavior.time_scale_realtime_append_behavior
    }

    pub fn set_time_scale_realtime_append_behavior(
        &mut self,
        behavior: TimeScaleRealtimeAppendBehavior,
    ) -> ChartResult<()> {
        time_scale_validation::validate_time_scale_realtime_append_behavior(behavior)?;
        self.core.behavior.time_scale_realtime_append_behavior = behavior;
        Ok(())
    }

    /// Overrides visible time range (zoom/pan style behavior).
    pub fn set_time_visible_range(&mut self, start: f64, end: f64) -> ChartResult<()> {
        self.core.model.time_scale.set_visible_range(start, end)?;
        // Explicit visible-range requests should preserve host intent. Apply
        // only hard constraints (zoom limits + edge clamps), but do not
        // re-synthesize navigation offset/spacing here.
        let _ = self.apply_time_scale_zoom_limit_behavior()?;
        let _ = self.apply_time_scale_edge_behavior()?;
        self.set_lwc_time_scale_invalidation_intent(
            super::chart_runtime::LwcTimeScaleInvalidationIntent::ApplyRange,
        );
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Resets visible range to fitted full range.
    pub fn reset_time_visible_range(&mut self) {
        self.core.model.time_scale.reset_visible_range_to_full();
        self.set_lwc_time_scale_invalidation_intent(
            super::chart_runtime::LwcTimeScaleInvalidationIntent::Reset,
        );
        self.emit_visible_range_changed();
    }

    /// Scrolls the visible range to the realtime edge.
    ///
    /// This mirrors Lightweight Charts `timeScale().scrollToRealTime()` style
    /// behavior: when right-offset/spacing navigation policies are active they
    /// are reapplied; otherwise current visible span is preserved and shifted so
    /// that the right edge lands on the latest data boundary.
    ///
    /// Returns `true` when visible range changed.
    pub fn scroll_time_to_realtime(&mut self) -> ChartResult<bool> {
        TimeScaleCoordinator::scroll_time_to_realtime(self)
    }

    /// Returns current scroll position in reference bars, measured as
    /// distance from the latest data bar to the visible right edge.
    ///
    /// Positive values mean extra right whitespace; negative values mean the
    /// viewport is lagging behind realtime.
    ///
    /// Returns `None` when the engine cannot resolve a reference bar step.
    #[must_use]
    pub fn time_scroll_position_bars(&self) -> Option<f64> {
        TimeScaleCoordinator::time_scroll_position_bars(self)
    }

    /// Scrolls visible range to an explicit position in reference bars.
    ///
    /// The method preserves current visible span and shifts the window so that
    /// right edge lands on `full_end + position_bars * step`.
    ///
    /// Returns `true` when visible range changed.
    pub fn scroll_time_to_position_bars(&mut self, position_bars: f64) -> ChartResult<bool> {
        TimeScaleCoordinator::scroll_time_to_position_bars(self, position_bars)
    }

    /// Pans visible range by explicit time delta.
    pub fn pan_time_visible_by(&mut self, delta_time: f64) -> ChartResult<()> {
        TimeScaleCoordinator::pan_time_visible_by(self, delta_time)
    }

    /// Pans visible range using pixel drag delta.
    ///
    /// Positive `delta_px` moves the range to earlier times, matching common
    /// drag-to-scroll chart behavior.
    pub fn pan_time_visible_by_pixels(&mut self, delta_px: f64) -> ChartResult<()> {
        TimeScaleCoordinator::pan_time_visible_by_pixels(self, delta_px)
    }

    /// Applies touch-drag driven pan using horizontal and/or vertical movement.
    ///
    /// The driving axis is selected from enabled touch-drag gates:
    /// - only horizontal enabled -> `delta_x_px`
    /// - only vertical enabled -> `delta_y_px`
    /// - both enabled -> dominant absolute component
    ///
    /// Returns the applied time displacement.
    pub fn touch_drag_pan_time_visible(
        &mut self,
        delta_x_px: f64,
        delta_y_px: f64,
    ) -> ChartResult<f64> {
        TimeScaleCoordinator::touch_drag_pan_time_visible(self, delta_x_px, delta_y_px)
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
        TimeScaleCoordinator::wheel_pan_time_visible(self, wheel_delta_x, pan_step_ratio)
    }

    /// Zooms visible range around a logical time anchor.
    pub fn zoom_time_visible_around_time(
        &mut self,
        factor: f64,
        anchor_time: f64,
        min_span_absolute: f64,
    ) -> ChartResult<()> {
        TimeScaleCoordinator::zoom_time_visible_around_time(
            self,
            factor,
            anchor_time,
            min_span_absolute,
        )
    }

    /// Zooms visible range around a pixel anchor.
    pub fn zoom_time_visible_around_pixel(
        &mut self,
        factor: f64,
        anchor_px: f64,
        min_span_absolute: f64,
    ) -> ChartResult<()> {
        TimeScaleCoordinator::zoom_time_visible_around_pixel(
            self,
            factor,
            anchor_px,
            min_span_absolute,
        )
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
        TimeScaleCoordinator::wheel_zoom_time_visible(
            self,
            wheel_delta_y,
            anchor_px,
            zoom_step_ratio,
            min_span_absolute,
        )
    }

    /// Applies pinch-driven zoom around a pixel anchor.
    ///
    /// When pinch-scale interaction is disabled this is a deterministic
    /// no-op returning `1.0`.
    pub fn pinch_zoom_time_visible(
        &mut self,
        factor: f64,
        anchor_px: f64,
        min_span_absolute: f64,
    ) -> ChartResult<f64> {
        TimeScaleCoordinator::pinch_zoom_time_visible(self, factor, anchor_px, min_span_absolute)
    }

    /// Advances kinetic pan by a deterministic simulation step.
    ///
    /// Returns `true` when a displacement was applied.
    pub fn step_kinetic_pan(&mut self, delta_seconds: f64) -> ChartResult<bool> {
        InteractionCoordinator::step_kinetic_pan(self, delta_seconds)
    }

    /// Fits time scale against available point/candle data.
    pub fn fit_time_to_data(&mut self, tuning: TimeScaleTuning) -> ChartResult<()> {
        TimeScaleCoordinator::fit_time_to_data(self, tuning)
    }

    pub(crate) fn apply_time_scale_constraints(&mut self) -> ChartResult<bool> {
        TimeScaleCoordinator::apply_time_scale_constraints(self)
    }

    pub(crate) fn apply_time_scale_edge_behavior(&mut self) -> ChartResult<bool> {
        TimeScaleCoordinator::apply_time_scale_edge_behavior(self)
    }

    pub(crate) fn apply_time_scale_zoom_limit_behavior(&mut self) -> ChartResult<bool> {
        TimeScaleCoordinator::apply_time_scale_zoom_limit_behavior(self)
    }

    pub(crate) fn apply_time_scale_resize_behavior(
        &mut self,
        previous_viewport_width_px: u32,
    ) -> ChartResult<bool> {
        TimeScaleCoordinator::apply_time_scale_resize_behavior(self, previous_viewport_width_px)
    }

    pub(crate) fn handle_realtime_time_append(&mut self, appended_time: f64) -> bool {
        TimeScaleCoordinator::handle_realtime_time_append(self, appended_time)
    }

    pub(crate) fn resolve_right_margin_zoom_anchor_px(&self) -> Option<f64> {
        TimeScaleCoordinator::resolve_right_margin_zoom_anchor_px(self)
    }

    pub(crate) fn resolve_time_index_coordinate_space(
        &self,
    ) -> Option<(crate::core::TimeIndexCoordinateSpace, f64)> {
        TimeScaleCoordinator::resolve_time_index_coordinate_space(self)
    }
}
