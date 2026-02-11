use chrono::TimeZone;
use chrono::Utc;
use rust_decimal::Decimal;

use chart_rs::core::{DataPoint, OhlcBar};

#[test]
fn data_point_from_decimal_time_is_supported() {
    let time = Utc
        .timestamp_opt(1_700_000_000, 0)
        .single()
        .expect("valid ts");
    let point = DataPoint::from_decimal_time(time, Decimal::new(12345, 2)).expect("point");

    assert!((point.x - 1_700_000_000.0).abs() <= 1e-6);
    assert!((point.y - 123.45).abs() <= 1e-9);
}

#[test]
fn ohlc_from_decimal_time_is_supported() {
    let time = Utc
        .timestamp_opt(1_700_000_100, 0)
        .single()
        .expect("valid ts");
    let bar = OhlcBar::from_decimal_time(
        time,
        Decimal::new(1000, 1),
        Decimal::new(1200, 1),
        Decimal::new(900, 1),
        Decimal::new(1100, 1),
    )
    .expect("ohlc");

    assert!((bar.time - 1_700_000_100.0).abs() <= 1e-6);
    assert!((bar.open - 100.0).abs() <= 1e-9);
    assert!((bar.high - 120.0).abs() <= 1e-9);
    assert!((bar.low - 90.0).abs() <= 1e-9);
    assert!((bar.close - 110.0).abs() <= 1e-9);
}
