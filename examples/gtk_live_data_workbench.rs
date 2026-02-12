#[path = "shared/mod.rs"]
mod shared;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use chart_rs::api::{
    ChartEngine, ChartEngineConfig, CrosshairLabelSourceMode, PriceScaleRealtimeBehavior,
    RenderStyle, TimeAxisLabelConfig, TimeAxisLabelPolicy,
};
use chart_rs::core::{DataPoint, OhlcBar, TimeScaleTuning, Viewport};
use chart_rs::extensions::{ChartPlugin, PluginContext, PluginEvent};
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::{CairoRenderer, Color};
use gtk4 as gtk;
use gtk4::prelude::*;

#[derive(Debug, Default)]
struct PluginCounters {
    data_updates: u64,
    candle_updates: u64,
    pointer_moves: u64,
    range_updates: u64,
    rendered: u64,
}

struct CounterPlugin {
    counters: Rc<RefCell<PluginCounters>>,
}

impl CounterPlugin {
    fn new(counters: Rc<RefCell<PluginCounters>>) -> Self {
        Self { counters }
    }
}

impl ChartPlugin for CounterPlugin {
    fn id(&self) -> &str {
        "live-counter"
    }

    fn on_event(&mut self, event: PluginEvent, _context: PluginContext) {
        let mut counters = self.counters.borrow_mut();
        match event {
            PluginEvent::DataUpdated { .. } => counters.data_updates += 1,
            PluginEvent::CandlesUpdated { .. } => counters.candle_updates += 1,
            PluginEvent::PointerMoved { .. } => counters.pointer_moves += 1,
            PluginEvent::VisibleRangeChanged { .. } => counters.range_updates += 1,
            PluginEvent::Rendered => counters.rendered += 1,
            PluginEvent::PointerLeft | PluginEvent::PanStarted | PluginEvent::PanEnded => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LiveState {
    next_time: f64,
    last_price: f64,
    tick: u64,
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.gtk_live_data_workbench")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let (seed_points, seed_candles, seed_state) = match seed_dataset() {
        Ok(seed) => seed,
        Err(err) => {
            eprintln!("failed to build seed dataset: {err}");
            return;
        }
    };

    let renderer = match CairoRenderer::new(1280, 820) {
        Ok(renderer) => renderer,
        Err(err) => {
            eprintln!("failed to initialize cairo renderer: {err}");
            return;
        }
    };

    let config = ChartEngineConfig::new(Viewport::new(1280, 820), 0.0, 320.0)
        .with_price_domain(90.0, 200.0)
        .with_crosshair_mode(chart_rs::interaction::CrosshairMode::Normal);

    let mut engine = match ChartEngine::new(renderer, config) {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("failed to initialize live workbench engine: {err}");
            return;
        }
    };
    engine.set_price_scale_realtime_behavior(PriceScaleRealtimeBehavior {
        autoscale_on_data_set: true,
        autoscale_on_data_update: true,
        autoscale_on_time_range_change: true,
    });

    engine.set_data(seed_points.clone());
    engine.set_candles(seed_candles.clone());
    if let Err(err) = engine.fit_time_to_data(TimeScaleTuning::default()) {
        eprintln!("fit_time_to_data failed: {err}");
    }
    if let Err(err) = engine.autoscale_price_from_candles() {
        eprintln!("autoscale_price_from_candles failed: {err}");
    }

    if let Err(err) = engine.set_time_axis_label_config(TimeAxisLabelConfig {
        policy: TimeAxisLabelPolicy::LogicalDecimal { precision: 0 },
        ..TimeAxisLabelConfig::default()
    }) {
        eprintln!("set_time_axis_label_config failed: {err}");
    }

    let style = RenderStyle {
        show_crosshair_time_label_box: true,
        show_crosshair_price_label_box: true,
        crosshair_time_label_box_color: Some(Color::rgba(0.11, 0.24, 0.55, 0.92)),
        crosshair_price_label_box_color: Some(Color::rgba(0.50, 0.16, 0.10, 0.92)),
        ..engine.render_style()
    };
    if let Err(err) = engine.set_render_style(style) {
        eprintln!("set_render_style failed: {err}");
    }

    engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, context| {
        let src = match context.source_mode {
            CrosshairLabelSourceMode::SnappedData => "snap",
            CrosshairLabelSourceMode::PointerProjected => "ptr",
        };
        format!(
            "time={value:.1} [{src}] span={:.1}",
            context.visible_span_abs
        )
    }));

    let counters = Rc::new(RefCell::new(PluginCounters::default()));
    if let Err(err) = engine.register_plugin(Box::new(CounterPlugin::new(Rc::clone(&counters)))) {
        eprintln!("failed to register counter plugin: {err}");
    }

    let adapter = chart_rs::platform_gtk::GtkChartAdapter::new(engine);

    let snapshot_bytes = Rc::new(Cell::new(0usize));
    adapter.set_snapshot_json_hook(7.0, {
        let snapshot_bytes = Rc::clone(&snapshot_bytes);
        move |snapshot_json| {
            snapshot_bytes.set(snapshot_json.len());
        }
    });

    let diagnostics_text = Rc::new(RefCell::new(String::new()));
    adapter.set_crosshair_diagnostics_hook({
        let diagnostics_text = Rc::clone(&diagnostics_text);
        move |diagnostics| {
            *diagnostics_text.borrow_mut() = format!(
                "diag(gen_t/gen_p={}/{}, cache_t={}/{}, cache_p={}/{})",
                diagnostics.time_formatter_generation,
                diagnostics.price_formatter_generation,
                diagnostics.time_cache.hits,
                diagnostics.time_cache.misses,
                diagnostics.price_cache.hits,
                diagnostics.price_cache.misses,
            );
        }
    });

    let engine = adapter.engine();
    let drawing_area = adapter.drawing_area().clone();
    shared::attach_default_interactions(&drawing_area, Rc::clone(&engine));

    let running = Rc::new(Cell::new(true));
    let follow_tail = Rc::new(Cell::new(true));
    let live_state = Rc::new(RefCell::new(seed_state));

    let run_toggle = gtk::CheckButton::with_label("Live Feed Running");
    run_toggle.set_active(true);

    let follow_tail_toggle = gtk::CheckButton::with_label("Follow Tail");
    follow_tail_toggle.set_active(true);

    let normal_crosshair_toggle = gtk::CheckButton::with_label("Normal crosshair (off=Magnet)");
    normal_crosshair_toggle.set_active(true);

    let reset_button = gtk::Button::with_label("Reset Seed Data");
    let clear_caches_button = gtk::Button::with_label("Clear Formatter Caches");

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.append(&run_toggle);
    controls.append(&follow_tail_toggle);
    controls.append(&normal_crosshair_toggle);
    controls.append(&reset_button);
    controls.append(&clear_caches_button);

    {
        let running = Rc::clone(&running);
        run_toggle.connect_toggled(move |toggle| {
            running.set(toggle.is_active());
        });
    }

    {
        let follow_tail = Rc::clone(&follow_tail);
        follow_tail_toggle.connect_toggled(move |toggle| {
            follow_tail.set(toggle.is_active());
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        normal_crosshair_toggle.connect_toggled(move |toggle| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                if toggle.is_active() {
                    chart.set_crosshair_mode(CrosshairMode::Normal);
                } else {
                    chart.set_crosshair_mode(CrosshairMode::Magnet);
                }
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let live_state = Rc::clone(&live_state);
        let seed_points = seed_points.clone();
        let seed_candles = seed_candles.clone();
        reset_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                chart.set_data(seed_points.clone());
                chart.set_candles(seed_candles.clone());
                let _ = chart.fit_time_to_data(TimeScaleTuning::default());
                let _ = chart.autoscale_price_from_candles();
            }
            *live_state.borrow_mut() = seed_state;
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        clear_caches_button.connect_clicked(move |_| {
            if let Ok(chart) = engine.try_borrow() {
                chart.clear_crosshair_formatter_caches();
            }
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let running = Rc::clone(&running);
        let follow_tail = Rc::clone(&follow_tail);
        let live_state = Rc::clone(&live_state);
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(120), move || {
            if !running.get() {
                return gtk::glib::ControlFlow::Continue;
            }

            let mut state = live_state.borrow_mut();
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let phase = state.tick as f64;
                let wave = (phase / 6.0).sin() * 0.85 + (phase / 19.0).cos() * 0.45;
                let drift = (phase / 140.0).sin() * 0.15;
                let close = (state.last_price + wave + drift).max(0.1);
                let amplitude = 0.6 + (phase / 11.0).sin().abs() * 1.3;
                let high = close.max(state.last_price) + amplitude;
                let low = (close.min(state.last_price) - amplitude).max(0.01);

                chart.append_point(DataPoint::new(state.next_time, close));
                if let Ok(candle) =
                    OhlcBar::new(state.next_time, state.last_price, high, low, close)
                {
                    chart.append_candle(candle);
                }
                let _ = chart.autoscale_price_from_candles();

                if follow_tail.get() {
                    let (visible_start, visible_end) = chart.time_visible_range();
                    let span = (visible_end - visible_start).max(60.0);
                    let new_end = state.next_time + 6.0;
                    let new_start = new_end - span;
                    let _ = chart.set_time_visible_range(new_start, new_end);
                }
            }

            state.next_time += 1.0;
            state.last_price = (state.last_price + (phase_drift(state.tick) * 0.03)).max(0.1);
            state.tick += 1;

            drawing_area.queue_draw();
            gtk::glib::ControlFlow::Continue
        });
    }

    let status_label = gtk::Label::new(None);
    status_label.set_xalign(0.0);
    {
        let engine = Rc::clone(&engine);
        let counters = Rc::clone(&counters);
        let diagnostics_text = Rc::clone(&diagnostics_text);
        let snapshot_bytes = Rc::clone(&snapshot_bytes);
        let running = Rc::clone(&running);
        let follow_tail = Rc::clone(&follow_tail);
        let status_label = status_label.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
            if let Ok(chart) = engine.try_borrow() {
                let (time_start, time_end) = chart.time_visible_range();
                let (price_min, price_max) = chart.price_domain();
                let counters = counters.borrow();

                status_label.set_text(&format!(
                    "run={} follow_tail={} t=[{time_start:.1}, {time_end:.1}] p=[{price_min:.2}, {price_max:.2}] events(data={}, candles={}, move={}, range={}, rendered={}) snapshot_bytes={} {}",
                    running.get(),
                    follow_tail.get(),
                    counters.data_updates,
                    counters.candle_updates,
                    counters.pointer_moves,
                    counters.range_updates,
                    counters.rendered,
                    snapshot_bytes.get(),
                    diagnostics_text.borrow(),
                ));
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    let instructions = gtk::Label::new(Some(
        "Live workbench: stream sintético + eventos plugin + hooks de diagnostics/snapshot. Mouse: mover/rueda/drag.",
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
    root.append(adapter.drawing_area());

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("chart-rs GTK Live Data Workbench")
        .default_width(1280)
        .default_height(860)
        .build();
    window.set_child(Some(&root));
    window.present();
}

fn seed_dataset() -> chart_rs::ChartResult<(Vec<DataPoint>, Vec<OhlcBar>, LiveState)> {
    let points = shared::build_wave_points(260, 0.0, 1.0, 124.0);
    let candles = shared::build_candles_from_points(&points)?;

    let last_time = points.last().map(|point| point.x).unwrap_or(0.0);
    let last_price = points.last().map(|point| point.y).unwrap_or(124.0);

    Ok((
        points,
        candles,
        LiveState {
            next_time: last_time + 1.0,
            last_price,
            tick: 0,
        },
    ))
}

fn phase_drift(tick: u64) -> f64 {
    let phase = tick as f64;
    (phase / 9.0).sin() + (phase / 37.0).cos()
}
