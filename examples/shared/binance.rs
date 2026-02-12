use std::time::Duration;

use chart_rs::core::{DataPoint, OhlcBar};
use serde_json::Value;

use crate::shared;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MarketData {
    pub symbol: String,
    pub interval: String,
    pub candles: Vec<OhlcBar>,
    pub close_points: Vec<DataPoint>,
    pub volumes: Vec<DataPoint>,
    pub volume_up: Vec<bool>,
    pub source_label: String,
}

#[allow(dead_code)]
pub fn fetch_market_data(symbol: &str, interval: &str, limit: u16) -> Result<MarketData, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|err| format!("http client error: {err}"))?;

    let response = client
        .get("https://api.binance.com/api/v3/uiKlines")
        .query(&[
            ("symbol", symbol),
            ("interval", interval),
            ("limit", &limit.to_string()),
        ])
        .send()
        .map_err(|err| format!("request error: {err}"))?
        .error_for_status()
        .map_err(|err| format!("response error: {err}"))?;

    let rows = response
        .json::<Vec<Vec<Value>>>()
        .map_err(|err| format!("json parse error: {err}"))?;

    build_market_data_from_rows(symbol, interval, rows, "binance-uiKlines")
}

#[allow(dead_code)]
pub fn fallback_market_data(symbol: &str, interval: &str) -> MarketData {
    let (start_time, step, base) = fallback_profile(interval);
    let close_points = shared::build_wave_points(540, start_time, step, base);
    let candles = shared::build_candles_from_points(&close_points).unwrap_or_default();

    let mut volumes = Vec::with_capacity(close_points.len());
    let mut volume_up = Vec::with_capacity(close_points.len());

    for (index, point) in close_points.iter().copied().enumerate() {
        let volume = 120.0
            + ((index as f64) / 3.5).sin().abs() * 90.0
            + ((index as f64) / 17.0).cos().abs() * 55.0;
        volumes.push(DataPoint::new(point.x, volume));
        let prev = close_points
            .get(index.saturating_sub(1))
            .map(|value| value.y)
            .unwrap_or(point.y);
        volume_up.push(point.y >= prev);
    }

    MarketData {
        symbol: symbol.to_owned(),
        interval: interval.to_owned(),
        candles,
        close_points,
        volumes,
        volume_up,
        source_label: "fallback".to_owned(),
    }
}

#[allow(dead_code)]
pub fn default_window_secs(interval: &str) -> f64 {
    match interval {
        "1m" => 6.0 * 3_600.0,
        "5m" => 24.0 * 3_600.0,
        "15m" => 3.0 * 86_400.0,
        "1h" => 7.0 * 86_400.0,
        "4h" => 30.0 * 86_400.0,
        "1d" => 365.0 * 86_400.0,
        _ => 7.0 * 86_400.0,
    }
}

#[allow(dead_code)]
pub fn data_time_range(data: &MarketData) -> (f64, f64) {
    if let (Some(first), Some(last)) = (data.close_points.first(), data.close_points.last()) {
        let span = (last.x - first.x).abs().max(1.0);
        return (first.x - span * 0.02, last.x + span * 0.02);
    }
    (0.0, 1.0)
}

#[allow(dead_code)]
pub fn data_price_range(data: &MarketData) -> (f64, f64) {
    if data.candles.is_empty() {
        return (0.0, 1.0);
    }

    let mut min_price = f64::INFINITY;
    let mut max_price = f64::NEG_INFINITY;
    for candle in &data.candles {
        min_price = min_price.min(candle.low);
        max_price = max_price.max(candle.high);
    }

    let span = (max_price - min_price).max(1e-6);
    let padded_min = (min_price - span * 0.08).max(0.0001);
    let padded_max = max_price + span * 0.08;
    (padded_min, padded_max)
}

fn build_market_data_from_rows(
    symbol: &str,
    interval: &str,
    rows: Vec<Vec<Value>>,
    source_label: &str,
) -> Result<MarketData, String> {
    if rows.is_empty() {
        return Err("empty kline response".to_owned());
    }

    let mut candles = Vec::with_capacity(rows.len());
    let mut close_points = Vec::with_capacity(rows.len());
    let mut volumes = Vec::with_capacity(rows.len());
    let mut volume_up = Vec::with_capacity(rows.len());

    for row in rows {
        if row.len() < 6 {
            return Err("invalid kline row length".to_owned());
        }

        let open_time_secs = row[0]
            .as_i64()
            .ok_or_else(|| "invalid open time".to_owned())? as f64
            / 1_000.0;
        let open = parse_value_f64(&row[1], "open")?;
        let high = parse_value_f64(&row[2], "high")?;
        let low = parse_value_f64(&row[3], "low")?;
        let close = parse_value_f64(&row[4], "close")?;
        let volume = parse_value_f64(&row[5], "volume")?;

        let candle = OhlcBar::new(open_time_secs, open, high, low, close)
            .map_err(|err| format!("invalid ohlc row: {err}"))?;

        candles.push(candle);
        close_points.push(DataPoint::new(open_time_secs, close));
        volumes.push(DataPoint::new(open_time_secs, volume.max(0.0)));
        volume_up.push(close >= open);
    }

    Ok(MarketData {
        symbol: symbol.to_owned(),
        interval: interval.to_owned(),
        candles,
        close_points,
        volumes,
        volume_up,
        source_label: source_label.to_owned(),
    })
}

fn parse_value_f64(value: &Value, name: &str) -> Result<f64, String> {
    let raw = value
        .as_str()
        .ok_or_else(|| format!("invalid {name}: expected string"))?;
    raw.parse::<f64>()
        .map_err(|err| format!("invalid {name}: {err}"))
}

fn fallback_profile(interval: &str) -> (f64, f64, f64) {
    match interval {
        "1m" => (1_700_000_000.0, 60.0, 42_000.0),
        "5m" => (1_700_000_000.0, 300.0, 42_000.0),
        "15m" => (1_700_000_000.0, 900.0, 42_000.0),
        "1h" => (1_700_000_000.0, 3_600.0, 42_000.0),
        "4h" => (1_700_000_000.0, 14_400.0, 42_000.0),
        "1d" => (1_700_000_000.0, 86_400.0, 42_000.0),
        _ => (1_700_000_000.0, 3_600.0, 42_000.0),
    }
}
