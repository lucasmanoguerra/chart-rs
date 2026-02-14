use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::ChartEngine;
use super::layout_helpers::resolve_axis_layout;

pub(super) struct ScaleCoordinator;

impl ScaleCoordinator {
    pub(super) fn axis_drag_scale_time<R: Renderer>(
        engine: &mut ChartEngine<R>,
        drag_delta_x_px: f64,
        anchor_x_px: f64,
        zoom_step_ratio: f64,
        min_span_absolute: f64,
    ) -> ChartResult<f64> {
        if !engine
            .core
            .behavior
            .interaction_input_behavior
            .allows_axis_drag_scale()
        {
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
            let viewport_width = f64::from(engine.core.model.viewport.width);
            let viewport_height = f64::from(engine.core.model.viewport.height);
            let layout = resolve_axis_layout(
                viewport_width,
                viewport_height,
                engine.core.presentation.render_style.price_axis_width_px,
                engine.core.presentation.render_style.time_axis_height_px,
            );
            let anchor_x = anchor_x_px.clamp(0.0, layout.plot_right);
            engine.zoom_time_visible_around_pixel(factor, anchor_x, min_span_absolute)?;
        }
        Ok(factor)
    }

    pub(super) fn axis_double_click_reset_time_scale<R: Renderer>(
        engine: &mut ChartEngine<R>,
    ) -> ChartResult<bool> {
        if !engine
            .core
            .behavior
            .interaction_input_behavior
            .allows_axis_double_click_reset()
        {
            return Ok(false);
        }

        let before = engine.core.model.time_scale.visible_range();
        engine.core.model.time_scale.reset_visible_range_to_full();
        let mut changed = engine.apply_time_scale_constraints()?;
        let after = engine.core.model.time_scale.visible_range();
        changed |= (after.0 - before.0).abs() > 1e-12 || (after.1 - before.1).abs() > 1e-12;

        if changed {
            engine.emit_visible_range_changed();
        }
        Ok(changed)
    }

    pub(super) fn axis_drag_pan_price<R: Renderer>(
        engine: &mut ChartEngine<R>,
        drag_delta_y_px: f64,
        anchor_y_px: f64,
    ) -> ChartResult<bool> {
        if !engine
            .core
            .behavior
            .interaction_input_behavior
            .allows_axis_drag_scale()
        {
            return Ok(false);
        }

        if !drag_delta_y_px.is_finite() {
            return Err(ChartError::InvalidData(
                "axis drag pan delta must be finite".to_owned(),
            ));
        }
        if !anchor_y_px.is_finite() {
            return Err(ChartError::InvalidData(
                "axis drag pan anchor y must be finite".to_owned(),
            ));
        }
        if drag_delta_y_px == 0.0 {
            return Ok(false);
        }

        let viewport_width = f64::from(engine.core.model.viewport.width);
        let viewport_height = f64::from(engine.core.model.viewport.height);
        let layout = resolve_axis_layout(
            viewport_width,
            viewport_height,
            engine.core.presentation.render_style.price_axis_width_px,
            engine.core.presentation.render_style.time_axis_height_px,
        );
        let plot_bottom = layout.plot_bottom;
        let anchor_y = anchor_y_px.clamp(0.0, plot_bottom);
        let shifted_anchor_y = (anchor_y + drag_delta_y_px).clamp(0.0, plot_bottom);
        if (shifted_anchor_y - anchor_y).abs() <= 1e-12 {
            return Ok(false);
        }

        let anchor_price_before = engine.map_pixel_to_price(anchor_y)?;
        let shifted_anchor_price_before = engine.map_pixel_to_price(shifted_anchor_y)?;
        let (domain_start, domain_end) = engine.core.model.price_scale.domain();

        let (new_start, new_end) = match engine.core.model.price_scale_mode {
            crate::core::PriceScaleMode::Log => {
                if anchor_price_before <= 0.0 || shifted_anchor_price_before <= 0.0 {
                    return Err(ChartError::InvalidData(
                        "axis drag pan requires positive anchor prices in log mode".to_owned(),
                    ));
                }
                let ratio = anchor_price_before / shifted_anchor_price_before;
                if !ratio.is_finite() || ratio <= 0.0 {
                    return Err(ChartError::InvalidData(
                        "axis drag pan produced invalid log-domain ratio".to_owned(),
                    ));
                }
                (domain_start * ratio, domain_end * ratio)
            }
            crate::core::PriceScaleMode::Linear
            | crate::core::PriceScaleMode::Percentage
            | crate::core::PriceScaleMode::IndexedTo100 => {
                let delta_price = anchor_price_before - shifted_anchor_price_before;
                (domain_start + delta_price, domain_end + delta_price)
            }
        };

        if !new_start.is_finite() || !new_end.is_finite() {
            return Err(ChartError::InvalidData(
                "axis drag pan produced non-finite price domain".to_owned(),
            ));
        }
        if (new_start - domain_start).abs() <= 1e-12 && (new_end - domain_end).abs() <= 1e-12 {
            return Ok(false);
        }

        Self::set_price_domain_preserving_mode(engine, new_start, new_end)?;
        Ok(true)
    }

    pub(super) fn axis_drag_scale_price<R: Renderer>(
        engine: &mut ChartEngine<R>,
        drag_delta_y_px: f64,
        anchor_y_px: f64,
        zoom_step_ratio: f64,
        min_span_absolute: f64,
    ) -> ChartResult<f64> {
        if !engine
            .core
            .behavior
            .interaction_input_behavior
            .allows_axis_drag_scale()
        {
            return Ok(1.0);
        }

        if !drag_delta_y_px.is_finite() {
            return Err(ChartError::InvalidData(
                "axis drag delta must be finite".to_owned(),
            ));
        }
        if !anchor_y_px.is_finite() {
            return Err(ChartError::InvalidData(
                "axis drag anchor y must be finite".to_owned(),
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
        if drag_delta_y_px == 0.0 {
            return Ok(1.0);
        }

        let normalized_steps = drag_delta_y_px / 120.0;
        let base = 1.0 + zoom_step_ratio;
        let mut factor = base.powf(normalized_steps);
        if !factor.is_finite() || factor <= 0.0 {
            return Err(ChartError::InvalidData(
                "computed axis drag zoom factor must be finite and > 0".to_owned(),
            ));
        }

        let (domain_start, domain_end) = engine.core.model.price_scale.domain();
        let current_span = (domain_end - domain_start).abs();
        if !current_span.is_finite() || current_span <= 0.0 {
            return Err(ChartError::InvalidData(
                "price domain span must be finite and non-zero".to_owned(),
            ));
        }

        let unclamped_target_span = current_span * factor;
        let target_span = unclamped_target_span.max(min_span_absolute);
        factor = target_span / current_span;

        let viewport_width = f64::from(engine.core.model.viewport.width);
        let viewport_height = f64::from(engine.core.model.viewport.height);
        let layout = resolve_axis_layout(
            viewport_width,
            viewport_height,
            engine.core.presentation.render_style.price_axis_width_px,
            engine.core.presentation.render_style.time_axis_height_px,
        );
        let plot_bottom = layout.plot_bottom;
        let anchor_y = anchor_y_px.clamp(0.0, plot_bottom);
        let anchor_price = engine.map_pixel_to_price(anchor_y)?;

        let scaled_start = anchor_price + (domain_start - anchor_price) * factor;
        let scaled_end = anchor_price + (domain_end - anchor_price) * factor;
        Self::set_price_domain_preserving_mode(engine, scaled_start, scaled_end)?;
        Ok(factor)
    }

    pub(super) fn axis_double_click_reset_price_scale<R: Renderer>(
        engine: &mut ChartEngine<R>,
    ) -> ChartResult<bool> {
        if !engine
            .core
            .behavior
            .interaction_input_behavior
            .allows_axis_double_click_reset()
        {
            return Ok(false);
        }

        let before = engine.core.model.price_scale.domain();
        if !engine.core.model.candles.is_empty() {
            engine.autoscale_price_from_candles()?;
        } else if !engine.core.model.points.is_empty() {
            engine.autoscale_price_from_data()?;
        } else {
            return Ok(false);
        }
        let after = engine.core.model.price_scale.domain();
        Ok((after.0 - before.0).abs() > 1e-12 || (after.1 - before.1).abs() > 1e-12)
    }

    fn set_price_domain_preserving_mode<R: Renderer>(
        engine: &mut ChartEngine<R>,
        domain_start: f64,
        domain_end: f64,
    ) -> ChartResult<()> {
        engine.rebuild_price_scale_from_domain_preserving_mode(domain_start, domain_end)
    }
}
