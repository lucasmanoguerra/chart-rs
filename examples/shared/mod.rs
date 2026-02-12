use std::cell::{Cell, RefCell};
use std::rc::Rc;

use chart_rs::api::ChartEngine;
use chart_rs::core::{DataPoint, OhlcBar};
use chart_rs::error::ChartResult;
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::CairoRenderer;
use gtk4 as gtk;
use gtk4::prelude::*;

pub mod binance;

pub type UiEngine = Rc<RefCell<ChartEngine<CairoRenderer>>>;

const PRICE_AXIS_HIT_SLOP_PX: f64 = 18.0;
const AXIS_ZOOM_STEP_RATIO: f64 = 0.18;
const AXIS_ZOOM_MIN_SPAN: f64 = 0.5;
const MIN_PLOT_WIDTH_PX: f64 = 80.0;
const MIN_PLOT_HEIGHT_PX: f64 = 56.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragGestureMode {
    Pan,
    PriceAxisScale,
    TimeAxisScale,
}

#[derive(Clone, Copy, Debug)]
struct InteractionZones {
    plot_right: f64,
    plot_bottom: f64,
}

impl InteractionZones {
    fn from_chart(
        chart: &ChartEngine<CairoRenderer>,
        drawing_area: &gtk::DrawingArea,
    ) -> InteractionZones {
        let viewport_width = f64::from(drawing_area.allocated_width().max(0));
        let viewport_height = f64::from(drawing_area.allocated_height().max(0));
        let style = chart.render_style();
        let max_price_axis_width = (viewport_width - MIN_PLOT_WIDTH_PX).max(0.0);
        let max_time_axis_height = (viewport_height - MIN_PLOT_HEIGHT_PX).max(0.0);
        let price_axis_width = style.price_axis_width_px.clamp(0.0, max_price_axis_width);
        let time_axis_height = style.time_axis_height_px.clamp(0.0, max_time_axis_height);
        let plot_right = (viewport_width - price_axis_width).clamp(0.0, viewport_width);
        let plot_bottom = (viewport_height - time_axis_height).clamp(0.0, viewport_height);
        InteractionZones {
            plot_right,
            plot_bottom,
        }
    }

    fn is_on_price_axis(self, x: f64) -> bool {
        x >= (self.plot_right - PRICE_AXIS_HIT_SLOP_PX).max(0.0)
    }

    fn is_on_time_axis(self, y: f64) -> bool {
        y >= self.plot_bottom
    }
}

fn apply_axis_time_scale_drag(
    chart: &mut ChartEngine<CairoRenderer>,
    drag_delta_x_px: f64,
    anchor_x_px: f64,
) -> f64 {
    if drag_delta_x_px == 0.0 {
        return 1.0;
    }
    chart
        .axis_drag_scale_time(
            drag_delta_x_px,
            anchor_x_px,
            AXIS_ZOOM_STEP_RATIO,
            AXIS_ZOOM_MIN_SPAN,
        )
        .unwrap_or(1.0)
}

fn autoscale_price_from_visible_window(chart: &mut ChartEngine<CairoRenderer>) {
    if !chart.candles().is_empty() {
        let _ = chart.autoscale_price_from_visible_candles();
    } else if !chart.points().is_empty() {
        let _ = chart.autoscale_price_from_visible_data();
    }
}

#[must_use]
pub fn build_wave_points(
    sample_count: usize,
    start_time: f64,
    step: f64,
    base_price: f64,
) -> Vec<DataPoint> {
    (0..sample_count)
        .map(|index| {
            let x = start_time + (index as f64) * step;
            let trend = (index as f64) * 0.035;
            let fast = (x / 4.0).sin() * 2.8;
            let slow = (x / 21.0).cos() * 8.5;
            let burst = ((x / 60.0).sin() + 1.0) * (x / 7.0).sin() * 1.6;
            DataPoint::new(x, (base_price + trend + fast + slow + burst).max(0.1))
        })
        .collect()
}

#[allow(dead_code)]
pub fn build_candles_from_points(points: &[DataPoint]) -> ChartResult<Vec<OhlcBar>> {
    if points.is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::with_capacity(points.len());
    let mut previous_close = points[0].y;

    for (index, point) in points.iter().copied().enumerate() {
        let volatility = 1.4 + ((index as f64) / 9.0).sin().abs() * 2.2;
        let open = previous_close;
        let close = point.y;
        let high = open.max(close) + volatility;
        let low = (open.min(close) - volatility * 0.85).max(0.0001);
        out.push(OhlcBar::new(point.x, open, high, low, close)?);
        previous_close = close;
    }

    Ok(out)
}

pub fn attach_default_interactions(drawing_area: &gtk::DrawingArea, engine: UiEngine) {
    drawing_area.set_focusable(true);
    if let Ok(mut chart) = engine.try_borrow_mut() {
        chart.set_crosshair_mode(CrosshairMode::Normal);
    }

    let pointer_x = Rc::new(Cell::new(0.0));
    let pointer_y = Rc::new(Cell::new(0.0));

    let motion = gtk::EventControllerMotion::new();
    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let pointer_x = Rc::clone(&pointer_x);
        let pointer_y = Rc::clone(&pointer_y);
        motion.connect_motion(move |_, x, y| {
            pointer_x.set(x);
            pointer_y.set(y);
            if let Ok(mut chart) = engine.try_borrow_mut() {
                chart.pointer_move(x, y);
            }
            drawing_area.queue_draw();
        });
    }
    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        motion.connect_leave(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                chart.pointer_leave();
            }
            drawing_area.queue_draw();
        });
    }
    drawing_area.add_controller(motion);

    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::BOTH_AXES);
    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let pointer_x = Rc::clone(&pointer_x);
        let pointer_y = Rc::clone(&pointer_y);
        scroll.connect_scroll(move |_, dx, dy| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let x = pointer_x.get();
                let y = pointer_y.get();
                let zones = InteractionZones::from_chart(&chart, &drawing_area);
                let mut time_range_changed = false;
                if zones.is_on_price_axis(x) {
                    if dy.abs() > f64::EPSILON {
                        let _ =
                            chart.axis_drag_scale_price(dy * 120.0, y, AXIS_ZOOM_STEP_RATIO, 1e-6);
                    }
                } else if zones.is_on_time_axis(y) {
                    let axis_delta = if dx.abs() > f64::EPSILON { dx } else { dy };
                    if axis_delta.abs() > f64::EPSILON {
                        let factor = apply_axis_time_scale_drag(&mut chart, axis_delta * 120.0, x);
                        time_range_changed |= (factor - 1.0).abs() > f64::EPSILON;
                    }
                } else {
                    if dx.abs() > f64::EPSILON {
                        let delta = chart
                            .wheel_pan_time_visible(dx * 120.0, 0.12)
                            .unwrap_or(0.0);
                        time_range_changed |= delta.abs() > f64::EPSILON;
                    }
                    if dy.abs() > f64::EPSILON {
                        let factor = chart
                            .wheel_zoom_time_visible(
                                dy * 120.0,
                                pointer_x.get(),
                                AXIS_ZOOM_STEP_RATIO,
                                AXIS_ZOOM_MIN_SPAN,
                            )
                            .unwrap_or(1.0);
                        time_range_changed |= (factor - 1.0).abs() > f64::EPSILON;
                    }
                }
                if time_range_changed {
                    autoscale_price_from_visible_window(&mut chart);
                }
            }
            drawing_area.queue_draw();
            gtk::glib::Propagation::Stop
        });
    }
    drawing_area.add_controller(scroll);

    let drag = gtk::GestureDrag::new();
    let last_pan_offset_x = Rc::new(Cell::new(0.0));
    let last_price_axis_offset_y = Rc::new(Cell::new(0.0));
    let last_time_axis_offset_x = Rc::new(Cell::new(0.0));
    let drag_mode = Rc::new(Cell::new(DragGestureMode::Pan));

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let last_pan_offset_x = Rc::clone(&last_pan_offset_x);
        let last_price_axis_offset_y = Rc::clone(&last_price_axis_offset_y);
        let last_time_axis_offset_x = Rc::clone(&last_time_axis_offset_x);
        let drag_mode = Rc::clone(&drag_mode);
        drag.connect_drag_begin(move |_, start_x, start_y| {
            last_pan_offset_x.set(0.0);
            last_price_axis_offset_y.set(0.0);
            last_time_axis_offset_x.set(0.0);

            let mut mode = DragGestureMode::Pan;
            if let Ok(chart) = engine.try_borrow() {
                let zones = InteractionZones::from_chart(&chart, &drawing_area);
                if zones.is_on_price_axis(start_x) {
                    mode = DragGestureMode::PriceAxisScale;
                } else if zones.is_on_time_axis(start_y) {
                    mode = DragGestureMode::TimeAxisScale;
                }
            }
            drag_mode.set(mode);

            if mode == DragGestureMode::Pan {
                if let Ok(mut chart) = engine.try_borrow_mut() {
                    chart.pan_start();
                }
            }
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let pointer_x = Rc::clone(&pointer_x);
        let pointer_y = Rc::clone(&pointer_y);
        let last_pan_offset_x = Rc::clone(&last_pan_offset_x);
        let last_price_axis_offset_y = Rc::clone(&last_price_axis_offset_y);
        let last_time_axis_offset_x = Rc::clone(&last_time_axis_offset_x);
        let drag_mode = Rc::clone(&drag_mode);
        drag.connect_drag_update(move |gesture, offset_x, offset_y| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let mut time_range_changed = false;
                if let Some((start_x, start_y)) = gesture.start_point() {
                    let x = start_x + offset_x;
                    let y = start_y + offset_y;

                    match drag_mode.get() {
                        DragGestureMode::Pan => {
                            let delta_x = offset_x - last_pan_offset_x.get();
                            last_pan_offset_x.set(offset_x);
                            let _ = chart.pan_time_visible_by_pixels(delta_x);
                            time_range_changed |= delta_x.abs() > f64::EPSILON;
                        }
                        DragGestureMode::PriceAxisScale => {
                            let delta_y = offset_y - last_price_axis_offset_y.get();
                            last_price_axis_offset_y.set(offset_y);
                            let _ =
                                chart.axis_drag_scale_price(delta_y, y, AXIS_ZOOM_STEP_RATIO, 1e-6);
                        }
                        DragGestureMode::TimeAxisScale => {
                            let delta_x = offset_x - last_time_axis_offset_x.get();
                            last_time_axis_offset_x.set(offset_x);
                            let factor = apply_axis_time_scale_drag(&mut chart, delta_x, x);
                            time_range_changed |= (factor - 1.0).abs() > f64::EPSILON;
                        }
                    }

                    pointer_x.set(x);
                    pointer_y.set(y);
                    chart.pointer_move(x, y);
                }
                if time_range_changed {
                    autoscale_price_from_visible_window(&mut chart);
                }
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let last_pan_offset_x = Rc::clone(&last_pan_offset_x);
        let last_price_axis_offset_y = Rc::clone(&last_price_axis_offset_y);
        let last_time_axis_offset_x = Rc::clone(&last_time_axis_offset_x);
        let drag_mode = Rc::clone(&drag_mode);
        drag.connect_drag_end(move |_, _, _| {
            let mode = drag_mode.get();
            last_pan_offset_x.set(0.0);
            last_price_axis_offset_y.set(0.0);
            last_time_axis_offset_x.set(0.0);
            if mode == DragGestureMode::Pan {
                if let Ok(mut chart) = engine.try_borrow_mut() {
                    chart.pan_end();
                }
            }
            drag_mode.set(DragGestureMode::Pan);
            drawing_area.queue_draw();
        });
    }

    drawing_area.add_controller(drag);

    let axis_reset_click = gtk::GestureClick::new();
    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        axis_reset_click.connect_pressed(move |_, n_press, x, _y| {
            if n_press != 2 {
                return;
            }

            let mut on_price_axis = false;
            if let Ok(chart) = engine.try_borrow() {
                let zones = InteractionZones::from_chart(&chart, &drawing_area);
                on_price_axis = zones.is_on_price_axis(x);
            }
            if !on_price_axis {
                return;
            }

            if let Ok(mut chart) = engine.try_borrow_mut() {
                let _ = chart.axis_double_click_reset_price_scale();
            }
            drawing_area.queue_draw();
        });
    }
    drawing_area.add_controller(axis_reset_click);
}
