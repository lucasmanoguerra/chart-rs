use crate::core::{TimeIndexCoordinateSpace, TimeScaleTuning};
use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::{
    ChartEngine, TimeScaleResizeAnchor, time_scale_input_validation,
    time_scale_navigation_target_resolver, time_scale_pan_delta_resolver,
    time_scale_zoom_factor_resolver,
};

pub(super) struct TimeScaleCoordinator;

impl TimeScaleCoordinator {
    fn zoom_with_scroll_anchor_policy<R: Renderer>(
        engine: &mut ChartEngine<R>,
        factor: f64,
        anchor_px: f64,
        min_span_absolute: f64,
    ) -> ChartResult<()> {
        time_scale_input_validation::validate_zoom_inputs(factor, anchor_px, min_span_absolute)?;

        if engine
            .core
            .behavior
            .time_scale_scroll_zoom_behavior
            .right_bar_stays_on_scroll
        {
            if let Some(anchor_px) = engine.resolve_right_margin_zoom_anchor_px() {
                engine.zoom_time_visible_around_pixel(factor, anchor_px, min_span_absolute)?;
            } else {
                let (_, right_edge) = engine.core.model.time_scale.visible_range();
                engine.zoom_time_visible_around_time(factor, right_edge, min_span_absolute)?;
            }
        } else {
            engine.zoom_time_visible_around_pixel(factor, anchor_px, min_span_absolute)?;
        }
        Ok(())
    }

    fn mark_scroll_invalidation_intent<R: Renderer>(
        engine: &mut ChartEngine<R>,
        before: (f64, f64),
    ) {
        let after = engine.core.model.time_scale.visible_range();
        let before_span = (before.1 - before.0).abs();
        let after_span = (after.1 - after.0).abs();
        if (before_span - after_span).abs() <= 1e-9 {
            engine.set_lwc_time_scale_invalidation_intent(
                super::chart_runtime::LwcTimeScaleInvalidationIntent::ApplyRightOffset,
            );
        } else {
            engine.set_lwc_time_scale_invalidation_intent(
                super::chart_runtime::LwcTimeScaleInvalidationIntent::ApplyRange,
            );
        }
    }

    fn mark_pan_invalidation_intent<R: Renderer>(engine: &mut ChartEngine<R>, before: (f64, f64)) {
        Self::mark_scroll_invalidation_intent(engine, before);
    }

    pub(super) fn pan_time_visible_by<R: Renderer>(
        engine: &mut ChartEngine<R>,
        delta_time: f64,
    ) -> ChartResult<()> {
        let visible_before = engine.core.model.time_scale.visible_range();
        engine
            .core
            .model
            .time_scale
            .pan_visible_by_delta(delta_time)?;
        let _ = Self::apply_time_scale_zoom_limit_behavior(engine)?;
        let _ = Self::apply_time_scale_edge_behavior(engine)?;
        Self::mark_pan_invalidation_intent(engine, visible_before);
        engine.emit_visible_range_changed();
        Ok(())
    }

    pub(super) fn zoom_time_visible_around_time<R: Renderer>(
        engine: &mut ChartEngine<R>,
        factor: f64,
        anchor_time: f64,
        min_span_absolute: f64,
    ) -> ChartResult<()> {
        engine.core.model.time_scale.zoom_visible_by_factor(
            factor,
            anchor_time,
            min_span_absolute,
        )?;
        let _ = Self::apply_time_scale_zoom_limit_behavior(engine)?;
        if engine.core.behavior.time_scale_right_offset_px.is_some() {
            let _ = Self::apply_time_scale_navigation_behavior(engine)?;
        }
        let _ = Self::apply_time_scale_edge_behavior(engine)?;
        engine.set_lwc_time_scale_invalidation_intent(
            super::chart_runtime::LwcTimeScaleInvalidationIntent::ApplyBarSpacingAndRightOffset,
        );
        engine.emit_visible_range_changed();
        Ok(())
    }

    pub(super) fn fit_time_to_data<R: Renderer>(
        engine: &mut ChartEngine<R>,
        tuning: TimeScaleTuning,
    ) -> ChartResult<()> {
        if engine.core.model.points.is_empty() && engine.core.model.candles.is_empty() {
            return Ok(());
        }

        let points = &engine.core.model.points;
        let candles = &engine.core.model.candles;
        engine
            .core
            .model
            .time_scale
            .fit_to_mixed_data(points, candles, tuning)?;
        let _ = Self::apply_time_scale_constraints(engine)?;
        engine.set_lwc_time_scale_invalidation_intent(
            super::chart_runtime::LwcTimeScaleInvalidationIntent::FitContent,
        );
        engine.emit_visible_range_changed();
        Ok(())
    }

    pub(super) fn scroll_time_to_realtime<R: Renderer>(
        engine: &mut ChartEngine<R>,
    ) -> ChartResult<bool> {
        let visible_before = engine.core.model.time_scale.visible_range();
        let navigation_active = engine
            .core
            .behavior
            .time_scale_navigation_behavior
            .right_offset_bars
            != 0.0
            || engine
                .core
                .behavior
                .time_scale_navigation_behavior
                .bar_spacing_px
                .is_some()
            || engine.core.behavior.time_scale_right_offset_px.is_some();

        let mut changed = if navigation_active {
            Self::apply_time_scale_constraints(engine)?
        } else {
            let (start, end) = engine.core.model.time_scale.visible_range();
            let (_, full_end) = engine.core.model.time_scale.full_range();
            let reference_step = time_scale_navigation_target_resolver::resolve_reference_time_step(
                &engine.core.model.points,
                &engine.core.model.candles,
            );
            let visible_span = (end - start).max(1e-9);
            let (_, target_end) =
                time_scale_navigation_target_resolver::resolve_navigation_target_range(
                    full_end,
                    engine
                        .core
                        .behavior
                        .time_scale_navigation_behavior
                        .right_offset_bars,
                    engine.core.behavior.time_scale_right_offset_px,
                    reference_step,
                    visible_span,
                    f64::from(engine.core.model.viewport.width),
                );
            let delta = target_end - end;
            if delta.abs() > 1e-12 {
                engine
                    .core
                    .model
                    .time_scale
                    .set_visible_range(start + delta, end + delta)?;
                true
            } else {
                false
            }
        };

        if Self::apply_time_scale_edge_behavior(engine)? {
            changed = true;
        }

        if changed {
            Self::mark_scroll_invalidation_intent(engine, visible_before);
            engine.emit_visible_range_changed();
        }
        Ok(changed)
    }

    pub(super) fn time_scroll_position_bars<R: Renderer>(engine: &ChartEngine<R>) -> Option<f64> {
        let (_, full_end) = engine.core.model.time_scale.full_range();
        let (_, visible_end) = engine.core.model.time_scale.visible_range();
        let distance = visible_end - full_end;

        let step = time_scale_navigation_target_resolver::resolve_reference_time_step(
            &engine.core.model.points,
            &engine.core.model.candles,
        )?;
        if !step.is_finite() || step <= 0.0 {
            return None;
        }
        Some(distance / step)
    }

    pub(super) fn scroll_time_to_position_bars<R: Renderer>(
        engine: &mut ChartEngine<R>,
        position_bars: f64,
    ) -> ChartResult<bool> {
        if !position_bars.is_finite() {
            return Err(ChartError::InvalidData(
                "scroll position bars must be finite".to_owned(),
            ));
        }

        let visible_before = engine.core.model.time_scale.visible_range();
        let (_, full_end) = engine.core.model.time_scale.full_range();
        let (_, end) = engine.core.model.time_scale.visible_range();

        let target_end = if position_bars == 0.0 {
            full_end
        } else {
            let Some(step) = time_scale_navigation_target_resolver::resolve_reference_time_step(
                &engine.core.model.points,
                &engine.core.model.candles,
            ) else {
                return Err(ChartError::InvalidData(
                    "cannot resolve scroll position without reference data step".to_owned(),
                ));
            };
            full_end + position_bars * step
        };

        let delta = target_end - end;
        let mut changed = false;
        if delta.abs() > 1e-12 {
            let (start, end) = engine.core.model.time_scale.visible_range();
            engine
                .core
                .model
                .time_scale
                .set_visible_range(start + delta, end + delta)?;
            changed = true;
        }

        if Self::apply_time_scale_edge_behavior(engine)? {
            changed = true;
        }
        if changed {
            Self::mark_scroll_invalidation_intent(engine, visible_before);
            engine.emit_visible_range_changed();
        }
        Ok(changed)
    }

    pub(super) fn zoom_time_visible_around_pixel<R: Renderer>(
        engine: &mut ChartEngine<R>,
        factor: f64,
        anchor_px: f64,
        min_span_absolute: f64,
    ) -> ChartResult<()> {
        time_scale_input_validation::validate_zoom_inputs(factor, anchor_px, min_span_absolute)?;

        if let Some((space, reference_step)) = engine.resolve_time_index_coordinate_space() {
            let (start, end) = engine.core.model.time_scale.visible_range();
            let current_span = (end - start).max(1e-9);
            let target_span = (current_span / factor).max(min_span_absolute);
            let effective_factor = current_span / target_span;
            let target_bar_spacing = (space.bar_spacing_px * effective_factor).max(f64::EPSILON);

            let anchor_x = anchor_px.clamp(0.0, f64::from(engine.core.model.viewport.width));
            let anchor_time_before = engine.map_pixel_to_x(anchor_x)?;
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
            let (_, full_end) = engine.core.model.time_scale.full_range();
            let target_end = full_end + target_right_offset * reference_step;
            let target_start = target_end - target_span;
            let viewport_width = f64::from(engine.core.model.viewport.width);
            let anchor_time_after = if viewport_width > 0.0 {
                target_start + (anchor_x / viewport_width) * target_span
            } else {
                anchor_time_before
            };
            if (anchor_time_after - anchor_time_before).abs() <= 1e-9 {
                engine
                    .core
                    .model
                    .time_scale
                    .set_visible_range_from_bar_spacing_and_right_offset(
                        target_bar_spacing,
                        target_right_offset,
                        reference_step,
                        space.width_px,
                    )?;
                let _ = Self::apply_time_scale_zoom_limit_behavior(engine)?;
                if engine.core.behavior.time_scale_right_offset_px.is_some() {
                    let _ = Self::apply_time_scale_navigation_behavior(engine)?;
                }
                let _ = Self::apply_time_scale_edge_behavior(engine)?;
                engine.set_lwc_time_scale_invalidation_intent(
                    super::chart_runtime::LwcTimeScaleInvalidationIntent::ApplyBarSpacingAndRightOffset,
                );
                engine.emit_visible_range_changed();
                return Ok(());
            }
        }

        let anchor_time = engine.map_pixel_to_x(anchor_px)?;
        engine.core.model.time_scale.zoom_visible_by_factor(
            factor,
            anchor_time,
            min_span_absolute,
        )?;
        let _ = Self::apply_time_scale_zoom_limit_behavior(engine)?;
        if engine.core.behavior.time_scale_right_offset_px.is_some() {
            let _ = Self::apply_time_scale_navigation_behavior(engine)?;
        }
        let _ = Self::apply_time_scale_edge_behavior(engine)?;
        engine.set_lwc_time_scale_invalidation_intent(
            super::chart_runtime::LwcTimeScaleInvalidationIntent::ApplyBarSpacingAndRightOffset,
        );
        engine.emit_visible_range_changed();
        Ok(())
    }

    pub(super) fn pan_time_visible_by_pixels<R: Renderer>(
        engine: &mut ChartEngine<R>,
        delta_px: f64,
    ) -> ChartResult<()> {
        if !engine
            .core
            .behavior
            .interaction_input_behavior
            .allows_drag_pan()
        {
            return Ok(());
        }

        time_scale_input_validation::validate_pan_pixel_delta(delta_px)?;

        if let Some((space, reference_step)) = engine.resolve_time_index_coordinate_space() {
            let visible_before = engine.core.model.time_scale.visible_range();
            let target_right_offset = space.pan_right_offset_by_pixels(-delta_px)?;
            engine
                .core
                .model
                .time_scale
                .set_visible_range_from_bar_spacing_and_right_offset(
                    space.bar_spacing_px,
                    target_right_offset,
                    reference_step,
                    space.width_px,
                )?;
            let _ = engine.apply_time_scale_zoom_limit_behavior()?;
            let _ = engine.apply_time_scale_edge_behavior()?;
            Self::mark_pan_invalidation_intent(engine, visible_before);
            engine.emit_visible_range_changed();
            return Ok(());
        }

        let (start, end) = engine.core.model.time_scale.visible_range();
        let span = end - start;
        let delta_time = time_scale_pan_delta_resolver::resolve_pixel_pan_delta_time(
            delta_px,
            f64::from(engine.core.model.viewport.width),
            span,
        )?;
        engine.pan_time_visible_by(delta_time)
    }

    pub(super) fn touch_drag_pan_time_visible<R: Renderer>(
        engine: &mut ChartEngine<R>,
        delta_x_px: f64,
        delta_y_px: f64,
    ) -> ChartResult<f64> {
        let behavior = engine.core.behavior.interaction_input_behavior;
        if !behavior.handle_scroll
            || (!behavior.scroll_horz_touch_drag && !behavior.scroll_vert_touch_drag)
        {
            return Ok(0.0);
        }

        time_scale_input_validation::validate_touch_drag_deltas(behavior, delta_x_px, delta_y_px)?;

        let (start, end) = engine.core.model.time_scale.visible_range();
        let span = end - start;
        let Some(delta_time) = time_scale_pan_delta_resolver::resolve_touch_drag_pan_delta_time(
            delta_x_px,
            delta_y_px,
            f64::from(engine.core.model.viewport.width),
            f64::from(engine.core.model.viewport.height),
            span,
            behavior.scroll_horz_touch_drag,
            behavior.scroll_vert_touch_drag,
        )?
        else {
            return Ok(0.0);
        };
        engine.pan_time_visible_by(delta_time)?;
        Ok(delta_time)
    }

    pub(super) fn wheel_pan_time_visible<R: Renderer>(
        engine: &mut ChartEngine<R>,
        wheel_delta_x: f64,
        pan_step_ratio: f64,
    ) -> ChartResult<f64> {
        if !engine
            .core
            .behavior
            .interaction_input_behavior
            .allows_wheel_pan()
        {
            return Ok(0.0);
        }

        time_scale_input_validation::validate_wheel_pan_inputs(wheel_delta_x, pan_step_ratio)?;
        if wheel_delta_x == 0.0 {
            return Ok(0.0);
        }

        let (start, end) = engine.core.model.time_scale.visible_range();
        let span = end - start;
        let Some(delta_time) = time_scale_pan_delta_resolver::resolve_wheel_pan_delta_time(
            wheel_delta_x,
            span,
            pan_step_ratio,
        )?
        else {
            return Ok(0.0);
        };
        engine.pan_time_visible_by(delta_time)?;
        Ok(delta_time)
    }

    pub(super) fn wheel_zoom_time_visible<R: Renderer>(
        engine: &mut ChartEngine<R>,
        wheel_delta_y: f64,
        anchor_px: f64,
        zoom_step_ratio: f64,
        min_span_absolute: f64,
    ) -> ChartResult<f64> {
        if !engine
            .core
            .behavior
            .interaction_input_behavior
            .allows_wheel_zoom()
        {
            return Ok(1.0);
        }

        time_scale_input_validation::validate_wheel_zoom_inputs(wheel_delta_y, zoom_step_ratio)?;
        let Some(factor) = time_scale_zoom_factor_resolver::resolve_wheel_zoom_factor(
            wheel_delta_y,
            zoom_step_ratio,
        )?
        else {
            return Ok(1.0);
        };

        Self::zoom_with_scroll_anchor_policy(engine, factor, anchor_px, min_span_absolute)?;
        Ok(factor)
    }

    pub(super) fn pinch_zoom_time_visible<R: Renderer>(
        engine: &mut ChartEngine<R>,
        pinch_scale_factor: f64,
        anchor_px: f64,
        min_span_absolute: f64,
    ) -> ChartResult<f64> {
        if !engine
            .core
            .behavior
            .interaction_input_behavior
            .allows_pinch_zoom()
        {
            return Ok(1.0);
        }
        let Some(factor) =
            time_scale_zoom_factor_resolver::resolve_pinch_zoom_factor(pinch_scale_factor)?
        else {
            return Ok(1.0);
        };
        Self::zoom_with_scroll_anchor_policy(engine, factor, anchor_px, min_span_absolute)?;
        Ok(factor)
    }

    pub(super) fn apply_time_scale_constraints<R: Renderer>(
        engine: &mut ChartEngine<R>,
    ) -> ChartResult<bool> {
        let mut changed = false;
        changed |= Self::apply_time_scale_navigation_behavior(engine)?;
        changed |= Self::apply_time_scale_zoom_limit_behavior(engine)?;
        changed |= Self::apply_time_scale_edge_behavior(engine)?;
        Ok(changed)
    }

    pub(super) fn apply_time_scale_edge_behavior<R: Renderer>(
        engine: &mut ChartEngine<R>,
    ) -> ChartResult<bool> {
        let fix_left_edge = engine.core.behavior.time_scale_edge_behavior.fix_left_edge;
        let fix_right_edge = engine.core.behavior.time_scale_edge_behavior.fix_right_edge;
        engine
            .core
            .model
            .time_scale
            .clamp_visible_range_to_full_edges(fix_left_edge, fix_right_edge)
    }

    pub(super) fn apply_time_scale_zoom_limit_behavior<R: Renderer>(
        engine: &mut ChartEngine<R>,
    ) -> ChartResult<bool> {
        let visible_before = engine.core.model.time_scale.visible_range();
        let behavior = engine.core.behavior.time_scale_zoom_limit_behavior;
        let viewport_width = f64::from(engine.core.model.viewport.width);
        if viewport_width <= 0.0 {
            return Ok(false);
        }

        let Some(reference_step) =
            time_scale_navigation_target_resolver::resolve_reference_time_step(
                &engine.core.model.points,
                &engine.core.model.candles,
            )
        else {
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

        let (visible_start, visible_end) = engine.core.model.time_scale.visible_range();
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

        let navigation_active = engine
            .core
            .behavior
            .time_scale_navigation_behavior
            .right_offset_bars
            != 0.0
            || engine
                .core
                .behavior
                .time_scale_navigation_behavior
                .bar_spacing_px
                .is_some()
            || engine.core.behavior.time_scale_right_offset_px.is_some();
        if navigation_active {
            let (_, full_end) = engine.core.model.time_scale.full_range();
            let (_, target_end) =
                time_scale_navigation_target_resolver::resolve_navigation_target_range(
                    full_end,
                    engine
                        .core
                        .behavior
                        .time_scale_navigation_behavior
                        .right_offset_bars,
                    engine.core.behavior.time_scale_right_offset_px,
                    Some(reference_step),
                    target_span,
                    viewport_width,
                );
            engine
                .core
                .model
                .time_scale
                .set_visible_range(target_end - target_span, target_end)?;
        } else {
            let center = (visible_start + visible_end) * 0.5;
            let half = target_span * 0.5;
            engine
                .core
                .model
                .time_scale
                .set_visible_range(center - half, center + half)?;
        }
        Self::mark_zoom_invalidation_intent(engine, visible_before);
        Ok(true)
    }

    pub(super) fn apply_time_scale_resize_behavior<R: Renderer>(
        engine: &mut ChartEngine<R>,
        previous_viewport_width_px: u32,
    ) -> ChartResult<bool> {
        let visible_before = engine.core.model.time_scale.visible_range();
        let behavior = engine.core.behavior.time_scale_resize_behavior;
        if !behavior.lock_visible_range_on_resize {
            return Ok(false);
        }

        let previous_width = f64::from(previous_viewport_width_px);
        let current_width = f64::from(engine.core.model.viewport.width);
        if previous_width <= 0.0 || current_width <= 0.0 {
            return Ok(false);
        }
        if (previous_width - current_width).abs() <= f64::EPSILON {
            return Ok(false);
        }

        let (start, end) = engine.core.model.time_scale.visible_range();
        let current_span = (end - start).max(1e-9);
        let center = (start + end) * 0.5;

        let target_span = if let Some(spacing_px) = engine
            .core
            .behavior
            .time_scale_navigation_behavior
            .bar_spacing_px
        {
            let Some(step) = time_scale_navigation_target_resolver::resolve_reference_time_step(
                &engine.core.model.points,
                &engine.core.model.candles,
            ) else {
                return Ok(false);
            };
            let visible_bars = (current_width / spacing_px).max(1.0);
            (step * visible_bars).max(1e-9)
        } else {
            current_span
        };

        let (target_start, target_end) =
            if engine.core.behavior.time_scale_right_offset_px.is_some() {
                let (_, full_end) = engine.core.model.time_scale.full_range();
                time_scale_navigation_target_resolver::resolve_navigation_target_range(
                    full_end,
                    engine
                        .core
                        .behavior
                        .time_scale_navigation_behavior
                        .right_offset_bars,
                    engine.core.behavior.time_scale_right_offset_px,
                    time_scale_navigation_target_resolver::resolve_reference_time_step(
                        &engine.core.model.points,
                        &engine.core.model.candles,
                    ),
                    target_span,
                    current_width,
                )
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
            engine
                .core
                .model
                .time_scale
                .set_visible_range(target_start, target_end)?;
            Self::mark_zoom_invalidation_intent(engine, visible_before);
        }
        Ok(changed)
    }

    pub(super) fn handle_realtime_time_append<R: Renderer>(
        engine: &mut ChartEngine<R>,
        appended_time: f64,
    ) -> bool {
        if !appended_time.is_finite() {
            return false;
        }

        let behavior = engine.core.behavior.time_scale_realtime_append_behavior;
        let (visible_start_before, visible_end_before) =
            engine.core.model.time_scale.visible_range();
        let (_, full_end_before) = engine.core.model.time_scale.full_range();
        let reference_step_before =
            time_scale_navigation_target_resolver::resolve_reference_time_step(
                &engine.core.model.points,
                &engine.core.model.candles,
            );

        let right_edge_before =
            time_scale_navigation_target_resolver::resolve_navigation_target_end(
                full_end_before,
                engine
                    .core
                    .behavior
                    .time_scale_navigation_behavior
                    .right_offset_bars,
                engine.core.behavior.time_scale_right_offset_px,
                reference_step_before,
                (visible_end_before - visible_start_before).max(1e-9),
                f64::from(engine.core.model.viewport.width),
            );
        let tolerance = Self::resolve_right_edge_tolerance(
            reference_step_before,
            behavior.right_edge_tolerance_bars,
        );
        let should_track_right_edge = behavior.preserve_right_edge_on_append
            && (visible_end_before - right_edge_before).abs() <= tolerance;

        let full_range_changed = engine
            .core
            .model
            .time_scale
            .include_time_in_full_range(appended_time, 1.0)
            .unwrap_or(false);
        if !full_range_changed {
            return false;
        }

        if !should_track_right_edge {
            return false;
        }

        let navigation_active = engine
            .core
            .behavior
            .time_scale_navigation_behavior
            .right_offset_bars
            != 0.0
            || engine
                .core
                .behavior
                .time_scale_navigation_behavior
                .bar_spacing_px
                .is_some()
            || engine.core.behavior.time_scale_right_offset_px.is_some();
        if navigation_active {
            let changed = Self::apply_time_scale_constraints(engine).unwrap_or(false);
            if changed
                && engine
                    .core
                    .runtime
                    .pending_lwc_time_scale_invalidation_intent
                    .is_none()
            {
                Self::mark_scroll_invalidation_intent(
                    engine,
                    (visible_start_before, visible_end_before),
                );
            }
            return changed;
        }

        let (_, full_end_after) = engine.core.model.time_scale.full_range();
        let reference_step_after =
            time_scale_navigation_target_resolver::resolve_reference_time_step(
                &engine.core.model.points,
                &engine.core.model.candles,
            )
            .or(reference_step_before);
        let (_, right_edge_after) =
            time_scale_navigation_target_resolver::resolve_navigation_target_range(
                full_end_after,
                engine
                    .core
                    .behavior
                    .time_scale_navigation_behavior
                    .right_offset_bars,
                engine.core.behavior.time_scale_right_offset_px,
                reference_step_after,
                (visible_end_before - visible_start_before).max(1e-9),
                f64::from(engine.core.model.viewport.width),
            );
        let delta = right_edge_after - right_edge_before;

        let mut changed = false;
        if delta.abs() > 1e-12 {
            let target_start = visible_start_before + delta;
            let target_end = visible_end_before + delta;
            changed = engine
                .core
                .model
                .time_scale
                .set_visible_range(target_start, target_end)
                .is_ok();
        }

        if Self::apply_time_scale_edge_behavior(engine).unwrap_or(false) {
            changed = true;
        }

        if changed {
            Self::mark_scroll_invalidation_intent(
                engine,
                (visible_start_before, visible_end_before),
            );
        }

        changed
    }

    pub(super) fn resolve_right_margin_zoom_anchor_px<R: Renderer>(
        engine: &ChartEngine<R>,
    ) -> Option<f64> {
        let offset_px = engine.core.behavior.time_scale_right_offset_px?;
        let viewport_width = f64::from(engine.core.model.viewport.width);
        if !viewport_width.is_finite() || viewport_width <= 0.0 {
            return None;
        }
        Some((viewport_width - offset_px).clamp(0.0, viewport_width))
    }

    pub(super) fn resolve_time_index_coordinate_space<R: Renderer>(
        engine: &ChartEngine<R>,
    ) -> Option<(TimeIndexCoordinateSpace, f64)> {
        let viewport_width = f64::from(engine.core.model.viewport.width);
        if !viewport_width.is_finite() || viewport_width <= 0.0 {
            return None;
        }

        let reference_step = time_scale_navigation_target_resolver::resolve_reference_time_step(
            &engine.core.model.points,
            &engine.core.model.candles,
        )?;
        if !reference_step.is_finite() || reference_step <= 0.0 {
            return None;
        }

        let (bar_spacing_px, right_offset_bars) = engine
            .core
            .model
            .time_scale
            .derive_visible_bar_spacing_and_right_offset(reference_step, viewport_width)
            .ok()?;
        let (_, full_end) = engine.core.model.time_scale.full_range();

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

    fn apply_time_scale_navigation_behavior<R: Renderer>(
        engine: &mut ChartEngine<R>,
    ) -> ChartResult<bool> {
        let visible_before = engine.core.model.time_scale.visible_range();
        let behavior = engine.core.behavior.time_scale_navigation_behavior;
        if behavior.right_offset_bars == 0.0
            && behavior.bar_spacing_px.is_none()
            && engine.core.behavior.time_scale_right_offset_px.is_none()
        {
            return Ok(false);
        }
        let reference_step = time_scale_navigation_target_resolver::resolve_reference_time_step(
            &engine.core.model.points,
            &engine.core.model.candles,
        );

        let (visible_start, visible_end) = engine.core.model.time_scale.visible_range();
        let current_span = (visible_end - visible_start).max(1e-9);

        let (_, full_end) = engine.core.model.time_scale.full_range();
        let viewport_width = f64::from(engine.core.model.viewport.width);
        if engine.core.behavior.time_scale_right_offset_px.is_none() {
            if let (Some(step), Some(spacing_px)) = (reference_step, behavior.bar_spacing_px) {
                let previous = engine.core.model.time_scale.visible_range();
                engine
                    .core
                    .model
                    .time_scale
                    .set_visible_range_from_bar_spacing_and_right_offset(
                        spacing_px,
                        behavior.right_offset_bars,
                        step,
                        viewport_width,
                    )?;
                let current = engine.core.model.time_scale.visible_range();
                let changed = (current.0 - previous.0).abs() > 1e-12
                    || (current.1 - previous.1).abs() > 1e-12;
                if changed {
                    Self::mark_zoom_invalidation_intent(engine, visible_before);
                }
                return Ok(changed);
            }
        }

        let target_span = match behavior.bar_spacing_px {
            Some(spacing_px) => {
                if let Some(step) = reference_step {
                    let visible_bars = (viewport_width / spacing_px).max(1.0);
                    (step * visible_bars).max(1e-9)
                } else {
                    current_span
                }
            }
            None => current_span,
        };
        let (target_start, target_end) =
            time_scale_navigation_target_resolver::resolve_navigation_target_range(
                full_end,
                behavior.right_offset_bars,
                engine.core.behavior.time_scale_right_offset_px,
                reference_step,
                target_span,
                viewport_width,
            );

        let changed = (target_start - visible_start).abs() > 1e-12
            || (target_end - visible_end).abs() > 1e-12;
        if changed {
            engine
                .core
                .model
                .time_scale
                .set_visible_range(target_start, target_end)?;
            Self::mark_zoom_invalidation_intent(engine, visible_before);
        }
        Ok(changed)
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

    fn mark_zoom_invalidation_intent<R: Renderer>(engine: &mut ChartEngine<R>, before: (f64, f64)) {
        let after = engine.core.model.time_scale.visible_range();
        let before_span = (before.1 - before.0).abs();
        let after_span = (after.1 - after.0).abs();
        if (before_span - after_span).abs() <= 1e-9 {
            engine.set_lwc_time_scale_invalidation_intent(
                super::chart_runtime::LwcTimeScaleInvalidationIntent::ApplyRightOffset,
            );
        } else {
            engine.set_lwc_time_scale_invalidation_intent(
                super::chart_runtime::LwcTimeScaleInvalidationIntent::ApplyBarSpacingAndRightOffset,
            );
        }
    }
}
