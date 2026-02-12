#[path = "shared/mod.rs"]
mod shared;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{OhlcBar, TimeScaleTuning, Viewport};
use chart_rs::interaction::CrosshairMode;
use chart_rs::render::CairoRenderer;
use gtk4 as gtk;
use gtk4::prelude::*;
use serde_json::Value;
use tungstenite::{Message, connect};

use shared::binance::{self, MarketData};

const DEFAULT_SYMBOL: &str = "BTCUSDT";
const DEFAULT_INTERVAL: &str = "1m";
const DEFAULT_LIMIT: u16 = 800;
const LIVE_DRAIN_MS: u64 = 120;

#[derive(Debug, Clone, Copy)]
struct LiveKline {
    open_time_secs: f64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("rs.chart.examples.gtk_binance_defaults_normal_live")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    let initial_data =
        match binance::fetch_market_data(DEFAULT_SYMBOL, DEFAULT_INTERVAL, DEFAULT_LIMIT) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("binance historical fetch failed, using fallback dataset: {err}");
                binance::fallback_market_data(DEFAULT_SYMBOL, DEFAULT_INTERVAL)
            }
        };

    let engine = match build_engine(&initial_data) {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("failed to initialize defaults-normal-live engine: {err}");
            return;
        }
    };

    let engine = Rc::new(RefCell::new(engine));
    let source_label = Rc::new(RefCell::new(initial_data.source_label.clone()));
    let live_enabled = Rc::new(Cell::new(true));
    let follow_tail = Rc::new(Cell::new(true));
    let last_live_time = Rc::new(Cell::new(None::<f64>));

    let (live_tx, live_rx) = mpsc::channel::<LiveKline>();
    spawn_binance_kline_worker(
        DEFAULT_SYMBOL.to_owned(),
        DEFAULT_INTERVAL.to_owned(),
        live_tx,
    );

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    drawing_area.set_draw_func({
        let engine = Rc::clone(&engine);
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
        }
    });
    shared::attach_default_interactions(&drawing_area, Rc::clone(&engine));

    let live_toggle = gtk::CheckButton::with_label("Realtime WebSocket");
    live_toggle.set_active(true);
    let follow_tail_toggle = gtk::CheckButton::with_label("Follow Tail");
    follow_tail_toggle.set_active(true);
    let reset_button = gtk::Button::with_label("Reload Historical");

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.append(&live_toggle);
    controls.append(&follow_tail_toggle);
    controls.append(&reset_button);

    {
        let live_enabled = Rc::clone(&live_enabled);
        live_toggle.connect_toggled(move |toggle| {
            live_enabled.set(toggle.is_active());
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
        let source_label = Rc::clone(&source_label);
        reset_button.connect_clicked(move |_| {
            match binance::fetch_market_data(DEFAULT_SYMBOL, DEFAULT_INTERVAL, DEFAULT_LIMIT) {
                Ok(data) => {
                    if let Ok(mut chart) = engine.try_borrow_mut() {
                        apply_historical_data(&mut chart, &data);
                    }
                    *source_label.borrow_mut() = data.source_label;
                }
                Err(err) => eprintln!("reload historical fetch error: {err}"),
            }
            drawing_area.queue_draw();
        });
    }

    {
        let engine = Rc::clone(&engine);
        let drawing_area = drawing_area.clone();
        let source_label = Rc::clone(&source_label);
        let live_enabled = Rc::clone(&live_enabled);
        let follow_tail = Rc::clone(&follow_tail);
        let last_live_time = Rc::clone(&last_live_time);
        gtk::glib::timeout_add_local(Duration::from_millis(LIVE_DRAIN_MS), move || {
            while let Ok(live) = live_rx.try_recv() {
                if !live_enabled.get() {
                    continue;
                }

                if let Ok(candle) = OhlcBar::new(
                    live.open_time_secs,
                    live.open,
                    live.high,
                    live.low,
                    live.close,
                ) {
                    if let Ok(mut chart) = engine.try_borrow_mut() {
                        let _ = chart.update_candle(candle);
                        if follow_tail.get() {
                            let _ = chart.scroll_time_to_realtime();
                        }
                    }
                    last_live_time.set(Some(live.open_time_secs));
                    *source_label.borrow_mut() = "binance-websocket".to_owned();
                }
            }

            drawing_area.queue_draw();
            gtk::glib::ControlFlow::Continue
        });
    }

    let status_label = gtk::Label::new(None);
    status_label.set_xalign(0.0);
    {
        let engine = Rc::clone(&engine);
        let status_label = status_label.clone();
        let source_label = Rc::clone(&source_label);
        let live_enabled = Rc::clone(&live_enabled);
        let follow_tail = Rc::clone(&follow_tail);
        let last_live_time = Rc::clone(&last_live_time);
        gtk::glib::timeout_add_local(Duration::from_millis(220), move || {
            if let Ok(chart) = engine.try_borrow() {
                let (time_start, time_end) = chart.time_visible_range();
                let (price_min, price_max) = chart.price_domain();

                let mut status = format!(
                    "{} {} source={} crosshair={:?} live={} follow_tail={} candles={} t=[{time_start:.0},{time_end:.0}] p=[{price_min:.3},{price_max:.3}]",
                    DEFAULT_SYMBOL,
                    DEFAULT_INTERVAL,
                    source_label.borrow().as_str(),
                    chart.crosshair_mode(),
                    live_enabled.get(),
                    follow_tail.get(),
                    chart.candles().len(),
                );
                if let Some(time) = last_live_time.get() {
                    status.push_str(&format!(" last_live_t={time:.0}"));
                }
                status_label.set_text(&status);
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    let instructions = gtk::Label::new(Some(
        "Defaults + Crosshair Normal + Binance historical + realtime por WebSocket (candlestick kline).",
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
        .title("chart-rs Binance Defaults + Normal (WebSocket)")
        .default_width(1360)
        .default_height(860)
        .build();
    window.set_child(Some(&root));
    window.present();
}

fn build_engine(data: &MarketData) -> chart_rs::ChartResult<ChartEngine<CairoRenderer>> {
    let (time_start, time_end) = binance::data_time_range(data);
    let (price_min, price_max) = binance::data_price_range(data);

    let renderer = CairoRenderer::new(1360, 860)?;
    let config = ChartEngineConfig::new(Viewport::new(1360, 860), time_start, time_end)
        .with_price_domain(price_min, price_max)
        .with_crosshair_mode(CrosshairMode::Normal);
    let mut engine = ChartEngine::new(renderer, config)?;
    apply_historical_data(&mut engine, data);
    Ok(engine)
}

fn apply_historical_data(engine: &mut ChartEngine<CairoRenderer>, data: &MarketData) {
    engine.set_candles(data.candles.clone());
    let _ = engine.fit_time_to_data(TimeScaleTuning::default());
    let _ = engine.autoscale_price_from_candles();
}

fn spawn_binance_kline_worker(symbol: String, interval: String, tx: mpsc::Sender<LiveKline>) {
    thread::spawn(move || {
        let stream_name = format!("{}@kline_{}", symbol.to_lowercase(), interval);
        let endpoint = format!("wss://stream.binance.com:9443/ws/{stream_name}");

        loop {
            let (mut socket, _) = match connect(endpoint.as_str()) {
                Ok(conn) => conn,
                Err(err) => {
                    eprintln!("websocket connect error: {err}");
                    thread::sleep(Duration::from_secs(2));
                    continue;
                }
            };

            loop {
                let message = match socket.read() {
                    Ok(msg) => msg,
                    Err(err) => {
                        eprintln!("websocket read error: {err}");
                        break;
                    }
                };

                let Message::Text(payload) = message else {
                    continue;
                };

                if let Some(kline) = parse_live_kline(&payload) {
                    if tx.send(kline).is_err() {
                        return;
                    }
                }
            }

            thread::sleep(Duration::from_secs(1));
        }
    });
}

fn parse_live_kline(payload: &str) -> Option<LiveKline> {
    let json = serde_json::from_str::<Value>(payload).ok()?;
    let kline = json.get("k")?;

    let open_time_ms = kline.get("t")?.as_i64()? as f64;
    let open = parse_number(kline.get("o")?)?;
    let high = parse_number(kline.get("h")?)?;
    let low = parse_number(kline.get("l")?)?;
    let close = parse_number(kline.get("c")?)?;

    Some(LiveKline {
        open_time_secs: open_time_ms / 1_000.0,
        open,
        high,
        low,
        close,
    })
}

fn parse_number(value: &Value) -> Option<f64> {
    value.as_str()?.parse::<f64>().ok()
}
