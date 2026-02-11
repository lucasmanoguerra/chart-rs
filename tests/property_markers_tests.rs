use chart_rs::core::{OhlcBar, PriceScale, TimeScale, Viewport};
use chart_rs::extensions::{
    MarkerPlacementConfig, MarkerPosition, MarkerSide, SeriesMarker, place_markers_on_candles,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn placed_markers_do_not_overlap_within_side_lane(
        candle_count in 8usize..64,
        marker_count in 8usize..128,
        seed in 0u64..1_000_000u64
    ) {
        let candles: Vec<OhlcBar> = (0..candle_count)
            .map(|i| {
                let t = i as f64;
                let open = 100.0 + (i as f64 * 0.3);
                let close = if i % 2 == 0 { open + 1.5 } else { open - 1.5 };
                let low = open.min(close) - 1.0;
                let high = open.max(close) + 1.0;
                OhlcBar::new(t, open, high, low, close).expect("valid candle")
            })
            .collect();

        let mut markers = Vec::with_capacity(marker_count);
        for i in 0..marker_count {
            let idx = ((seed as usize + i * 37) % candle_count) as f64;
            let position = if i % 2 == 0 {
                MarkerPosition::AboveBar
            } else {
                MarkerPosition::BelowBar
            };
            let text = if i % 3 == 0 { "long-label" } else { "m" };
            markers.push(
                SeriesMarker::new(format!("m-{i}"), idx, position)
                    .with_text(text)
                    .with_priority((i % 5) as i32),
            );
        }

        let config = MarkerPlacementConfig::default();
        let placed = place_markers_on_candles(
            &markers,
            &candles,
            TimeScale::new(0.0, candle_count as f64).expect("time scale"),
            PriceScale::new(0.0, 300.0).expect("price scale"),
            Viewport::new(1600, 900),
            config,
        ).expect("placement");

        for marker in &placed {
            prop_assert!(marker.x.is_finite());
            prop_assert!(marker.y.is_finite());
            prop_assert!(marker.collision_left_px <= marker.collision_right_px);
            prop_assert!(marker.x >= 0.0);
            prop_assert!(marker.x <= 1600.0);
            prop_assert!(matches!(marker.side, MarkerSide::Above | MarkerSide::Below | MarkerSide::Center));
        }

        for i in 0..placed.len() {
            for j in (i + 1)..placed.len() {
                let a = &placed[i];
                let b = &placed[j];
                if a.side == b.side && a.lane == b.lane {
                    let non_overlap =
                        a.collision_right_px + config.min_horizontal_gap_px <= b.collision_left_px
                            || b.collision_right_px + config.min_horizontal_gap_px <= a.collision_left_px;
                    prop_assert!(non_overlap);
                }
            }
        }
    }
}
