#[path = "shared/mod.rs"]
mod shared;

use std::rc::Rc;
use std::sync::Arc;

use chart_rs::api::{
    ChartEngine, ChartEngineConfig, CrosshairLabelSourceMode, InteractionInputBehavior,
    PriceAxisLabelConfig, PriceAxisLabelPolicy, PriceScaleMarginBehavior, RenderStyle,
    TimeAxisLabelConfig, TimeAxisLabelPolicy, TimeScaleEdgeBehavior, TimeScaleNavigationBehavior,
    TimeScaleResizeAnchor, TimeScaleResizeBehavior,
};
use chart_rs::core::{PriceScaleMode, TimeScaleTuning, Viewport};
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::{CairoRenderer, Color, LineStrokeStyle};
use gtk4 as gtk;
use gtk4::prelude::*;

fn main() {
    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.gtk_interaction_lab")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let engine = match build_engine() {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("failed to initialize interaction lab engine: {err}");
            return;
        }
    };

    let diagnostics_label = gtk::Label::new(None);
    diagnostics_label.set_xalign(0.0);

    let adapter = chart_rs::platform_gtk::GtkChartAdapter::new(engine);
    adapter.set_crosshair_diagnostics_hook({
        let diagnostics_label = diagnostics_label.clone();
        move |diagnostics| {
            diagnostics_label.set_text(&format!(
                "diag: mode=({:?}/{:?}) gen=({}/{}) cache_t=({}/{}/{}) cache_p=({}/{}/{})",
                diagnostics.time_override_mode,
                diagnostics.price_override_mode,
                diagnostics.time_formatter_generation,
                diagnostics.price_formatter_generation,
                diagnostics.time_cache.hits,
                diagnostics.time_cache.misses,
                diagnostics.time_cache.size,
                diagnostics.price_cache.hits,
                diagnostics.price_cache.misses,
                diagnostics.price_cache.size,
            ));
        }
    });

    let engine = adapter.engine();
    let drawing_area = adapter.drawing_area().clone();
    shared::attach_default_interactions(&drawing_area, Rc::clone(&engine));

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    let crosshair_mode_toggle = gtk::ToggleButton::with_label("Normal Crosshair");
    crosshair_mode_toggle.set_active(true);
    controls.append(&crosshair_mode_toggle);

    let log_scale_toggle = gtk::ToggleButton::with_label("Log Price Scale");
    controls.append(&log_scale_toggle);

    let invert_scale_toggle = gtk::ToggleButton::with_label("Invert Price Scale");
    controls.append(&invert_scale_toggle);

    let fit_button = gtk::Button::with_label("Fit Data");
    controls.append(&fit_button);

    let autoscale_button = gtk::Button::with_label("Autoscale Price");
    controls.append(&autoscale_button);

    let reset_button = gtk::Button::with_label("Reset Time Range");
    controls.append(&reset_button);

    let fix_left_edge_toggle = gtk::CheckButton::with_label("Fix Left Edge");
    controls.append(&fix_left_edge_toggle);

    let fix_right_edge_toggle = gtk::CheckButton::with_label("Fix Right Edge");
    controls.append(&fix_right_edge_toggle);

    let handle_scroll_toggle = gtk::CheckButton::with_label("Handle Scroll");
    handle_scroll_toggle.set_active(true);
    controls.append(&handle_scroll_toggle);

    let handle_scale_toggle = gtk::CheckButton::with_label("Handle Scale");
    handle_scale_toggle.set_active(true);
    controls.append(&handle_scale_toggle);

    let top_margin_spin = gtk::SpinButton::with_range(0.0, 0.45, 0.01);
    top_margin_spin.set_value(0.0);
    top_margin_spin.set_width_chars(4);
    controls.append(&gtk::Label::new(Some("Top Margin")));
    controls.append(&top_margin_spin);

    let bottom_margin_spin = gtk::SpinButton::with_range(0.0, 0.45, 0.01);
    bottom_margin_spin.set_value(0.0);
    bottom_margin_spin.set_width_chars(4);
    controls.append(&gtk::Label::new(Some("Bottom Margin")));
    controls.append(&bottom_margin_spin);

    let apply_margin_button = gtk::Button::with_label("Apply Price Margins");
    controls.append(&apply_margin_button);

    let right_offset_spin = gtk::SpinButton::with_range(-40.0, 80.0, 0.5);
    right_offset_spin.set_value(0.0);
    right_offset_spin.set_width_chars(5);
    controls.append(&gtk::Label::new(Some("Right Offset")));
    controls.append(&right_offset_spin);

    let bar_spacing_toggle = gtk::CheckButton::with_label("Bar Spacing");
    controls.append(&bar_spacing_toggle);

    let bar_spacing_spin = gtk::SpinButton::with_range(2.0, 80.0, 1.0);
    bar_spacing_spin.set_value(20.0);
    bar_spacing_spin.set_sensitive(false);
    bar_spacing_spin.set_width_chars(4);
    controls.append(&bar_spacing_spin);

    let apply_nav_button = gtk::Button::with_label("Apply Time Nav");
    controls.append(&apply_nav_button);

    let resize_lock_toggle = gtk::CheckButton::with_label("Lock Resize");
    resize_lock_toggle.set_active(true);
    controls.append(&resize_lock_toggle);

    let resize_anchor_combo = gtk::ComboBoxText::new();
    resize_anchor_combo.append(Some("right"), "Resize Anchor: Right");
    resize_anchor_combo.append(Some("left"), "Resize Anchor: Left");
    resize_anchor_combo.append(Some("center"), "Resize Anchor: Center");
    resize_anchor_combo.set_active_id(Some("right"));
    controls.append(&resize_anchor_combo);

    let apply_resize_button = gtk::Button::with_label("Apply Resize Policy");
    controls.append(&apply_resize_button);

    let clear_cache_button = gtk::Button::with_label("Clear Crosshair Caches");
    controls.append(&clear_cache_button);

    let status_label = gtk::Label::new(None);
    status_label.set_xalign(0.0);

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        crosshair_mode_toggle.connect_toggled(move |button| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let mode = if button.is_active() {
                    CrosshairMode::Normal
                } else {
                    CrosshairMode::Magnet
                };
                chart.set_crosshair_mode(mode);
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        log_scale_toggle.connect_toggled(move |button| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let mode = if button.is_active() {
                    PriceScaleMode::Log
                } else {
                    PriceScaleMode::Linear
                };
                if let Err(err) = chart.set_price_scale_mode(mode) {
                    eprintln!("failed to switch price scale mode: {err}");
                } else {
                    let _ = chart.autoscale_price_from_data();
                }
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        invert_scale_toggle.connect_toggled(move |button| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                chart.set_price_scale_inverted(button.is_active());
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        fit_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let _ = chart.fit_time_to_data(TimeScaleTuning::default());
                let _ = chart.autoscale_price_from_data();
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        autoscale_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let _ = chart.autoscale_price_from_data();
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        reset_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                chart.reset_time_visible_range();
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let fix_left_edge_state = fix_left_edge_toggle.clone();
        let fix_right_edge_state = fix_right_edge_toggle.clone();
        fix_left_edge_toggle.clone().connect_toggled(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let _ = chart.set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
                    fix_left_edge: fix_left_edge_state.is_active(),
                    fix_right_edge: fix_right_edge_state.is_active(),
                });
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let fix_left_edge_state = fix_left_edge_toggle.clone();
        let fix_right_edge_state = fix_right_edge_toggle.clone();
        fix_right_edge_toggle.clone().connect_toggled(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let _ = chart.set_time_scale_edge_behavior(TimeScaleEdgeBehavior {
                    fix_left_edge: fix_left_edge_state.is_active(),
                    fix_right_edge: fix_right_edge_state.is_active(),
                });
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let handle_scroll_state = handle_scroll_toggle.clone();
        let handle_scale_state = handle_scale_toggle.clone();
        handle_scroll_toggle.clone().connect_toggled(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let current = chart.interaction_input_behavior();
                chart.set_interaction_input_behavior(InteractionInputBehavior {
                    handle_scroll: handle_scroll_state.is_active(),
                    handle_scale: handle_scale_state.is_active(),
                    ..current
                });
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let handle_scroll_state = handle_scroll_toggle.clone();
        let handle_scale_state = handle_scale_toggle.clone();
        handle_scale_toggle.clone().connect_toggled(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let current = chart.interaction_input_behavior();
                chart.set_interaction_input_behavior(InteractionInputBehavior {
                    handle_scroll: handle_scroll_state.is_active(),
                    handle_scale: handle_scale_state.is_active(),
                    ..current
                });
            }
            drawing_area.queue_draw();
        });
    }

    {
        let bar_spacing_spin = bar_spacing_spin.clone();
        bar_spacing_toggle.connect_toggled(move |toggle| {
            bar_spacing_spin.set_sensitive(toggle.is_active());
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let top_margin_spin = top_margin_spin.clone();
        let bottom_margin_spin = bottom_margin_spin.clone();
        apply_margin_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let _ = chart.set_price_scale_margin_behavior(PriceScaleMarginBehavior {
                    top_margin_ratio: top_margin_spin.value(),
                    bottom_margin_ratio: bottom_margin_spin.value(),
                });
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let right_offset_spin = right_offset_spin.clone();
        let bar_spacing_toggle = bar_spacing_toggle.clone();
        let bar_spacing_spin = bar_spacing_spin.clone();
        apply_nav_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let behavior = TimeScaleNavigationBehavior {
                    right_offset_bars: right_offset_spin.value(),
                    bar_spacing_px: if bar_spacing_toggle.is_active() {
                        Some(bar_spacing_spin.value())
                    } else {
                        None
                    },
                };
                let _ = chart.set_time_scale_navigation_behavior(behavior);
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let resize_lock_toggle = resize_lock_toggle.clone();
        let resize_anchor_combo = resize_anchor_combo.clone();
        apply_resize_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let anchor = match resize_anchor_combo.active_id().as_deref() {
                    Some("left") => TimeScaleResizeAnchor::Left,
                    Some("center") => TimeScaleResizeAnchor::Center,
                    _ => TimeScaleResizeAnchor::Right,
                };
                let _ = chart.set_time_scale_resize_behavior(TimeScaleResizeBehavior {
                    lock_visible_range_on_resize: resize_lock_toggle.is_active(),
                    anchor,
                });
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        clear_cache_button.connect_clicked(move |_| {
            if let Ok(chart) = engine.try_borrow() {
                chart.clear_crosshair_formatter_caches();
            }
        });
    }

    {
        let engine = Rc::clone(&engine);
        let status_label = status_label.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(120), move || {
            if let Ok(chart) = engine.try_borrow() {
                let (time_start, time_end) = chart.time_visible_range();
                let (price_min, price_max) = chart.price_domain();
                let edge_behavior = chart.time_scale_edge_behavior();
                let navigation_behavior = chart.time_scale_navigation_behavior();
                let resize_behavior = chart.time_scale_resize_behavior();
                let input_behavior = chart.interaction_input_behavior();
                let margin_behavior = chart.price_scale_margin_behavior();
                status_label.set_text(&format!(
                    "range: t=[{time_start:.2}, {time_end:.2}] p=[{price_min:.2}, {price_max:.2}] mode={:?} inverted={} margins=({:.2},{:.2}) crosshair={:?} edges(L={},R={}) input(scroll={},scale={}) nav(offset={:.2}, spacing={:?}) resize(lock={}, anchor={:?})",
                    chart.price_scale_mode(),
                    chart.price_scale_inverted(),
                    margin_behavior.top_margin_ratio,
                    margin_behavior.bottom_margin_ratio,
                    chart.crosshair_mode(),
                    edge_behavior.fix_left_edge,
                    edge_behavior.fix_right_edge,
                    input_behavior.handle_scroll,
                    input_behavior.handle_scale,
                    navigation_behavior.right_offset_bars,
                    navigation_behavior.bar_spacing_px,
                    resize_behavior.lock_visible_range_on_resize,
                    resize_behavior.anchor,
                ));
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    let instructions = gtk::Label::new(Some(
        "Mouse: move=Crosshair | wheel=Zoom/Pan | drag=Pan. Toggle controls to inspect interaction behavior.",
    ));
    instructions.set_xalign(0.0);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_margin_top(10);
    root.set_margin_bottom(10);
    root.set_margin_start(10);
    root.set_margin_end(10);
    root.append(&instructions);
    root.append(&controls);
    root.append(&status_label);
    root.append(&diagnostics_label);
    root.append(adapter.drawing_area());

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("chart-rs GTK Interaction Lab")
        .default_width(1280)
        .default_height(800)
        .build();
    window.set_child(Some(&root));
    window.present();
}

fn build_engine() -> chart_rs::ChartResult<ChartEngine<CairoRenderer>> {
    let renderer = CairoRenderer::new(1280, 800)?;
    let config = ChartEngineConfig::new(Viewport::new(1280, 800), 0.0, 720.0)
        .with_price_domain(90.0, 200.0)
        .with_crosshair_mode(chart_rs::interaction::CrosshairMode::Normal);
    let mut engine = ChartEngine::new(renderer, config)?;

    let points = shared::build_wave_points(720, 0.0, 1.0, 124.0);
    engine.set_data(points);
    engine.fit_time_to_data(TimeScaleTuning::default())?;
    engine.autoscale_price_from_data()?;

    engine.set_time_axis_label_config(TimeAxisLabelConfig {
        policy: TimeAxisLabelPolicy::LogicalDecimal { precision: 0 },
        ..TimeAxisLabelConfig::default()
    })?;

    engine.set_price_axis_label_config(PriceAxisLabelConfig {
        policy: PriceAxisLabelPolicy::Adaptive,
        ..PriceAxisLabelConfig::default()
    })?;

    let style = RenderStyle {
        crosshair_line_style: LineStrokeStyle::Dashed,
        crosshair_horizontal_line_color: Some(Color::rgba(0.20, 0.45, 0.95, 0.90)),
        crosshair_vertical_line_color: Some(Color::rgba(0.95, 0.35, 0.20, 0.90)),
        show_crosshair_time_label_box: true,
        show_crosshair_price_label_box: true,
        ..engine.render_style()
    };
    engine.set_render_style(style)?;

    engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, context| {
        let source = match context.source_mode {
            CrosshairLabelSourceMode::SnappedData => "snap",
            CrosshairLabelSourceMode::PointerProjected => "ptr",
        };
        format!(
            "T={value:.2} [{source}] span={:.1}",
            context.visible_span_abs
        )
    }));
    engine.set_crosshair_price_label_formatter_with_context(Arc::new(|value, context| {
        let source = match context.source_mode {
            CrosshairLabelSourceMode::SnappedData => "snap",
            CrosshairLabelSourceMode::PointerProjected => "ptr",
        };
        format!(
            "P={value:.3} [{source}] span={:.1}",
            context.visible_span_abs
        )
    }));

    Ok(engine)
}
