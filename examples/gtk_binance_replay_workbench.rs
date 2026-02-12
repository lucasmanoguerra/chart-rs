#[path = "shared/mod.rs"]
mod shared;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use chart_rs::api::{
    ChartEngine, ChartEngineConfig, RenderStyle, TimeAxisLabelConfig, TimeAxisLabelPolicy,
};
use chart_rs::core::{DataPoint, OhlcBar, TimeScaleTuning, Viewport};
use chart_rs::render::{CairoRenderer, Color};
use gtk4 as gtk;
use gtk4::prelude::*;

use shared::binance::{self, MarketData};

#[derive(Debug, Clone)]
struct ReplayCursor {
    market: MarketData,
    seed_len: usize,
    cursor: usize,
}

impl ReplayCursor {
    fn from_market_data(market: MarketData) -> Self {
        let max_len = market.close_points.len().min(market.candles.len());
        let seed_len = initial_seed_len(max_len);
        Self {
            market,
            seed_len,
            cursor: seed_len,
        }
    }

    fn total_len(&self) -> usize {
        self.market
            .close_points
            .len()
            .min(self.market.candles.len())
    }

    fn seeded_points(&self) -> Vec<DataPoint> {
        self.market.close_points[..self.seed_len].to_vec()
    }

    fn seeded_candles(&self) -> Vec<OhlcBar> {
        self.market.candles[..self.seed_len].to_vec()
    }

    fn reset(&mut self) {
        self.cursor = self.seed_len;
    }

    fn consumed_len(&self) -> usize {
        self.cursor
    }

    fn next_item(&mut self) -> Option<(DataPoint, OhlcBar)> {
        if self.cursor >= self.total_len() {
            return None;
        }
        let index = self.cursor;
        self.cursor += 1;
        Some((self.market.close_points[index], self.market.candles[index]))
    }
}

fn initial_seed_len(total_len: usize) -> usize {
    if total_len <= 2 {
        return total_len;
    }
    let projected = (total_len as f64 * 0.28).round() as usize;
    projected.clamp(80, total_len - 1)
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.gtk_binance_replay_workbench")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let initial_data = match binance::fetch_market_data("BTCUSDT", "1h", 720) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("binance fetch failed, using fallback dataset: {err}");
            binance::fallback_market_data("BTCUSDT", "1h")
        }
    };

    let replay = Rc::new(RefCell::new(ReplayCursor::from_market_data(initial_data)));

    let renderer = match CairoRenderer::new(1380, 900) {
        Ok(renderer) => renderer,
        Err(err) => {
            eprintln!("renderer init error: {err}");
            return;
        }
    };

    let config = ChartEngineConfig::new(Viewport::new(1380, 900), 0.0, 1.0)
        .with_price_domain(0.0, 1.0)
        .with_crosshair_mode(chart_rs::interaction::CrosshairMode::Normal);
    let mut engine = match ChartEngine::new(renderer, config) {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("engine init error: {err}");
            return;
        }
    };
    engine.set_price_scale_realtime_behavior(chart_rs::api::PriceScaleRealtimeBehavior {
        autoscale_on_data_set: true,
        autoscale_on_data_update: true,
    });

    if let Err(err) = prepare_engine_from_replay(&mut engine, &replay.borrow(), true) {
        eprintln!("failed to initialize replay engine: {err}");
        return;
    }

    let adapter = chart_rs::platform_gtk::GtkChartAdapter::new(engine);
    let drawing_area = adapter.drawing_area().clone();
    let engine = adapter.engine();

    shared::attach_default_interactions(&drawing_area, Rc::clone(&engine));

    let running = Rc::new(Cell::new(true));
    let follow_tail = Rc::new(Cell::new(true));
    let fetch_error = Rc::new(RefCell::new(String::new()));

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

    let limit_spin = gtk::SpinButton::with_range(180.0, 1500.0, 60.0);
    limit_spin.set_value(720.0);

    let bars_per_tick_spin = gtk::SpinButton::with_range(1.0, 12.0, 1.0);
    bars_per_tick_spin.set_value(1.0);

    let running_toggle = gtk::CheckButton::with_label("Replay running");
    running_toggle.set_active(true);

    let follow_tail_toggle = gtk::CheckButton::with_label("Follow tail");
    follow_tail_toggle.set_active(true);

    let reload_button = gtk::Button::with_label("Reload Binance");
    let step_button = gtk::Button::with_label("Step +1");
    let rewind_button = gtk::Button::with_label("Rewind seed");
    let fit_button = gtk::Button::with_label("Fit + autoscale");

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.append(&gtk::Label::new(Some("Symbol")));
    controls.append(&symbol_combo);
    controls.append(&gtk::Label::new(Some("Interval")));
    controls.append(&interval_combo);
    controls.append(&gtk::Label::new(Some("Limit")));
    controls.append(&limit_spin);
    controls.append(&reload_button);
    controls.append(&running_toggle);
    controls.append(&follow_tail_toggle);
    controls.append(&gtk::Label::new(Some("Bars/tick")));
    controls.append(&bars_per_tick_spin);
    controls.append(&step_button);
    controls.append(&rewind_button);
    controls.append(&fit_button);

    {
        let running = Rc::clone(&running);
        running_toggle.connect_toggled(move |toggle| {
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
        let replay = Rc::clone(&replay);
        let drawing_area = drawing_area.clone();
        let follow_tail = Rc::clone(&follow_tail);
        step_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                step_replay_once(&mut chart, &mut replay.borrow_mut(), follow_tail.get());
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let replay = Rc::clone(&replay);
        let drawing_area = drawing_area.clone();
        let follow_tail = Rc::clone(&follow_tail);
        rewind_button.connect_clicked(move |_| {
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let mut replay = replay.borrow_mut();
                replay.reset();
                let _ = prepare_engine_from_replay(&mut chart, &replay, follow_tail.get());
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
        let replay = Rc::clone(&replay);
        let fetch_error = Rc::clone(&fetch_error);
        let drawing_area = drawing_area.clone();
        let follow_tail = Rc::clone(&follow_tail);
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
                        let replay_state = ReplayCursor::from_market_data(data);
                        let _ = prepare_engine_from_replay(
                            &mut chart,
                            &replay_state,
                            follow_tail.get(),
                        );
                        *replay.borrow_mut() = replay_state;
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

    {
        let engine = Rc::clone(&engine);
        let replay = Rc::clone(&replay);
        let drawing_area = drawing_area.clone();
        let running = Rc::clone(&running);
        let follow_tail = Rc::clone(&follow_tail);
        let bars_per_tick_spin = bars_per_tick_spin.clone();

        gtk::glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
            if !running.get() {
                return gtk::glib::ControlFlow::Continue;
            }

            let bars_per_tick = bars_per_tick_spin.value().round().clamp(1.0, 12.0) as usize;
            if let Ok(mut chart) = engine.try_borrow_mut() {
                let mut replay = replay.borrow_mut();
                let mut appended = 0usize;

                for _ in 0..bars_per_tick {
                    if step_replay_once(&mut chart, &mut replay, follow_tail.get()) {
                        appended += 1;
                    } else {
                        running.set(false);
                        break;
                    }
                }

                if appended > 0 {
                    drawing_area.queue_draw();
                }
            }

            gtk::glib::ControlFlow::Continue
        });
    }

    let status_label = gtk::Label::new(None);
    status_label.set_xalign(0.0);
    {
        let engine = Rc::clone(&engine);
        let replay = Rc::clone(&replay);
        let running = Rc::clone(&running);
        let follow_tail = Rc::clone(&follow_tail);
        let fetch_error = Rc::clone(&fetch_error);
        let status_label = status_label.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(170), move || {
            if let Ok(chart) = engine.try_borrow() {
                let replay = replay.borrow();
                let visible_points = chart.visible_points().len();
                let visible_candles = chart.visible_candles().len();
                let (time_start, time_end) = chart.time_visible_range();
                let (price_min, price_max) = chart.price_domain();

                let metadata = chart.series_metadata();
                let symbol = metadata.get("symbol").map(String::as_str).unwrap_or("?");
                let interval = metadata.get("interval").map(String::as_str).unwrap_or("?");
                let source = metadata.get("source").map(String::as_str).unwrap_or("?");

                let mut text = format!(
                    "replay: run={} follow_tail={} {symbol} {interval} source={source} progress={}/{} visible(points={}, candles={}) t=[{time_start:.0},{time_end:.0}] p=[{price_min:.2},{price_max:.2}] crosshair={:?}",
                    running.get(),
                    follow_tail.get(),
                    replay.consumed_len(),
                    replay.total_len(),
                    visible_points,
                    visible_candles,
                    chart.crosshair_mode(),
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
        "Replay workbench con Binance real: arranca con seed parcial y reinyecta barras reales usando append_point/append_candle.",
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
        .title("chart-rs Binance Replay Workbench")
        .default_width(1400)
        .default_height(920)
        .build();
    window.set_child(Some(&root));
    window.present();
}

fn prepare_engine_from_replay(
    chart: &mut ChartEngine<CairoRenderer>,
    replay: &ReplayCursor,
    follow_tail: bool,
) -> chart_rs::ChartResult<()> {
    chart.set_data(replay.seeded_points());
    chart.set_candles(replay.seeded_candles());

    chart.set_series_metadata("symbol", replay.market.symbol.clone());
    chart.set_series_metadata("interval", replay.market.interval.clone());
    chart.set_series_metadata("source", replay.market.source_label.clone());

    chart.fit_time_to_data(TimeScaleTuning::default())?;
    chart.autoscale_price_from_candles()?;

    chart.set_time_axis_label_config(TimeAxisLabelConfig {
        policy: TimeAxisLabelPolicy::UtcAdaptive,
        ..TimeAxisLabelConfig::default()
    })?;

    let style = RenderStyle {
        show_crosshair_time_label_box: true,
        show_crosshair_price_label_box: true,
        crosshair_time_label_box_color: Some(Color::rgba(0.11, 0.24, 0.55, 0.94)),
        crosshair_price_label_box_color: Some(Color::rgba(0.52, 0.18, 0.10, 0.94)),
        ..chart.render_style()
    };
    chart.set_render_style(style)?;

    if follow_tail {
        apply_tail_window(chart, replay.market.interval.as_str());
    }

    Ok(())
}

fn step_replay_once(
    chart: &mut ChartEngine<CairoRenderer>,
    replay: &mut ReplayCursor,
    follow_tail: bool,
) -> bool {
    let Some((point, candle)) = replay.next_item() else {
        return false;
    };

    chart.append_point(point);
    chart.append_candle(candle);
    let _ = chart.fit_time_to_data(TimeScaleTuning::default());
    let _ = chart.autoscale_price_from_candles();

    if follow_tail {
        apply_tail_window(chart, replay.market.interval.as_str());
    }

    true
}

fn apply_tail_window(chart: &mut ChartEngine<CairoRenderer>, interval: &str) {
    let (_, end) = chart.time_full_range();
    let window = binance::default_window_secs(interval);
    let start = end - window.max(120.0);
    let _ = chart.set_time_visible_range(start, end);
}
