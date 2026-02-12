#[path = "shared/mod.rs"]
mod shared;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use chart_rs::api::{
    ChartEngine, ChartEngineConfig, CrosshairLabelSourceMode, CrosshairTimeLabelFormatterContext,
    PriceAxisLabelConfig, PriceAxisLabelPolicy, TimeAxisLabelConfig, TimeAxisLabelPolicy,
};
use chart_rs::core::{TimeScaleTuning, Viewport};
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::CairoRenderer;
use gtk4 as gtk;
use gtk4::prelude::*;

use shared::binance;

#[derive(Debug, Clone, Copy)]
enum FormatterMode {
    None,
    Legacy,
    Context,
}

impl FormatterMode {
    fn from_id(id: &str) -> Self {
        match id {
            "legacy" => Self::Legacy,
            "context" => Self::Context,
            _ => Self::None,
        }
    }
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.gtk_binance_contracts_lab")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let data = match binance::fetch_market_data("BTCUSDT", "1h", 360) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("binance fetch failed, using fallback dataset: {err}");
            binance::fallback_market_data("BTCUSDT", "1h")
        }
    };

    let (time_start, time_end) = binance::data_time_range(&data);
    let (price_min, price_max) = binance::data_price_range(&data);

    let renderer = match CairoRenderer::new(1320, 900) {
        Ok(renderer) => renderer,
        Err(err) => {
            eprintln!("renderer error: {err}");
            return;
        }
    };

    let config = ChartEngineConfig::new(Viewport::new(1320, 900), time_start, time_end)
        .with_price_domain(price_min, price_max)
        .with_crosshair_mode(chart_rs::interaction::CrosshairMode::Normal);
    let mut engine = match ChartEngine::new(renderer, config) {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("engine init error: {err}");
            return;
        }
    };

    engine.set_data(data.close_points);
    engine.set_candles(data.candles);
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
    let engine = adapter.engine();
    let drawing_area = adapter.drawing_area().clone();

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
    limit_spin.set_value(360.0);

    let reload_button = gtk::Button::with_label("Reload Binance");

    let time_mode_combo = gtk::ComboBoxText::new();
    time_mode_combo.append(Some("none"), "Time fmt: none");
    time_mode_combo.append(Some("legacy"), "Time fmt: legacy");
    time_mode_combo.append(Some("context"), "Time fmt: context");
    time_mode_combo.set_active_id(Some("none"));

    let price_mode_combo = gtk::ComboBoxText::new();
    price_mode_combo.append(Some("none"), "Price fmt: none");
    price_mode_combo.append(Some("legacy"), "Price fmt: legacy");
    price_mode_combo.append(Some("context"), "Price fmt: context");
    price_mode_combo.set_active_id(Some("none"));

    let apply_modes_button = gtk::Button::with_label("Apply Formatter Modes");
    let export_contracts_button = gtk::Button::with_label("Export Snapshot/Diagnostics JSON");
    let clear_cache_button = gtk::Button::with_label("Clear Crosshair Formatter Caches");

    let magnet_toggle = gtk::CheckButton::with_label("Magnet crosshair");
    magnet_toggle.set_active(false);

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.append(&gtk::Label::new(Some("Symbol")));
    controls.append(&symbol_combo);
    controls.append(&gtk::Label::new(Some("Interval")));
    controls.append(&interval_combo);
    controls.append(&gtk::Label::new(Some("Limit")));
    controls.append(&limit_spin);
    controls.append(&reload_button);
    controls.append(&time_mode_combo);
    controls.append(&price_mode_combo);
    controls.append(&apply_modes_button);
    controls.append(&export_contracts_button);
    controls.append(&clear_cache_button);
    controls.append(&magnet_toggle);

    let text_view = gtk::TextView::new();
    text_view.set_editable(false);
    text_view.set_monospace(true);
    let text_buffer = gtk::TextBuffer::new(None);
    text_view.set_buffer(Some(&text_buffer));

    let scrolled = gtk::ScrolledWindow::builder()
        .min_content_height(220)
        .vexpand(false)
        .child(&text_view)
        .build();

    let latest_snapshot_bytes = Rc::new(RefCell::new(0usize));
    adapter.set_snapshot_json_hook(7.0, {
        let latest_snapshot_bytes = Rc::clone(&latest_snapshot_bytes);
        move |json| {
            *latest_snapshot_bytes.borrow_mut() = json.len();
        }
    });

    let latest_diag = Rc::new(RefCell::new(String::new()));
    adapter.set_crosshair_diagnostics_hook({
        let latest_diag = Rc::clone(&latest_diag);
        move |diag| {
            *latest_diag.borrow_mut() = format!(
                "diag: mode=({:?}/{:?}) gen=({}/{}) cache=({}/{})",
                diag.time_override_mode,
                diag.price_override_mode,
                diag.time_formatter_generation,
                diag.price_formatter_generation,
                diag.time_cache.size,
                diag.price_cache.size,
            );
        }
    });

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let time_mode_combo = time_mode_combo.clone();
        let price_mode_combo = price_mode_combo.clone();
        apply_modes_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let time_mode = FormatterMode::from_id(
                    time_mode_combo.active_id().as_deref().unwrap_or("none"),
                );
                let price_mode = FormatterMode::from_id(
                    price_mode_combo.active_id().as_deref().unwrap_or("none"),
                );
                apply_formatter_modes(&mut chart, time_mode, price_mode);
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
        let text_buffer = text_buffer.clone();
        export_contracts_button.connect_clicked(move |_| {
            if let Ok(chart) = engine.try_borrow() {
                let snapshot = chart
                    .snapshot_json_contract_v1_pretty(7.0)
                    .unwrap_or_else(|err| format!("snapshot error: {err}"));
                let diagnostics = chart
                    .crosshair_formatter_diagnostics_json_contract_v1_pretty()
                    .unwrap_or_else(|err| format!("diagnostics error: {err}"));

                let preview = format!(
                    "=== snapshot_json_contract_v1 ===\n{}\n\n=== diagnostics_json_contract_v1 ===\n{}",
                    snapshot, diagnostics
                );
                text_buffer.set_text(&preview);
            }
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
        let drawing_area = drawing_area.clone();
        let text_buffer = text_buffer.clone();
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

            let data = match binance::fetch_market_data(&symbol, &interval, limit) {
                Ok(data) => data,
                Err(err) => {
                    text_buffer.set_text(&format!("reload error: {err}"));
                    binance::fallback_market_data(&symbol, &interval)
                }
            };

            if let Ok(mut chart) = engine.try_borrow_mut() {
                chart.set_data(data.close_points);
                chart.set_candles(data.candles);
                let _ = chart.fit_time_to_data(TimeScaleTuning::default());
                let _ = chart.autoscale_price_from_candles();
                apply_formatter_modes(&mut chart, FormatterMode::None, FormatterMode::None);
            }
            drawing_area.queue_draw();
        });
    }

    let status_label = gtk::Label::new(None);
    status_label.set_xalign(0.0);
    {
        let engine = Rc::clone(&engine);
        let latest_diag = Rc::clone(&latest_diag);
        let latest_snapshot_bytes = Rc::clone(&latest_snapshot_bytes);
        let status_label = status_label.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(170), move || {
            if let Ok(chart) = engine.try_borrow() {
                let (t_gen, p_gen) = chart.crosshair_label_formatter_generations();
                let t_mode = chart.crosshair_time_label_formatter_override_mode();
                let p_mode = chart.crosshair_price_label_formatter_override_mode();
                let (time_start, time_end) = chart.time_visible_range();
                let (price_min, price_max) = chart.price_domain();
                status_label.set_text(&format!(
                    "contracts-lab: t=[{time_start:.0},{time_end:.0}] p=[{price_min:.2},{price_max:.2}] mode=({t_mode:?}/{p_mode:?}) gen=({t_gen}/{p_gen}) snapshot_bytes={} {}",
                    *latest_snapshot_bytes.borrow(),
                    latest_diag.borrow(),
                ));
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    let instructions = gtk::Label::new(Some(
        "Contracts lab con Binance real: prueba lifecycle de formatters (none/legacy/context) y export de contratos JSON v1.",
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
    root.append(&scrolled);
    root.append(adapter.drawing_area());

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("chart-rs Binance Contracts Lab")
        .default_width(1320)
        .default_height(920)
        .build();
    window.set_child(Some(&root));
    window.present();
}

fn apply_formatter_modes(
    chart: &mut ChartEngine<CairoRenderer>,
    time_mode: FormatterMode,
    price_mode: FormatterMode,
) {
    match time_mode {
        FormatterMode::None => {
            chart.clear_crosshair_time_label_formatter();
            chart.clear_crosshair_time_label_formatter_with_context();
        }
        FormatterMode::Legacy => {
            chart.clear_crosshair_time_label_formatter_with_context();
            chart.set_crosshair_time_label_formatter(Arc::new(|value| format!("T-L:{value:.0}")));
        }
        FormatterMode::Context => {
            chart.clear_crosshair_time_label_formatter();
            chart.set_crosshair_time_label_formatter_with_context(Arc::new(
                |value, context: CrosshairTimeLabelFormatterContext| {
                    let src = match context.source_mode {
                        CrosshairLabelSourceMode::SnappedData => "snap",
                        CrosshairLabelSourceMode::PointerProjected => "ptr",
                    };
                    format!("T-C:{value:.0}|{src}|span={:.0}", context.visible_span_abs)
                },
            ));
        }
    }

    match price_mode {
        FormatterMode::None => {
            chart.clear_crosshair_price_label_formatter();
            chart.clear_crosshair_price_label_formatter_with_context();
        }
        FormatterMode::Legacy => {
            chart.clear_crosshair_price_label_formatter_with_context();
            chart.set_crosshair_price_label_formatter(Arc::new(|value| format!("P-L:{value:.3}")));
        }
        FormatterMode::Context => {
            chart.clear_crosshair_price_label_formatter();
            chart.set_crosshair_price_label_formatter_with_context(Arc::new(|value, context| {
                let src = match context.source_mode {
                    CrosshairLabelSourceMode::SnappedData => "snap",
                    CrosshairLabelSourceMode::PointerProjected => "ptr",
                };
                format!("P-C:{value:.3}|{src}|span={:.0}", context.visible_span_abs)
            }));
        }
    }
}
