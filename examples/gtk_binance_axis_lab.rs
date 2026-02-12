#[path = "shared/mod.rs"]
mod shared;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use chart_rs::api::{
    PriceAxisDisplayMode, PriceAxisLabelConfig, PriceAxisLabelPolicy, TimeAxisLabelConfig,
    TimeAxisLabelPolicy, TimeAxisSessionConfig, TimeAxisTimeZone,
};
use chart_rs::core::{TimeScaleTuning, Viewport};
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::CairoRenderer;
use gtk4 as gtk;
use gtk4::prelude::*;

use shared::binance;

fn main() {
    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.gtk_binance_axis_lab")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let data = match binance::fetch_market_data("BTCUSDT", "1h", 480) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("binance fetch failed, using fallback dataset: {err}");
            binance::fallback_market_data("BTCUSDT", "1h")
        }
    };

    let (time_start, time_end) = binance::data_time_range(&data);
    let (price_min, price_max) = binance::data_price_range(&data);

    let renderer = match CairoRenderer::new(1320, 860) {
        Ok(renderer) => renderer,
        Err(err) => {
            eprintln!("renderer error: {err}");
            return;
        }
    };

    let config =
        chart_rs::api::ChartEngineConfig::new(Viewport::new(1320, 860), time_start, time_end)
            .with_price_domain(price_min, price_max)
            .with_crosshair_mode(chart_rs::interaction::CrosshairMode::Normal);
    let mut engine = match chart_rs::api::ChartEngine::new(renderer, config) {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("engine init error: {err}");
            return;
        }
    };

    engine.set_data(data.close_points.clone());
    engine.set_candles(data.candles.clone());
    let _ = engine.fit_time_to_data(TimeScaleTuning::default());
    let _ = engine.autoscale_price_from_candles();
    let _ = engine.set_time_axis_label_config(TimeAxisLabelConfig {
        policy: TimeAxisLabelPolicy::UtcAdaptive,
        ..TimeAxisLabelConfig::default()
    });
    let _ = engine.set_price_axis_label_config(PriceAxisLabelConfig {
        policy: PriceAxisLabelPolicy::Adaptive,
        ..PriceAxisLabelConfig::default()
    });

    let adapter = chart_rs::platform_gtk::GtkChartAdapter::new(engine);
    let drawing_area = adapter.drawing_area().clone();
    let engine = adapter.engine();
    let fetch_error = Rc::new(RefCell::new(String::new()));

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

    let limit_spin = gtk::SpinButton::with_range(120.0, 1200.0, 60.0);
    limit_spin.set_value(480.0);

    let reload_button = gtk::Button::with_label("Reload Binance");

    let time_policy_combo = gtk::ComboBoxText::new();
    time_policy_combo.append(Some("logical"), "Time logical(2)");
    time_policy_combo.append(Some("utc"), "Time UTC date/min");
    time_policy_combo.append(Some("utc-sec"), "Time UTC date/sec");
    time_policy_combo.append(Some("adaptive"), "Time UTC adaptive");
    time_policy_combo.set_active_id(Some("adaptive"));

    let timezone_spin = gtk::SpinButton::with_range(-840.0, 840.0, 15.0);
    timezone_spin.set_value(0.0);

    let session_toggle = gtk::CheckButton::with_label("Session 09:30-16:00");

    let price_policy_combo = gtk::ComboBoxText::new();
    price_policy_combo.append(Some("adaptive"), "Price adaptive");
    price_policy_combo.append(Some("fixed2"), "Price fixed(2)");
    price_policy_combo.append(Some("fixed4"), "Price fixed(4)");
    price_policy_combo.append(Some("move01"), "Price min-move 0.1");
    price_policy_combo.append(Some("move001"), "Price min-move 0.01");
    price_policy_combo.set_active_id(Some("adaptive"));

    let price_display_combo = gtk::ComboBoxText::new();
    price_display_combo.append(Some("normal"), "Display normal");
    price_display_combo.append(Some("percent"), "Display percent");
    price_display_combo.append(Some("indexed"), "Display indexed100");
    price_display_combo.set_active_id(Some("normal"));

    let custom_time_toggle = gtk::CheckButton::with_label("Custom time fmt");
    let custom_price_toggle = gtk::CheckButton::with_label("Custom price fmt");
    let magnet_toggle = gtk::CheckButton::with_label("Magnet crosshair");

    let apply_button = gtk::Button::with_label("Apply Axis Config");
    let clear_cache_button = gtk::Button::with_label("Clear Axis Caches");

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.append(&gtk::Label::new(Some("Symbol")));
    controls.append(&symbol_combo);
    controls.append(&gtk::Label::new(Some("Interval")));
    controls.append(&interval_combo);
    controls.append(&gtk::Label::new(Some("Limit")));
    controls.append(&limit_spin);
    controls.append(&reload_button);
    controls.append(&time_policy_combo);
    controls.append(&gtk::Label::new(Some("TZ")));
    controls.append(&timezone_spin);
    controls.append(&session_toggle);
    controls.append(&price_policy_combo);
    controls.append(&price_display_combo);
    controls.append(&custom_time_toggle);
    controls.append(&custom_price_toggle);
    controls.append(&magnet_toggle);
    controls.append(&apply_button);
    controls.append(&clear_cache_button);

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let time_policy_combo = time_policy_combo.clone();
        let timezone_spin = timezone_spin.clone();
        let session_toggle = session_toggle.clone();
        let price_policy_combo = price_policy_combo.clone();
        let price_display_combo = price_display_combo.clone();
        let custom_time_toggle = custom_time_toggle.clone();
        let custom_price_toggle = custom_price_toggle.clone();

        apply_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let time_policy = match time_policy_combo.active_id().as_deref() {
                    Some("logical") => TimeAxisLabelPolicy::LogicalDecimal { precision: 2 },
                    Some("utc") => TimeAxisLabelPolicy::UtcDateTime {
                        show_seconds: false,
                    },
                    Some("utc-sec") => TimeAxisLabelPolicy::UtcDateTime { show_seconds: true },
                    _ => TimeAxisLabelPolicy::UtcAdaptive,
                };
                let timezone = TimeAxisTimeZone::FixedOffsetMinutes {
                    minutes: timezone_spin.value().round() as i16,
                };
                let session = if session_toggle.is_active() {
                    Some(TimeAxisSessionConfig {
                        start_hour: 9,
                        start_minute: 30,
                        end_hour: 16,
                        end_minute: 0,
                    })
                } else {
                    None
                };
                let _ = chart.set_time_axis_label_config(TimeAxisLabelConfig {
                    policy: time_policy,
                    timezone,
                    session,
                    ..TimeAxisLabelConfig::default()
                });

                let price_policy = match price_policy_combo.active_id().as_deref() {
                    Some("fixed2") => PriceAxisLabelPolicy::FixedDecimals { precision: 2 },
                    Some("fixed4") => PriceAxisLabelPolicy::FixedDecimals { precision: 4 },
                    Some("move01") => PriceAxisLabelPolicy::MinMove {
                        min_move: 0.1,
                        trim_trailing_zeros: true,
                    },
                    Some("move001") => PriceAxisLabelPolicy::MinMove {
                        min_move: 0.01,
                        trim_trailing_zeros: true,
                    },
                    _ => PriceAxisLabelPolicy::Adaptive,
                };
                let display_mode = match price_display_combo.active_id().as_deref() {
                    Some("percent") => PriceAxisDisplayMode::Percentage { base_price: None },
                    Some("indexed") => PriceAxisDisplayMode::IndexedTo100 { base_price: None },
                    _ => PriceAxisDisplayMode::Normal,
                };
                let _ = chart.set_price_axis_label_config(PriceAxisLabelConfig {
                    policy: price_policy,
                    display_mode,
                    ..PriceAxisLabelConfig::default()
                });

                if custom_time_toggle.is_active() {
                    chart.set_time_label_formatter(Arc::new(|value| format!("T*{value:.0}")));
                } else {
                    chart.clear_time_label_formatter();
                }
                if custom_price_toggle.is_active() {
                    chart.set_price_label_formatter(Arc::new(|value| format!("P*{value:.4}")));
                } else {
                    chart.clear_price_label_formatter();
                }
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        magnet_toggle.connect_toggled(move |toggle| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                if toggle.is_active() {
                    chart.set_crosshair_mode(CrosshairMode::Magnet);
                } else {
                    chart.set_crosshair_mode(CrosshairMode::Normal);
                }
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        clear_cache_button.connect_clicked(move |_| {
            if let Ok(chart) = engine.try_borrow() {
                chart.clear_time_label_cache();
                chart.clear_price_label_cache();
            }
        });
    }

    {
        let engine = Rc::clone(&engine);
        let fetch_error = Rc::clone(&fetch_error);
        let drawing_area = drawing_area.clone();
        let symbol_combo = symbol_combo.clone();
        let interval_combo = interval_combo.clone();
        let limit_spin = limit_spin.clone();

        reload_button.connect_clicked(move |_| {
            let symbol = symbol_combo
                .active_id()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "BTCUSDT".to_owned());
            let interval = interval_combo
                .active_id()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "1h".to_owned());
            let limit = limit_spin.value().round() as u16;

            match binance::fetch_market_data(&symbol, &interval, limit) {
                Ok(data) => {
                    if let Ok(mut chart) = engine.try_borrow_mut() {
                        chart.set_data(data.close_points);
                        chart.set_candles(data.candles);
                        let _ = chart.fit_time_to_data(TimeScaleTuning::default());
                        let _ = chart.autoscale_price_from_candles();
                    }
                    fetch_error.borrow_mut().clear();
                }
                Err(err) => {
                    *fetch_error.borrow_mut() = format!("fetch error: {err}");
                }
            }
            drawing_area.queue_draw();
        });
    }

    let status_label = gtk::Label::new(None);
    status_label.set_xalign(0.0);
    {
        let engine = Rc::clone(&engine);
        let fetch_error = Rc::clone(&fetch_error);
        let status_label = status_label.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(170), move || {
            if let Ok(chart) = engine.try_borrow() {
                let (time_start, time_end) = chart.time_visible_range();
                let (price_min, price_max) = chart.price_domain();
                let t_cache = chart.time_label_cache_stats();
                let p_cache = chart.price_label_cache_stats();
                let cross = chart.crosshair_mode();
                let mut text = format!(
                    "axis-lab: t=[{time_start:.0},{time_end:.0}] p=[{price_min:.3},{price_max:.3}] crosshair={cross:?} cache_t={}/{} cache_p={}/{}",
                    t_cache.hits, t_cache.misses, p_cache.hits, p_cache.misses,
                );

                let err = fetch_error.borrow();
                if !err.is_empty() {
                    text.push_str(" | ");
                    text.push_str(err.as_str());
                }
                status_label.set_text(&text);
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    let instructions = gtk::Label::new(Some(
        "Axis lab con Binance real: prueba políticas de formateo de tiempo/precio, timezone/session, display mode y cache lifecycle.",
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
        .title("chart-rs Binance Axis Lab")
        .default_width(1320)
        .default_height(880)
        .build();
    window.set_child(Some(&root));
    window.present();
}
