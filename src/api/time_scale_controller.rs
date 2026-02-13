use crate::core::{DataPoint, OhlcBar, TimeIndexCoordinateSpace, TimeScaleTuning};
use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::{
    ChartEngine, PluginEvent, TimeScaleEdgeBehavior, TimeScaleNavigationBehavior,
    TimeScaleRealtimeAppendBehavior, TimeScaleResizeAnchor, TimeScaleResizeBehavior,
    TimeScaleScrollZoomBehavior, TimeScaleZoomLimitBehavior,
};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn time_scale_edge_behavior(&self) -> TimeScaleEdgeBehavior {
        self.time_scale_edge_behavior
    }

    pub fn set_time_scale_edge_behavior(
        &mut self,
        behavior: TimeScaleEdgeBehavior,
    ) -> ChartResult<()> {
        self.time_scale_edge_behavior = behavior;
        if self.apply_time_scale_constraints()? {
            self.emit_visible_range_changed();
        }
        Ok(())
    }

    #[must_use]
    pub fn time_scale_navigation_behavior(&self) -> TimeScaleNavigationBehavior {
        self.time_scale_navigation_behavior
    }

    pub fn set_time_scale_navigation_behavior(
        &mut self,
        behavior: TimeScaleNavigationBehavior,
    ) -> ChartResult<()> {
        validate_time_scale_navigation_behavior(behavior)?;
        self.time_scale_navigation_behavior = behavior;
        if self.apply_time_scale_constraints()? {
            self.emit_visible_range_changed();
        }
        Ok(())
    }

    #[must_use]
    pub fn time_scale_zoom_limit_behavior(&self) -> TimeScaleZoomLimitBehavior {
        self.time_scale_zoom_limit_behavior
    }

    pub fn set_time_scale_zoom_limit_behavior(
        &mut self,
        behavior: TimeScaleZoomLimitBehavior,
    ) -> ChartResult<()> {
        validate_time_scale_zoom_limit_behavior(behavior)?;
        self.time_scale_zoom_limit_behavior = behavior;
        if self.apply_time_scale_constraints()? {
            self.emit_visible_range_changed();
        }
        Ok(())
    }

    #[must_use]
    pub fn time_scale_right_offset_px(&self) -> Option<f64> {
        self.time_scale_right_offset_px
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
        self.time_scale_right_offset_px = right_offset_px;
        if self.apply_time_scale_constraints()? {
            self.emit_visible_range_changed();
        }
        Ok(())
    }

    #[must_use]
    pub fn time_scale_scroll_zoom_behavior(&self) -> TimeScaleScrollZoomBehavior {
        self.time_scale_scroll_zoom_behavior
    }

    pub fn set_time_scale_scroll_zoom_behavior(
        &mut self,
        behavior: TimeScaleScrollZoomBehavior,
    ) -> ChartResult<()> {
        self.time_scale_scroll_zoom_behavior = behavior;
        Ok(())
    }

    #[must_use]
    pub fn time_scale_resize_behavior(&self) -> TimeScaleResizeBehavior {
        self.time_scale_resize_behavior
    }

    pub fn set_time_scale_resize_behavior(
        &mut self,
        behavior: TimeScaleResizeBehavior,
    ) -> ChartResult<()> {
        self.time_scale_resize_behavior = behavior;
        Ok(())
    }

    #[must_use]
    pub fn time_scale_realtime_append_behavior(&self) -> TimeScaleRealtimeAppendBehavior {
        self.time_scale_realtime_append_behavior
    }

    pub fn set_time_scale_realtime_append_behavior(
        &mut self,
        behavior: TimeScaleRealtimeAppendBehavior,
    ) -> ChartResult<()> {
        validate_time_scale_realtime_append_behavior(behavior)?;
        self.time_scale_realtime_append_behavior = behavior;
        Ok(())
    }

    /// Overrides visible time range (zoom/pan style behavior).
    pub fn set_time_visible_range(&mut self, start: f64, end: f64) -> ChartResult<()> {
        self.time_scale.set_visible_range(start, end)?;
        // Explicit visible-range requests should preserve host intent. Apply
        // only hard constraints (zoom limits + edge clamps), but do not
        // re-synthesize navigation offset/spacing here.
        let _ = self.apply_time_scale_zoom_limit_behavior()?;
        let _ = self.apply_time_scale_edge_behavior()?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Resets visible range to fitted full range.
    pub fn reset_time_visible_range(&mut self) {
        self.time_scale.reset_visible_range_to_full();
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
        let navigation_active = self.time_scale_navigation_behavior.right_offset_bars != 0.0
            || self.time_scale_navigation_behavior.bar_spacing_px.is_some()
            || self.time_scale_right_offset_px.is_some();

        let mut changed = if navigation_active {
            self.apply_time_scale_constraints()?
        } else {
            let (start, end) = self.time_scale.visible_range();
            let (_, full_end) = self.time_scale.full_range();
            let reference_step = resolve_reference_time_step(&self.points, &self.candles);
            let visible_span = (end - start).max(1e-9);
            let target_end = resolve_navigation_target_end(
                full_end,
                self.time_scale_navigation_behavior.right_offset_bars,
                self.time_scale_right_offset_px,
                reference_step,
                visible_span,
                f64::from(self.viewport.width),
            );
            let delta = target_end - end;
            if delta.abs() > 1e-12 {
                self.time_scale
                    .set_visible_range(start + delta, end + delta)?;
                true
            } else {
                false
            }
        };

        if self.apply_time_scale_edge_behavior()? {
            changed = true;
        }

        if changed {
            self.emit_visible_range_changed();
        }
        Ok(changed)
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
        let (_, full_end) = self.time_scale.full_range();
        let (_, visible_end) = self.time_scale.visible_range();
        let distance = visible_end - full_end;

        let step = resolve_reference_time_step(&self.points, &self.candles)?;
        if !step.is_finite() || step <= 0.0 {
            return None;
        }
        Some(distance / step)
    }

    /// Scrolls visible range to an explicit position in reference bars.
    ///
    /// The method preserves current visible span and shifts the window so that
    /// right edge lands on `full_end + position_bars * step`.
    ///
    /// Returns `true` when visible range changed.
    pub fn scroll_time_to_position_bars(&mut self, position_bars: f64) -> ChartResult<bool> {
        if !position_bars.is_finite() {
            return Err(ChartError::InvalidData(
                "scroll position bars must be finite".to_owned(),
            ));
        }

        let (_, full_end) = self.time_scale.full_range();
        let (_, end) = self.time_scale.visible_range();

        let target_end = if position_bars == 0.0 {
            full_end
        } else {
            let Some(step) = resolve_reference_time_step(&self.points, &self.candles) else {
                return Err(ChartError::InvalidData(
                    "cannot resolve scroll position without reference data step".to_owned(),
                ));
            };
            full_end + position_bars * step
        };

        let delta = target_end - end;
        let mut changed = false;
        if delta.abs() > 1e-12 {
            let (start, end) = self.time_scale.visible_range();
            self.time_scale
                .set_visible_range(start + delta, end + delta)?;
            changed = true;
        }

        if self.apply_time_scale_edge_behavior()? {
            changed = true;
        }
        if changed {
            self.emit_visible_range_changed();
        }
        Ok(changed)
    }

    /// Pans visible range by explicit time delta.
    pub fn pan_time_visible_by(&mut self, delta_time: f64) -> ChartResult<()> {
        self.time_scale.pan_visible_by_delta(delta_time)?;
        let _ = self.apply_time_scale_zoom_limit_behavior()?;
        let _ = self.apply_time_scale_edge_behavior()?;
        self.emit_visible_range_changed();
        Ok(())
    }

    /// Pans visible range using pixel drag delta.
    ///
    /// Positive `delta_px` moves the range to earlier times, matching common
    /// drag-to-scroll chart behavior.
    pub fn pan_time_visible_by_pixels(&mut self, delta_px: f64) -> ChartResult<()> {
        if !self.interaction_input_behavior.allows_drag_pan() {
            return Ok(());
        }

        if !delta_px.is_finite() {
            return Err(ChartError::InvalidData(
                "pan pixel delta must be finite".to_owned(),
            ));
        }

        if let Some((space, reference_step)) = self.resolve_time_index_coordinate_space() {
            let target_right_offset = space.pan_right_offset_by_pixels(-delta_px)?;
            self.time_scale
                .set_visible_range_from_bar_spacing_and_right_offset(
                    space.bar_spacing_px,
                    target_right_offset,
                    reference_step,
                    space.width_px,
                )?;
            let _ = self.apply_time_scale_zoom_limit_behavior()?;
            let _ = self.apply_time_scale_edge_behavior()?;
            self.emit_visible_range_changed();
            return Ok(());
        }

        let (start, end) = self.time_scale.visible_range();
        let span = end - start;
        let delta_time = -(delta_px / f64::from(self.viewport.width)) * span;
        self.pan_time_visible_by(delta_time)
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
        let behavior = self.interaction_input_behavior;
        if !behavior.handle_scroll
            || (!behavior.scroll_horz_touch_drag && !behavior.scroll_vert_touch_drag)
        {
            return Ok(0.0);
        }

        if behavior.scroll_horz_touch_drag && !delta_x_px.is_finite() {
            return Err(ChartError::InvalidData(
                "touch drag horizontal delta must be finite when horizontal touch pan is enabled"
                    .to_owned(),
            ));
        }
        if behavior.scroll_vert_touch_drag && !delta_y_px.is_finite() {
            return Err(ChartError::InvalidData(
                "touch drag vertical delta must be finite when vertical touch pan is enabled"
                    .to_owned(),
            ));
        }

        let (driving_px, driving_axis_span_px) = match (
            behavior.scroll_horz_touch_drag,
            behavior.scroll_vert_touch_drag,
        ) {
            (true, false) => (delta_x_px, f64::from(self.viewport.width)),
            (false, true) => (delta_y_px, f64::from(self.viewport.height)),
            (true, true) => {
                if delta_x_px.abs() >= delta_y_px.abs() {
                    (delta_x_px, f64::from(self.viewport.width))
                } else {
                    (delta_y_px, f64::from(self.viewport.height))
                }
            }
            (false, false) => (0.0, f64::from(self.viewport.width)),
        };

        if driving_px == 0.0 {
            return Ok(0.0);
        }
        if !driving_axis_span_px.is_finite() || driving_axis_span_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "touch drag driving axis span must be finite and > 0".to_owned(),
            ));
        }

        let (start, end) = self.time_scale.visible_range();
        let span = end - start;
        let delta_time = -(driving_px / driving_axis_span_px) * span;
        self.pan_time_visible_by(delta_time)?;
        Ok(delta_time)
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
        if !self.interaction_input_behavior.allows_wheel_pan() {
            return Ok(0.0);
        }

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
        let _ = self.apply_time_scale_zoom_limit_behavior()?;
        let _ = self.apply_time_scale_edge_behavior()?;
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
        if !factor.is_finite() || factor <= 0.0 {
            return Err(ChartError::InvalidData(
                "zoom factor must be finite and > 0".to_owned(),
            ));
        }
        if !anchor_px.is_finite() {
            return Err(ChartError::InvalidData(
                "zoom anchor px must be finite".to_owned(),
            ));
        }
        if !min_span_absolute.is_finite() || min_span_absolute <= 0.0 {
            return Err(ChartError::InvalidData(
                "zoom min span must be finite and > 0".to_owned(),
            ));
        }

        if let Some((space, reference_step)) = self.resolve_time_index_coordinate_space() {
            let (start, end) = self.time_scale.visible_range();
            let current_span = (end - start).max(1e-9);
            let target_span = (current_span / factor).max(min_span_absolute);
            let effective_factor = current_span / target_span;
            let target_bar_spacing = (space.bar_spacing_px * effective_factor).max(f64::EPSILON);

            let anchor_x = anchor_px.clamp(0.0, f64::from(self.viewport.width));
            let anchor_time_before = self.map_pixel_to_x(anchor_x)?;
            let anchor_logical_index = space.coordinate_to_logical_index(anchor_x)?;
            let zoomed_space = TimeIndexCoordinateSpace {
                bar_spacing_px: target_bar_spacing,
                ..space
            };
            let target_right_offset = zoomed_space.solve_right_offset_for_anchor_preserving_zoom(
                space.bar_spacing_px,
                space.right_offset_bars,
                anchor_logical_index,
            )?;
            let (_, full_end) = self.time_scale.full_range();
            let target_end = full_end + target_right_offset * reference_step;
            let target_start = target_end - target_span;
            let viewport_width = f64::from(self.viewport.width);
            let anchor_time_after = if viewport_width > 0.0 {
                target_start + (anchor_x / viewport_width) * target_span
            } else {
                anchor_time_before
            };
            if (anchor_time_after - anchor_time_before).abs() <= 1e-9 {
                self.time_scale
                    .set_visible_range_from_bar_spacing_and_right_offset(
                        target_bar_spacing,
                        target_right_offset,
                        reference_step,
                        space.width_px,
                    )?;
                let _ = self.apply_time_scale_zoom_limit_behavior()?;
                let _ = self.apply_time_scale_edge_behavior()?;
                self.emit_visible_range_changed();
                return Ok(());
            }
        }

        let anchor_time = self.map_pixel_to_x(anchor_px)?;
        self.time_scale
            .zoom_visible_by_factor(factor, anchor_time, min_span_absolute)?;
        let _ = self.apply_time_scale_zoom_limit_behavior()?;
        let _ = self.apply_time_scale_edge_behavior()?;
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
        if !self.interaction_input_behavior.allows_wheel_zoom() {
            return Ok(1.0);
        }

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

        if self
            .time_scale_scroll_zoom_behavior
            .right_bar_stays_on_scroll
        {
            if let Some(anchor_px) = self.resolve_right_margin_zoom_anchor_px() {
                self.zoom_time_visible_around_pixel(factor, anchor_px, min_span_absolute)?;
            } else {
                let (_, right_edge) = self.time_scale.visible_range();
                self.zoom_time_visible_around_time(factor, right_edge, min_span_absolute)?;
            }
        } else {
            self.zoom_time_visible_around_pixel(factor, anchor_px, min_span_absolute)?;
        }
        Ok(factor)
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
        if !self.interaction_input_behavior.allows_pinch_zoom() {
            return Ok(1.0);
        }
        if self
            .time_scale_scroll_zoom_behavior
            .right_bar_stays_on_scroll
        {
            if let Some(anchor_px) = self.resolve_right_margin_zoom_anchor_px() {
                self.zoom_time_visible_around_pixel(factor, anchor_px, min_span_absolute)?;
            } else {
                let (_, right_edge) = self.time_scale.visible_range();
                self.zoom_time_visible_around_time(factor, right_edge, min_span_absolute)?;
            }
        } else {
            self.zoom_time_visible_around_pixel(factor, anchor_px, min_span_absolute)?;
        }
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
        let _ = self.apply_time_scale_constraints()?;
        self.emit_visible_range_changed();
        Ok(())
    }

    pub(crate) fn apply_time_scale_constraints(&mut self) -> ChartResult<bool> {
        let mut changed = false;
        changed |= self.apply_time_scale_navigation_behavior()?;
        changed |= self.apply_time_scale_zoom_limit_behavior()?;
        changed |= self.apply_time_scale_edge_behavior()?;
        Ok(changed)
    }

    pub(crate) fn apply_time_scale_edge_behavior(&mut self) -> ChartResult<bool> {
        self.time_scale.clamp_visible_range_to_full_edges(
            self.time_scale_edge_behavior.fix_left_edge,
            self.time_scale_edge_behavior.fix_right_edge,
        )
    }

    fn apply_time_scale_navigation_behavior(&mut self) -> ChartResult<bool> {
        let behavior = self.time_scale_navigation_behavior;
        if behavior.right_offset_bars == 0.0
            && behavior.bar_spacing_px.is_none()
            && self.time_scale_right_offset_px.is_none()
        {
            return Ok(false);
        }
        let reference_step = resolve_reference_time_step(&self.points, &self.candles);

        let (visible_start, visible_end) = self.time_scale.visible_range();
        let current_span = (visible_end - visible_start).max(1e-9);

        let (_, full_end) = self.time_scale.full_range();
        if self.time_scale_right_offset_px.is_none() {
            if let (Some(step), Some(spacing_px)) = (reference_step, behavior.bar_spacing_px) {
                let previous = self.time_scale.visible_range();
                self.time_scale
                    .set_visible_range_from_bar_spacing_and_right_offset(
                        spacing_px,
                        behavior.right_offset_bars,
                        step,
                        f64::from(self.viewport.width),
                    )?;
                let current = self.time_scale.visible_range();
                let changed = (current.0 - previous.0).abs() > 1e-12
                    || (current.1 - previous.1).abs() > 1e-12;
                return Ok(changed);
            }
        }

        let target_span = match behavior.bar_spacing_px {
            Some(spacing_px) => {
                if let Some(step) = reference_step {
                    let visible_bars = (f64::from(self.viewport.width) / spacing_px).max(1.0);
                    (step * visible_bars).max(1e-9)
                } else {
                    current_span
                }
            }
            None => current_span,
        };
        let target_end = resolve_navigation_target_end(
            full_end,
            behavior.right_offset_bars,
            self.time_scale_right_offset_px,
            reference_step,
            target_span,
            f64::from(self.viewport.width),
        );
        let target_start = target_end - target_span;

        let changed = (target_start - visible_start).abs() > 1e-12
            || (target_end - visible_end).abs() > 1e-12;
        if changed {
            self.time_scale
                .set_visible_range(target_start, target_end)?;
        }
        Ok(changed)
    }

    pub(crate) fn apply_time_scale_zoom_limit_behavior(&mut self) -> ChartResult<bool> {
        let behavior = self.time_scale_zoom_limit_behavior;
        let viewport_width = f64::from(self.viewport.width);
        if viewport_width <= 0.0 {
            return Ok(false);
        }

        let Some(reference_step) = resolve_reference_time_step(&self.points, &self.candles) else {
            return Ok(false);
        };
        if !reference_step.is_finite() || reference_step <= 0.0 {
            return Ok(false);
        }

        let max_span =
            (reference_step * (viewport_width / behavior.min_bar_spacing_px).max(1.0)).max(1e-9);
        let min_span = match behavior.max_bar_spacing_px {
            Some(max_spacing_px) => {
                (reference_step * (viewport_width / max_spacing_px).max(1.0)).max(1e-9)
            }
            None => 1e-9,
        };

        let (visible_start, visible_end) = self.time_scale.visible_range();
        let current_span = (visible_end - visible_start).max(1e-9);
        let target_span = current_span.clamp(min_span, max_span);
        if (target_span - current_span).abs() <= 1e-12 {
            return Ok(false);
        }
        if !target_span.is_finite() || target_span <= 0.0 {
            return Err(ChartError::InvalidData(
                "time scale zoom-limit target span must be finite and > 0".to_owned(),
            ));
        }

        let navigation_active = self.time_scale_navigation_behavior.right_offset_bars != 0.0
            || self.time_scale_navigation_behavior.bar_spacing_px.is_some()
            || self.time_scale_right_offset_px.is_some();
        if navigation_active {
            let (_, full_end) = self.time_scale.full_range();
            let target_end = resolve_navigation_target_end(
                full_end,
                self.time_scale_navigation_behavior.right_offset_bars,
                self.time_scale_right_offset_px,
                Some(reference_step),
                target_span,
                viewport_width,
            );
            self.time_scale
                .set_visible_range(target_end - target_span, target_end)?;
        } else {
            let center = (visible_start + visible_end) * 0.5;
            let half = target_span * 0.5;
            self.time_scale
                .set_visible_range(center - half, center + half)?;
        }
        Ok(true)
    }

    pub(crate) fn apply_time_scale_resize_behavior(
        &mut self,
        previous_viewport_width_px: u32,
    ) -> ChartResult<bool> {
        let behavior = self.time_scale_resize_behavior;
        if !behavior.lock_visible_range_on_resize {
            return Ok(false);
        }

        let previous_width = f64::from(previous_viewport_width_px);
        let current_width = f64::from(self.viewport.width);
        if previous_width <= 0.0 || current_width <= 0.0 {
            return Ok(false);
        }
        if (previous_width - current_width).abs() <= f64::EPSILON {
            return Ok(false);
        }

        let (start, end) = self.time_scale.visible_range();
        let current_span = (end - start).max(1e-9);
        let center = (start + end) * 0.5;

        let target_span =
            if let Some(spacing_px) = self.time_scale_navigation_behavior.bar_spacing_px {
                let Some(step) = resolve_reference_time_step(&self.points, &self.candles) else {
                    return Ok(false);
                };
                let visible_bars = (current_width / spacing_px).max(1.0);
                (step * visible_bars).max(1e-9)
            } else {
                current_span
            };

        let (target_start, target_end) = if self.time_scale_right_offset_px.is_some() {
            let (_, full_end) = self.time_scale.full_range();
            let target_end = resolve_navigation_target_end(
                full_end,
                self.time_scale_navigation_behavior.right_offset_bars,
                self.time_scale_right_offset_px,
                resolve_reference_time_step(&self.points, &self.candles),
                target_span,
                current_width,
            );
            (target_end - target_span, target_end)
        } else {
            match behavior.anchor {
                TimeScaleResizeAnchor::Left => (start, start + target_span),
                TimeScaleResizeAnchor::Center => {
                    let half = target_span * 0.5;
                    (center - half, center + half)
                }
                TimeScaleResizeAnchor::Right => (end - target_span, end),
            }
        };

        let changed = (target_start - start).abs() > 1e-12 || (target_end - end).abs() > 1e-12;
        if changed {
            self.time_scale
                .set_visible_range(target_start, target_end)?;
        }
        Ok(changed)
    }

    pub(crate) fn handle_realtime_time_append(&mut self, appended_time: f64) -> bool {
        if !appended_time.is_finite() {
            return false;
        }

        let behavior = self.time_scale_realtime_append_behavior;
        let (visible_start_before, visible_end_before) = self.time_scale.visible_range();
        let (_, full_end_before) = self.time_scale.full_range();
        let reference_step_before = resolve_reference_time_step(&self.points, &self.candles);

        let right_edge_before = resolve_navigation_target_end(
            full_end_before,
            self.time_scale_navigation_behavior.right_offset_bars,
            self.time_scale_right_offset_px,
            reference_step_before,
            (visible_end_before - visible_start_before).max(1e-9),
            f64::from(self.viewport.width),
        );
        let tolerance =
            resolve_right_edge_tolerance(reference_step_before, behavior.right_edge_tolerance_bars);
        let should_track_right_edge = behavior.preserve_right_edge_on_append
            && (visible_end_before - right_edge_before).abs() <= tolerance;

        let full_range_changed = self
            .time_scale
            .include_time_in_full_range(appended_time, 1.0)
            .unwrap_or(false);
        if !full_range_changed {
            return false;
        }

        if !should_track_right_edge {
            return false;
        }

        let navigation_active = self.time_scale_navigation_behavior.right_offset_bars != 0.0
            || self.time_scale_navigation_behavior.bar_spacing_px.is_some()
            || self.time_scale_right_offset_px.is_some();
        if navigation_active {
            return self.apply_time_scale_constraints().unwrap_or(false);
        }

        let (_, full_end_after) = self.time_scale.full_range();
        let reference_step_after =
            resolve_reference_time_step(&self.points, &self.candles).or(reference_step_before);
        let right_edge_after = resolve_navigation_target_end(
            full_end_after,
            self.time_scale_navigation_behavior.right_offset_bars,
            self.time_scale_right_offset_px,
            reference_step_after,
            (visible_end_before - visible_start_before).max(1e-9),
            f64::from(self.viewport.width),
        );
        let delta = right_edge_after - right_edge_before;

        let mut changed = false;
        if delta.abs() > 1e-12 {
            let target_start = visible_start_before + delta;
            let target_end = visible_end_before + delta;
            changed = self
                .time_scale
                .set_visible_range(target_start, target_end)
                .is_ok();
        }

        if self.apply_time_scale_edge_behavior().unwrap_or(false) {
            changed = true;
        }

        changed
    }

    pub(crate) fn resolve_right_margin_zoom_anchor_px(&self) -> Option<f64> {
        let offset_px = self.time_scale_right_offset_px?;
        let viewport_width = f64::from(self.viewport.width);
        if !viewport_width.is_finite() || viewport_width <= 0.0 {
            return None;
        }
        Some((viewport_width - offset_px).clamp(0.0, viewport_width))
    }

    pub(crate) fn resolve_time_index_coordinate_space(
        &self,
    ) -> Option<(TimeIndexCoordinateSpace, f64)> {
        let viewport_width = f64::from(self.viewport.width);
        if !viewport_width.is_finite() || viewport_width <= 0.0 {
            return None;
        }

        let reference_step = resolve_reference_time_step(&self.points, &self.candles)?;
        if !reference_step.is_finite() || reference_step <= 0.0 {
            return None;
        }

        let (bar_spacing_px, right_offset_bars) = self
            .time_scale
            .derive_visible_bar_spacing_and_right_offset(reference_step, viewport_width)
            .ok()?;
        let (_, full_end) = self.time_scale.full_range();

        let base_index = full_end / reference_step;
        if !base_index.is_finite() {
            return None;
        }

        Some((
            TimeIndexCoordinateSpace {
                base_index,
                right_offset_bars,
                bar_spacing_px,
                width_px: viewport_width,
            },
            reference_step,
        ))
    }
}

fn validate_time_scale_navigation_behavior(
    behavior: TimeScaleNavigationBehavior,
) -> ChartResult<()> {
    if !behavior.right_offset_bars.is_finite() {
        return Err(ChartError::InvalidData(
            "time scale right offset must be finite".to_owned(),
        ));
    }

    if let Some(bar_spacing_px) = behavior.bar_spacing_px {
        if !bar_spacing_px.is_finite() || bar_spacing_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "time scale bar spacing must be finite and > 0".to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_time_scale_realtime_append_behavior(
    behavior: TimeScaleRealtimeAppendBehavior,
) -> ChartResult<()> {
    if !behavior.right_edge_tolerance_bars.is_finite() || behavior.right_edge_tolerance_bars < 0.0 {
        return Err(ChartError::InvalidData(
            "time scale realtime right-edge tolerance must be finite and >= 0".to_owned(),
        ));
    }
    Ok(())
}

fn validate_time_scale_zoom_limit_behavior(
    behavior: TimeScaleZoomLimitBehavior,
) -> ChartResult<()> {
    if !behavior.min_bar_spacing_px.is_finite() || behavior.min_bar_spacing_px <= 0.0 {
        return Err(ChartError::InvalidData(
            "time scale minimum bar spacing must be finite and > 0".to_owned(),
        ));
    }

    if let Some(max_bar_spacing_px) = behavior.max_bar_spacing_px {
        if !max_bar_spacing_px.is_finite() || max_bar_spacing_px <= 0.0 {
            return Err(ChartError::InvalidData(
                "time scale maximum bar spacing must be finite and > 0".to_owned(),
            ));
        }
        if max_bar_spacing_px < behavior.min_bar_spacing_px {
            return Err(ChartError::InvalidData(
                "time scale maximum bar spacing must be >= minimum bar spacing".to_owned(),
            ));
        }
    }

    Ok(())
}

fn resolve_navigation_target_end(
    full_end: f64,
    right_offset_bars: f64,
    right_offset_px: Option<f64>,
    reference_step: Option<f64>,
    visible_span: f64,
    viewport_width: f64,
) -> f64 {
    if let Some(px) = right_offset_px {
        if viewport_width > 0.0 {
            return full_end + (visible_span.max(1e-9) / viewport_width) * px;
        }
        return full_end;
    }

    if right_offset_bars == 0.0 {
        return full_end;
    }
    match reference_step {
        Some(step) if step.is_finite() && step > 0.0 => full_end + right_offset_bars * step,
        _ => full_end,
    }
}

fn resolve_right_edge_tolerance(reference_step: Option<f64>, tolerance_bars: f64) -> f64 {
    let epsilon = 1e-9;
    if !tolerance_bars.is_finite() || tolerance_bars < 0.0 {
        return epsilon;
    }
    match reference_step {
        Some(step) if step.is_finite() && step > 0.0 => epsilon + step * tolerance_bars,
        _ => epsilon,
    }
}

fn resolve_reference_time_step(points: &[DataPoint], candles: &[OhlcBar]) -> Option<f64> {
    if let Some(step) = estimate_positive_time_step(candles.iter().map(|bar| bar.time)) {
        return Some(step);
    }
    estimate_positive_time_step(points.iter().map(|point| point.x))
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
