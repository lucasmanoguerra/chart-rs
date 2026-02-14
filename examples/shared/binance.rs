#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, OhlcBar, TimeScaleTuning, Viewport};
use chart_rs::platform_gtk::GtkChartAdapter;
use chart_rs::render::CairoRenderer;
use gtk4 as gtk;
use gtk4::prelude::*;

#[derive(Debug, Clone)]
pub struct BinanceKline {
    pub open_time_sec: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub fn fetch_binance_klines(
    symbol: &str,
    interval: &str,
    limit: usize,
) -> Result<Vec<BinanceKline>, String> {
    let url = format!(
        "https://api.binance.com/api/v3/klines?symbol={symbol}&interval={interval}&limit={limit}"
    );
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|e| format!("reqwest client error: {e}"))?;
    let raw: Vec<Vec<serde_json::Value>> = client
        .get(url)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("binance request error: {e}"))?
        .json()
        .map_err(|e| format!("binance decode error: {e}"))?;

    let mut out = Vec::with_capacity(raw.len());
    for row in raw {
        if row.len() < 6 {
            continue;
        }
        let Some(open_time_ms) = row[0]
            .as_f64()
            .or_else(|| row[0].as_i64().map(|v| v as f64))
        else {
            continue;
        };
        let open = parse_binance_num(&row[1])?;
        let high = parse_binance_num(&row[2])?;
        let low = parse_binance_num(&row[3])?;
        let close = parse_binance_num(&row[4])?;
        let volume = parse_binance_num(&row[5])?;
        out.push(BinanceKline {
            open_time_sec: open_time_ms / 1000.0,
            open,
            high,
            low,
            close,
            volume,
        });
    }
    Ok(out)
}

fn parse_binance_num(value: &serde_json::Value) -> Result<f64, String> {
    if let Some(v) = value.as_f64() {
        return Ok(v);
    }
    if let Some(v) = value.as_str() {
        return v
            .parse::<f64>()
            .map_err(|e| format!("invalid numeric value `{v}`: {e}"));
    }
    Err(format!("invalid numeric json value: {value}"))
}

pub fn klines_to_ohlc(klines: &[BinanceKline]) -> Result<Vec<OhlcBar>, String> {
    let mut out = Vec::with_capacity(klines.len());
    for k in klines {
        out.push(
            OhlcBar::new(k.open_time_sec, k.open, k.high, k.low, k.close)
                .map_err(|e| format!("invalid ohlc from kline: {e}"))?,
        );
    }
    Ok(out)
}

pub fn klines_to_volume_points(klines: &[BinanceKline]) -> Vec<DataPoint> {
    klines
        .iter()
        .map(|k| DataPoint::new(k.open_time_sec, k.volume))
        .collect()
}

pub fn build_engine_with_binance_candles(
    symbol: &str,
    interval: &str,
    limit: usize,
    width: u32,
    height: u32,
) -> Result<ChartEngine<CairoRenderer>, String> {
    let klines = fetch_binance_klines(symbol, interval, limit)?;
    if klines.is_empty() {
        return Err("binance returned empty klines".to_owned());
    }

    let candles = klines_to_ohlc(&klines)?;
    let min_t = klines
        .first()
        .map(|k| k.open_time_sec)
        .ok_or_else(|| "missing first kline".to_owned())?;
    let max_t = klines
        .last()
        .map(|k| k.open_time_sec)
        .ok_or_else(|| "missing last kline".to_owned())?;

    let renderer = CairoRenderer::new(width as i32, height as i32)
        .map_err(|e| format!("cairo renderer init error: {e}"))?;
    let config = ChartEngineConfig::new(Viewport::new(width, height), min_t, max_t)
        .with_price_domain(0.0, 1.0);
    let mut engine = ChartEngine::new(renderer, config).map_err(|e| format!("engine init: {e}"))?;
    engine.set_candles(candles);
    engine
        .autoscale_price_from_candles()
        .map_err(|e| format!("autoscale candles: {e}"))?;
    engine
        .fit_time_to_data(TimeScaleTuning::default())
        .map_err(|e| format!("fit time: {e}"))?;
    Ok(engine)
}

pub fn install_default_interaction(adapter: Rc<GtkChartAdapter<CairoRenderer>>) {
    let area = adapter.drawing_area().clone();

    let motion = gtk::EventControllerMotion::new();
    {
        let adapter = Rc::clone(&adapter);
        motion.connect_motion(move |_, x, y| {
            let _ = adapter.update_engine(|engine| {
                engine.pointer_move(x, y);
                Ok(())
            });
        });
    }
    {
        let adapter = Rc::clone(&adapter);
        motion.connect_leave(move |_| {
            let _ = adapter.update_engine(|engine| {
                engine.pointer_leave();
                Ok(())
            });
        });
    }
    area.add_controller(motion);

    #[derive(Clone, Copy, Debug)]
    enum DragRegion {
        Plot,
        PriceAxis,
        TimeAxis,
    }

    #[derive(Clone, Copy, Debug)]
    struct DragState {
        region: DragRegion,
        last_dx: f64,
        last_dy: f64,
    }

    let drag_state = Rc::new(RefCell::new(DragState {
        region: DragRegion::Plot,
        last_dx: 0.0,
        last_dy: 0.0,
    }));

    let drag = gtk::GestureDrag::new();
    {
        let adapter = Rc::clone(&adapter);
        let drag_state = Rc::clone(&drag_state);
        drag.connect_drag_begin(move |_, start_x, start_y| {
            let region = {
                let engine_rc = adapter.engine();
                let engine = engine_rc.borrow();
                let viewport = engine.viewport();
                let style = engine.render_style();
                let plot_right = viewport.width as f64 - style.price_axis_width_px;
                let plot_bottom = viewport.height as f64 - style.time_axis_height_px;
                if start_x >= plot_right {
                    DragRegion::PriceAxis
                } else if start_y >= plot_bottom {
                    DragRegion::TimeAxis
                } else {
                    DragRegion::Plot
                }
            };
            *drag_state.borrow_mut() = DragState {
                region,
                last_dx: 0.0,
                last_dy: 0.0,
            };
            let _ = adapter.update_engine(|engine| {
                engine.pan_start();
                Ok(())
            });
        });
    }
    {
        let adapter = Rc::clone(&adapter);
        let drag_state = Rc::clone(&drag_state);
        drag.connect_drag_update(move |_, offset_x, offset_y| {
            let mut state = drag_state.borrow_mut();
            let delta_x = offset_x - state.last_dx;
            let delta_y = offset_y - state.last_dy;
            state.last_dx = offset_x;
            state.last_dy = offset_y;

            let _ = adapter.update_engine(|engine| match state.region {
                DragRegion::Plot => engine.pan_time_visible_by_pixels(delta_x),
                DragRegion::PriceAxis => {
                    let anchor_y = engine.viewport().height as f64 * 0.5;
                    let _ = engine.axis_drag_scale_price(delta_y, anchor_y, 0.16, 0.000_000_1)?;
                    Ok(())
                }
                DragRegion::TimeAxis => {
                    let anchor_x = engine.viewport().width as f64 * 0.5;
                    let _ = engine.axis_drag_scale_time(delta_x, anchor_x, 0.16, 1.0)?;
                    Ok(())
                }
            });
        });
    }
    {
        let adapter = Rc::clone(&adapter);
        drag.connect_drag_end(move |_, _, _| {
            let _ = adapter.update_engine(|engine| {
                engine.pan_end();
                Ok(())
            });
        });
    }
    area.add_controller(drag);

    let scroll = gtk::EventControllerScroll::new(
        gtk::EventControllerScrollFlags::VERTICAL | gtk::EventControllerScrollFlags::HORIZONTAL,
    );
    {
        let adapter = Rc::clone(&adapter);
        scroll.connect_scroll(move |_, dx, dy| {
            let _ = adapter.update_engine(|engine| {
                if dy != 0.0 {
                    let wheel_delta = dy * 120.0;
                    let anchor_px = engine.viewport().width as f64 * 0.5;
                    let _ = engine.wheel_zoom_time_visible(wheel_delta, anchor_px, 0.12, 1.0)?;
                }
                if dx != 0.0 {
                    let wheel_delta = dx * 120.0;
                    let _ = engine.wheel_pan_time_visible(wheel_delta, 0.16)?;
                }
                Ok(())
            });
            gtk::glib::Propagation::Stop
        });
    }
    area.add_controller(scroll);
}
