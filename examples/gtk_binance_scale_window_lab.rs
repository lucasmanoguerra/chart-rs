#[path = "shared/mod.rs"]
mod shared;

use std::cell::RefCell;
use std::rc::Rc;

use chart_rs::api::{
    ChartEngine, ChartEngineConfig, RenderStyle, TimeAxisLabelConfig, TimeAxisLabelPolicy,
};
use chart_rs::core::{TimeScaleTuning, Viewport};
use chart_rs::render::{CairoRenderer, Color};
use gtk4 as gtk;
use gtk4::prelude::*;

use shared::binance;

fn main() {
    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.gtk_binance_scale_window_lab")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let data = match binance::fetch_market_data("BTCUSDT", "1h", 520) {
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
            eprintln!("renderer init error: {err}");
            return;
        }
    };

    let config = ChartEngineConfig::new(Viewport::new(1320, 860), time_start, time_end)
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
    engine.set_series_metadata("symbol", data.symbol.clone());
    engine.set_series_metadata("interval", data.interval.clone());
    engine.set_series_metadata("source", data.source_label.clone());

    let _ = engine.set_time_axis_label_config(TimeAxisLabelConfig {
        policy: TimeAxisLabelPolicy::UtcAdaptive,
        ..TimeAxisLabelConfig::default()
    });

    let style = RenderStyle {
        major_grid_line_color: Color::rgba(0.31, 0.44, 0.70, 0.75),
        crosshair_line_color: Color::rgba(0.93, 0.70, 0.40, 0.55),
        ..engine.render_style()
    };
    let _ = engine.set_render_style(style);

    let adapter = chart_rs::platform_gtk::GtkChartAdapter::new(engine);
    let drawing_area = adapter.drawing_area().clone();
    let engine = adapter.engine();
    let fetch_error = Rc::new(RefCell::new(String::new()));
    let probe_note = Rc::new(RefCell::new(String::new()));

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

    let limit_spin = gtk::SpinButton::with_range(180.0, 1400.0, 40.0);
    limit_spin.set_value(520.0);

    let reload_button = gtk::Button::with_label("Reload Binance");
    let fit_button = gtk::Button::with_label("Fit");
    let reset_button = gtk::Button::with_label("Reset");
    let pan_left_button = gtk::Button::with_label("Pan <-");
    let pan_right_button = gtk::Button::with_label("Pan ->");
    let zoom_in_button = gtk::Button::with_label("Zoom +");
    let zoom_out_button = gtk::Button::with_label("Zoom -");
    let probe_button = gtk::Button::with_label("Probe round-trip");

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.append(&gtk::Label::new(Some("Symbol")));
    controls.append(&symbol_combo);
    controls.append(&gtk::Label::new(Some("Interval")));
    controls.append(&interval_combo);
    controls.append(&gtk::Label::new(Some("Limit")));
    controls.append(&limit_spin);
    controls.append(&reload_button);
    controls.append(&fit_button);
    controls.append(&reset_button);
    controls.append(&pan_left_button);
    controls.append(&pan_right_button);
    controls.append(&zoom_in_button);
    controls.append(&zoom_out_button);
    controls.append(&probe_button);

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
        let drawing_area = drawing_area.clone();
        pan_left_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let _ = chart.pan_time_visible_by_pixels(180.0);
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        pan_right_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let _ = chart.pan_time_visible_by_pixels(-180.0);
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        zoom_in_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let anchor = chart.viewport().width as f64 * 0.5;
                let _ = chart.zoom_time_visible_around_pixel(1.35, anchor, 0.5);
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        zoom_out_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let anchor = chart.viewport().width as f64 * 0.5;
                let _ = chart.zoom_time_visible_around_pixel(0.78, anchor, 0.5);
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let probe_note = Rc::clone(&probe_note);
        probe_button.connect_clicked(move |_| {
            if let Ok(chart) = engine.try_borrow() {
                *probe_note.borrow_mut() = build_probe_note(&chart);
            }
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let fetch_error = Rc::clone(&fetch_error);
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
                        chart.set_series_metadata("symbol", data.symbol.clone());
                        chart.set_series_metadata("interval", data.interval.clone());
                        chart.set_series_metadata("source", data.source_label.clone());
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
        let probe_note = Rc::clone(&probe_note);
        let status_label = status_label.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(180), move || {
            if let Ok(chart) = engine.try_borrow() {
                let (visible_start, visible_end) = chart.time_visible_range();
                let (full_start, full_end) = chart.time_full_range();
                let (price_min, price_max) = chart.price_domain();

                let visible_points = chart.visible_points().len();
                let visible_candles = chart.visible_candles().len();
                let visible_points_overscan = chart
                    .visible_points_with_overscan(0.12)
                    .map_or(0, |points| points.len());
                let visible_candles_overscan = chart
                    .visible_candles_with_overscan(0.12)
                    .map_or(0, |candles| candles.len());

                let t_error = time_round_trip_error(&chart);
                let p_error = price_round_trip_error(&chart);

                let meta = chart.series_metadata();
                let symbol = meta.get("symbol").map(String::as_str).unwrap_or("?");
                let interval = meta.get("interval").map(String::as_str).unwrap_or("?");

                let mut text = format!(
                    "scale-lab: {symbol} {interval} vis_t=[{visible_start:.0},{visible_end:.0}] full_t=[{full_start:.0},{full_end:.0}] p=[{price_min:.3},{price_max:.3}] visible(p/c)={visible_points}/{visible_candles} overscan(p/c)={visible_points_overscan}/{visible_candles_overscan} roundtrip(t={t_error:.6}, p={p_error:.6})"
                );

                let note = probe_note.borrow();
                if !note.is_empty() {
                    text.push_str(" | ");
                    text.push_str(note.as_str());
                }

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
        "Scale/window lab con Binance real: valida pan/zoom/rango visible, overscan y conversiones pixel<->time/price.",
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
        .title("chart-rs Binance Scale Window Lab")
        .default_width(1320)
        .default_height(900)
        .build();
    window.set_child(Some(&root));
    window.present();
}

fn time_round_trip_error(chart: &ChartEngine<CairoRenderer>) -> f64 {
    let (start, end) = chart.time_visible_range();
    let sample = (start + end) * 0.5;
    chart
        .map_x_to_pixel(sample)
        .and_then(|pixel| chart.map_pixel_to_x(pixel))
        .map_or(f64::NAN, |mapped| (mapped - sample).abs())
}

fn price_round_trip_error(chart: &ChartEngine<CairoRenderer>) -> f64 {
    let (min, max) = chart.price_domain();
    let sample = (min + max) * 0.5;
    chart
        .map_price_to_pixel(sample)
        .and_then(|pixel| chart.map_pixel_to_price(pixel))
        .map_or(f64::NAN, |mapped| (mapped - sample).abs())
}

fn build_probe_note(chart: &ChartEngine<CairoRenderer>) -> String {
    let width = chart.viewport().width as f64;
    let height = chart.viewport().height as f64;

    let center_x = width * 0.5;
    let center_y = height * 0.5;

    let mapped_time = chart.map_pixel_to_x(center_x).ok();
    let mapped_price = chart.map_pixel_to_price(center_y).ok();

    format!(
        "probe(center_px={center_x:.1},{center_y:.1} -> t={:?}, p={:?})",
        mapped_time.map(|value| format!("{value:.2}")),
        mapped_price.map(|value| format!("{value:.4}")),
    )
}
