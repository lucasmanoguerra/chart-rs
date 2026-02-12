#[path = "shared/mod.rs"]
mod shared;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use chart_rs::api::{
    ChartEngine, ChartEngineConfig, CrosshairLabelSourceMode, PriceAxisDisplayMode,
    PriceAxisLabelConfig, PriceAxisLabelPolicy, RenderStyle, TimeAxisLabelConfig,
    TimeAxisLabelPolicy,
};
use chart_rs::core::{TimeScaleTuning, Viewport};
use chart_rs::interaction::{CrosshairMode, KineticPanConfig};
use chart_rs::render::{CairoRenderer, Color, LineStrokeStyle};
use gtk4 as gtk;
use gtk4::prelude::*;

use shared::binance::{self, MarketData};

fn main() {
    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.gtk_tradingview_binance")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let initial_data = match binance::fetch_market_data("BTCUSDT", "1h", 500) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("binance fetch failed, using fallback dataset: {err}");
            binance::fallback_market_data("BTCUSDT", "1h")
        }
    };

    let engine = match build_engine(&initial_data) {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("failed to initialize tradingview-like engine: {err}");
            return;
        }
    };

    let engine = Rc::new(RefCell::new(engine));
    let market_data = Rc::new(RefCell::new(initial_data));
    let fetch_error = Rc::new(RefCell::new(String::new()));
    let follow_tail = Rc::new(Cell::new(true));

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    drawing_area.set_draw_func({
        let engine = Rc::clone(&engine);
        let market_data = Rc::clone(&market_data);

        move |_widget, context, width, height| {
            if width <= 0 || height <= 0 {
                return;
            }

            let mut chart = match engine.try_borrow_mut() {
                Ok(chart) => chart,
                Err(_) => return,
            };

            let viewport = Viewport::new(width as u32, height as u32);
            if chart.viewport() != viewport {
                let _ = chart.set_viewport(viewport);
            }

            let _ = chart.render_on_cairo_context(context);

            if let Ok(candles) = chart.project_visible_candles_with_overscan(8.0, 0.04) {
                draw_candles_overlay(context, &candles);
            }

            draw_volume_overlay(
                context,
                &chart,
                &market_data.borrow(),
                width as f64,
                height as f64,
            );
            draw_header_overlay(context, &market_data.borrow(), width as f64);
        }
    });

    shared::attach_default_interactions(&drawing_area, Rc::clone(&engine));

    let symbol_combo = gtk::ComboBoxText::new();
    for symbol in ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT"] {
        symbol_combo.append(Some(symbol), symbol);
    }
    symbol_combo.set_active_id(Some("BTCUSDT"));

    let interval_combo = gtk::ComboBoxText::new();
    for interval in ["1m", "5m", "15m", "1h", "4h", "1d"] {
        interval_combo.append(Some(interval), interval);
    }
    interval_combo.set_active_id(Some("1h"));

    let limit_spin = gtk::SpinButton::with_range(100.0, 1000.0, 50.0);
    limit_spin.set_value(500.0);
    limit_spin.set_width_chars(5);

    let display_mode_combo = gtk::ComboBoxText::new();
    display_mode_combo.append(Some("normal"), "Price: Normal");
    display_mode_combo.append(Some("percent"), "Price: Percent");
    display_mode_combo.append(Some("indexed"), "Price: Indexed 100");
    display_mode_combo.set_active_id(Some("normal"));

    let log_scale_toggle = gtk::CheckButton::with_label("Log scale");
    let follow_tail_toggle = gtk::CheckButton::with_label("Follow tail");
    follow_tail_toggle.set_active(true);

    let fit_button = gtk::Button::with_label("Fit");
    let reset_button = gtk::Button::with_label("Reset range");
    let kinetic_button = gtk::Button::with_label("Kinetic pan demo");
    let reload_button = gtk::Button::with_label("Reload Binance");

    let range_1d_button = gtk::Button::with_label("1D");
    let range_1w_button = gtk::Button::with_label("1W");
    let range_1m_button = gtk::Button::with_label("1M");

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.append(&gtk::Label::new(Some("Symbol")));
    controls.append(&symbol_combo);
    controls.append(&gtk::Label::new(Some("Interval")));
    controls.append(&interval_combo);
    controls.append(&gtk::Label::new(Some("Limit")));
    controls.append(&limit_spin);
    controls.append(&reload_button);
    controls.append(&display_mode_combo);
    controls.append(&log_scale_toggle);
    controls.append(&follow_tail_toggle);
    controls.append(&fit_button);
    controls.append(&reset_button);
    controls.append(&kinetic_button);
    controls.append(&range_1d_button);
    controls.append(&range_1w_button);
    controls.append(&range_1m_button);

    {
        let follow_tail = Rc::clone(&follow_tail);
        follow_tail_toggle.connect_toggled(move |toggle| {
            follow_tail.set(toggle.is_active());
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        display_mode_combo.connect_changed(move |combo| {
            let mode = match combo.active_id().as_deref() {
                Some("percent") => PriceAxisDisplayMode::Percentage { base_price: None },
                Some("indexed") => PriceAxisDisplayMode::IndexedTo100 { base_price: None },
                _ => PriceAxisDisplayMode::Normal,
            };

            if let Ok(mut chart) = engine.try_borrow_mut() {
                let mut config = chart.price_axis_label_config();
                config.policy = PriceAxisLabelPolicy::Adaptive;
                config.display_mode = mode;
                let _ = chart.set_price_axis_label_config(config);
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        log_scale_toggle.connect_toggled(move |toggle| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let mode = if toggle.is_active() {
                    chart_rs::core::PriceScaleMode::Log
                } else {
                    chart_rs::core::PriceScaleMode::Linear
                };
                if chart.set_price_scale_mode(mode).is_ok() {
                    let _ = chart.autoscale_price_from_candles();
                }
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
                let _ = chart.autoscale_price_from_candles();
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
        kinetic_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let _ = chart.set_kinetic_pan_config(KineticPanConfig {
                    decay_per_second: 0.75,
                    stop_velocity_abs: 0.08,
                });
                let _ = chart.start_kinetic_pan(220.0);
            }
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        range_1d_button.connect_clicked(move |_| {
            apply_tail_range(&engine, 86_400.0);
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        range_1w_button.connect_clicked(move |_| {
            apply_tail_range(&engine, 7.0 * 86_400.0);
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        range_1m_button.connect_clicked(move |_| {
            apply_tail_range(&engine, 30.0 * 86_400.0);
            drawing_area.queue_draw();
        });
    }

    let status_label = gtk::Label::new(None);
    status_label.set_xalign(0.0);

    {
        let engine = Rc::clone(&engine);
        let market_data = Rc::clone(&market_data);
        let fetch_error = Rc::clone(&fetch_error);
        let follow_tail = Rc::clone(&follow_tail);
        let status_label = status_label.clone();
        gtk::glib::timeout_add_local(Duration::from_millis(180), move || {
            if let Ok(chart) = engine.try_borrow() {
                let (time_start, time_end) = chart.time_visible_range();
                let (price_min, price_max) = chart.price_domain();
                let visible_candles = chart.visible_candles().len();
                let crosshair = chart.crosshair_state();
                let price_cache = chart.price_label_cache_stats();
                let time_cache = chart.time_label_cache_stats();
                let data = market_data.borrow();

                let mut status = format!(
                    "{} {} source={} visible_candles={} t=[{:.0},{:.0}] p=[{:.2},{:.2}] crosshair={:?} follow_tail={} cache_t={}/{} cache_p={}/{}",
                    data.symbol,
                    data.interval,
                    data.source_label,
                    visible_candles,
                    time_start,
                    time_end,
                    price_min,
                    price_max,
                    chart.crosshair_mode(),
                    follow_tail.get(),
                    time_cache.hits,
                    time_cache.misses,
                    price_cache.hits,
                    price_cache.misses,
                );
                if crosshair.visible {
                    status.push_str(&format!(" cursor=({:.0},{:.0})", crosshair.x, crosshair.y));
                }

                let fetch_error_text = fetch_error.borrow();
                if !fetch_error_text.is_empty() {
                    status.push_str(" | ");
                    status.push_str(fetch_error_text.as_str());
                }
                status_label.set_text(&status);
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        gtk::glib::timeout_add_local(Duration::from_millis(16), move || {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                if chart.kinetic_pan_state().active {
                    let _ = chart.step_kinetic_pan(0.016);
                    drawing_area.queue_draw();
                }
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    {
        let engine = Rc::clone(&engine);
        let market_data = Rc::clone(&market_data);
        let fetch_error = Rc::clone(&fetch_error);
        let follow_tail = Rc::clone(&follow_tail);
        let drawing_area = drawing_area.clone();
        let symbol_combo = symbol_combo.clone();
        let interval_combo = interval_combo.clone();
        let limit_spin = limit_spin.clone();

        reload_button.connect_clicked(move |_| {
            let symbol = symbol_combo
                .active_id()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "BTCUSDT".to_owned());
            let interval = interval_combo
                .active_id()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "1h".to_owned());
            let limit = limit_spin.value().round() as u16;

            match binance::fetch_market_data(&symbol, &interval, limit) {
                Ok(data) => {
                    if let Ok(mut chart) = engine.try_borrow_mut() {
                        apply_market_data(&mut chart, &data, follow_tail.get());
                    }
                    *market_data.borrow_mut() = data;
                    fetch_error.borrow_mut().clear();
                }
                Err(err) => {
                    *fetch_error.borrow_mut() = format!("fetch error: {err}");
                }
            }
            drawing_area.queue_draw();
        });
    }

    let instructions = gtk::Label::new(Some(
        "TradingView-like lab: candles + volume + crosshair normal + zoom/pan + Binance spot klines. Mouse wheel: zoom/pan, drag: pan.",
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
    root.append(&drawing_area);

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("chart-rs Binance TradingView-like")
        .default_width(1420)
        .default_height(900)
        .build();
    window.set_child(Some(&root));
    window.present();
}

fn build_engine(data: &MarketData) -> chart_rs::ChartResult<ChartEngine<CairoRenderer>> {
    let (time_start, time_end) = binance::data_time_range(data);
    let (price_min, price_max) = binance::data_price_range(data);

    let mut renderer = CairoRenderer::new(1420, 900)?;
    renderer.set_clear_color(Color::rgb(0.06, 0.08, 0.11))?;

    let config = ChartEngineConfig::new(Viewport::new(1420, 900), time_start, time_end)
        .with_price_domain(price_min, price_max)
        .with_crosshair_mode(chart_rs::interaction::CrosshairMode::Normal);
    let mut engine = ChartEngine::new(renderer, config)?;
    engine.set_price_scale_realtime_behavior(chart_rs::api::PriceScaleRealtimeBehavior {
        autoscale_on_data_set: true,
        autoscale_on_data_update: true,
        autoscale_on_time_range_change: true,
    });

    apply_market_data(&mut engine, data, true);

    engine.set_time_axis_label_config(TimeAxisLabelConfig {
        policy: TimeAxisLabelPolicy::UtcAdaptive,
        ..TimeAxisLabelConfig::default()
    })?;

    engine.set_price_axis_label_config(PriceAxisLabelConfig {
        policy: PriceAxisLabelPolicy::Adaptive,
        display_mode: PriceAxisDisplayMode::Normal,
        ..PriceAxisLabelConfig::default()
    })?;

    let style = RenderStyle {
        series_line_color: Color::rgba(0.75, 0.80, 0.92, 0.0),
        grid_line_color: Color::rgba(0.22, 0.27, 0.34, 0.55),
        price_axis_grid_line_color: Color::rgba(0.22, 0.27, 0.34, 0.55),
        major_grid_line_color: Color::rgba(0.35, 0.41, 0.50, 0.85),
        axis_border_color: Color::rgba(0.33, 0.39, 0.47, 0.95),
        axis_label_color: Color::rgba(0.80, 0.85, 0.93, 0.95),
        time_axis_label_color: Color::rgba(0.74, 0.80, 0.90, 0.92),
        major_time_label_color: Color::rgba(0.92, 0.95, 0.99, 0.98),
        crosshair_line_color: Color::rgba(0.84, 0.87, 0.93, 0.50),
        crosshair_horizontal_line_color: Some(Color::rgba(0.82, 0.87, 0.95, 0.48)),
        crosshair_vertical_line_color: Some(Color::rgba(0.94, 0.83, 0.63, 0.50)),
        crosshair_line_style: LineStrokeStyle::Dashed,
        show_crosshair_time_label_box: true,
        show_crosshair_price_label_box: true,
        crosshair_time_label_box_color: Some(Color::rgba(0.20, 0.30, 0.50, 0.97)),
        crosshair_price_label_box_color: Some(Color::rgba(0.57, 0.31, 0.18, 0.97)),
        last_price_line_color: Color::rgba(0.98, 0.55, 0.36, 0.95),
        last_price_label_color: Color::rgba(0.98, 0.55, 0.36, 0.98),
        price_axis_width_px: 98.0,
        time_axis_height_px: 34.0,
        ..engine.render_style()
    };
    engine.set_render_style(style)?;

    engine.set_crosshair_mode(CrosshairMode::Normal);
    engine.set_crosshair_time_label_formatter_with_context(Arc::new(|value, context| {
        let src = match context.source_mode {
            CrosshairLabelSourceMode::SnappedData => "snap",
            CrosshairLabelSourceMode::PointerProjected => "ptr",
        };
        format!("{value:.0} [{src}] span={:.0}", context.visible_span_abs)
    }));

    Ok(engine)
}

fn apply_market_data(
    engine: &mut ChartEngine<CairoRenderer>,
    data: &MarketData,
    follow_tail: bool,
) {
    engine.set_data(data.close_points.clone());
    engine.set_candles(data.candles.clone());
    let _ = engine.fit_time_to_data(TimeScaleTuning::default());
    let _ = engine.autoscale_price_from_candles();
    if follow_tail {
        apply_tail_range_rc(engine, binance::default_window_secs(data.interval.as_str()));
    }
}

fn apply_tail_range(engine: &Rc<RefCell<ChartEngine<CairoRenderer>>>, seconds: f64) {
    if let Ok(mut chart) = engine.try_borrow_mut() {
        apply_tail_range_rc(&mut chart, seconds);
    }
}

fn apply_tail_range_rc(engine: &mut ChartEngine<CairoRenderer>, seconds: f64) {
    let (_, end) = engine.time_full_range();
    let start = end - seconds.max(60.0);
    let _ = engine.set_time_visible_range(start, end);
}

fn draw_candles_overlay(context: &gtk::cairo::Context, candles: &[chart_rs::core::CandleGeometry]) {
    for candle in candles {
        let (r, g, b) = if candle.is_bullish {
            (0.00, 0.76, 0.44)
        } else {
            (0.93, 0.29, 0.26)
        };

        context.set_source_rgba(r, g, b, 0.95);
        context.set_line_width(1.0);
        context.move_to(candle.center_x, candle.wick_top);
        context.line_to(candle.center_x, candle.wick_bottom);
        let _ = context.stroke();

        context.set_source_rgba(r, g, b, 0.34);
        context.rectangle(
            candle.body_left,
            candle.body_top,
            (candle.body_right - candle.body_left).max(1.0),
            (candle.body_bottom - candle.body_top).max(1.0),
        );
        let _ = context.fill_preserve();

        context.set_source_rgba(r, g, b, 0.95);
        let _ = context.stroke();
    }
}

fn draw_volume_overlay(
    context: &gtk::cairo::Context,
    chart: &ChartEngine<CairoRenderer>,
    data: &MarketData,
    width: f64,
    height: f64,
) {
    let style = chart.render_style();
    let plot_right = (width - style.price_axis_width_px).clamp(0.0, width);
    let plot_bottom = (height - style.time_axis_height_px).clamp(0.0, height);

    let panel_height = (height * 0.18).clamp(64.0, 180.0);
    let panel_bottom = (plot_bottom - 2.0).max(0.0);
    let panel_top = (panel_bottom - panel_height).max(0.0);

    context.set_source_rgba(0.07, 0.10, 0.14, 0.70);
    context.rectangle(0.0, panel_top, plot_right, panel_bottom - panel_top);
    let _ = context.fill();

    let (visible_start, visible_end) = chart.time_visible_range();

    let mut max_volume = 0.0_f64;
    let mut visible_count = 0usize;
    for point in &data.volumes {
        if point.x >= visible_start && point.x <= visible_end {
            max_volume = max_volume.max(point.y);
            visible_count += 1;
        }
    }
    if max_volume <= 0.0 || visible_count == 0 {
        return;
    }

    let bar_width = ((plot_right / visible_count as f64) * 0.72).clamp(1.0, 9.0);
    let half_width = bar_width * 0.5;

    for (index, point) in data.volumes.iter().enumerate() {
        if point.x < visible_start || point.x > visible_end {
            continue;
        }
        let x = match chart.map_x_to_pixel(point.x) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if x < -bar_width || x > plot_right + bar_width {
            continue;
        }

        let ratio = (point.y / max_volume).clamp(0.0, 1.0);
        let bar_height = ratio * (panel_bottom - panel_top - 1.0);
        let y = panel_bottom - bar_height;

        let (r, g, b) = if *data.volume_up.get(index).unwrap_or(&true) {
            (0.00, 0.62, 0.42)
        } else {
            (0.78, 0.28, 0.24)
        };

        context.set_source_rgba(r, g, b, 0.44);
        context.rectangle(x - half_width, y, bar_width, bar_height.max(1.0));
        let _ = context.fill();
    }

    context.set_source_rgba(0.62, 0.69, 0.82, 0.85);
    context.set_font_size(11.0);
    context.move_to(8.0, panel_top + 14.0);
    let _ = context.show_text("Volume");
}

fn draw_header_overlay(context: &gtk::cairo::Context, data: &MarketData, width: f64) {
    context.set_source_rgba(0.93, 0.95, 0.98, 0.94);
    context.set_font_size(14.0);
    context.move_to(12.0, 22.0);
    let _ = context.show_text(&format!("{}  {}", data.symbol, data.interval));

    context.set_source_rgba(0.62, 0.67, 0.76, 0.88);
    context.set_font_size(11.0);
    context.move_to(width - 180.0, 22.0);
    let _ = context.show_text(&format!("source: {}", data.source_label));
}
