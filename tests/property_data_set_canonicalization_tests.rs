use chart_rs::api::{ChartEngine, ChartEngineConfig};
use chart_rs::core::{DataPoint, OhlcBar, Viewport};
use chart_rs::render::NullRenderer;
use proptest::prelude::*;

fn engine() -> ChartEngine<NullRenderer> {
    let renderer = NullRenderer::default();
    let config = ChartEngineConfig::new(Viewport::new(1200, 700), -100.0, 100.0)
        .with_price_domain(-1_000.0, 1_000.0);
    ChartEngine::new(renderer, config).expect("engine init")
}

fn point_value_strategy() -> impl Strategy<Value = f64> {
    prop_oneof![
        -10_000.0f64..10_000.0,
        Just(f64::NAN),
        Just(f64::INFINITY),
        Just(f64::NEG_INFINITY),
    ]
}

fn candle_value_strategy() -> impl Strategy<Value = f64> {
    prop_oneof![
        -1_000.0f64..1_000.0,
        Just(f64::NAN),
        Just(f64::INFINITY),
        Just(f64::NEG_INFINITY),
    ]
}

fn canonicalize_points_contract(mut points: Vec<DataPoint>) -> Vec<DataPoint> {
    points.retain(|point| point.x.is_finite() && point.y.is_finite());
    points.sort_by(|a, b| a.x.total_cmp(&b.x));

    let mut deduped: Vec<DataPoint> = Vec::with_capacity(points.len());
    for point in points {
        if let Some(last) = deduped.last_mut() {
            if point.x.total_cmp(&last.x) == std::cmp::Ordering::Equal {
                *last = point;
                continue;
            }
        }
        deduped.push(point);
    }
    deduped
}

fn is_valid_candle(candle: &OhlcBar) -> bool {
    candle.time.is_finite()
        && candle.open.is_finite()
        && candle.high.is_finite()
        && candle.low.is_finite()
        && candle.close.is_finite()
        && candle.low <= candle.high
        && candle.open >= candle.low
        && candle.open <= candle.high
        && candle.close >= candle.low
        && candle.close <= candle.high
}

fn canonicalize_candles_contract(mut candles: Vec<OhlcBar>) -> Vec<OhlcBar> {
    candles.retain(is_valid_candle);
    candles.sort_by(|a, b| a.time.total_cmp(&b.time));

    let mut deduped: Vec<OhlcBar> = Vec::with_capacity(candles.len());
    for candle in candles {
        if let Some(last) = deduped.last_mut() {
            if candle.time.total_cmp(&last.time) == std::cmp::Ordering::Equal {
                *last = candle;
                continue;
            }
        }
        deduped.push(candle);
    }
    deduped
}

proptest! {
    #[test]
    fn set_data_matches_canonicalization_contract(
        raw in prop::collection::vec((point_value_strategy(), point_value_strategy()), 0..128)
    ) {
        let mut engine = engine();
        let input: Vec<DataPoint> = raw
            .into_iter()
            .map(|(x, y)| DataPoint::new(x, y))
            .collect();

        let expected = canonicalize_points_contract(input.clone());
        engine.set_data(input);

        let output = engine.points();
        prop_assert_eq!(output, expected.as_slice());
        for window in output.windows(2) {
            prop_assert!(window[0].x.total_cmp(&window[1].x) == std::cmp::Ordering::Less);
        }
        for point in output {
            prop_assert!(point.x.is_finite());
            prop_assert!(point.y.is_finite());
        }
    }

    #[test]
    fn set_candles_matches_canonicalization_contract(
        raw in prop::collection::vec(
            (
                candle_value_strategy(),
                candle_value_strategy(),
                candle_value_strategy(),
                candle_value_strategy(),
                candle_value_strategy(),
            ),
            0..128,
        )
    ) {
        let mut engine = engine();
        let input: Vec<OhlcBar> = raw
            .into_iter()
            .map(|(time, open, high, low, close)| OhlcBar {
                time,
                open,
                high,
                low,
                close,
            })
            .collect();

        let expected = canonicalize_candles_contract(input.clone());
        engine.set_candles(input);

        let output = engine.candles();
        prop_assert_eq!(output, expected.as_slice());
        for window in output.windows(2) {
            prop_assert!(window[0].time.total_cmp(&window[1].time) == std::cmp::Ordering::Less);
        }
        for candle in output {
            prop_assert!(is_valid_candle(candle));
        }
    }
}
