use crate::core::{DataPoint, OhlcBar, PriceScaleMode as CorePriceScaleMode};
use crate::error::ChartResult;
use crate::render::Renderer;

use super::{ChartEngine, InvalidationTopic, InvalidationTopics};

#[derive(Debug, Clone, Copy)]
struct PriceScaleSyncSnapshot {
    price_min: f64,
    price_max: f64,
    top_margin_ratio: f64,
    bottom_margin_ratio: f64,
    inverted: bool,
    mode: crate::lwc::model::PriceScaleMode,
    base_value: Option<f64>,
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn sync_lwc_model_from_core(&mut self) -> ChartResult<()> {
        self.sync_lwc_time_scale_from_core()?;
        self.sync_lwc_price_scales_from_core()
    }

    pub(super) fn sync_lwc_model_for_invalidation_topics(
        &mut self,
        topics: InvalidationTopics,
    ) -> ChartResult<()> {
        if topics.is_none() {
            return Ok(());
        }

        let sync_time_scale = topics.contains_topic(InvalidationTopic::General)
            || topics.contains_topic(InvalidationTopic::TimeScale)
            || topics.contains_topic(InvalidationTopic::Series)
            || topics.contains_topic(InvalidationTopic::PaneLayout);
        let sync_price_scale = topics.contains_topic(InvalidationTopic::General)
            || topics.contains_topic(InvalidationTopic::PriceScale)
            || topics.contains_topic(InvalidationTopic::Series)
            || topics.contains_topic(InvalidationTopic::PaneLayout)
            || topics.contains_topic(InvalidationTopic::TimeScale);

        if sync_time_scale {
            self.sync_lwc_time_scale_from_core()?;
        }
        if sync_price_scale {
            self.sync_lwc_price_scales_from_core()?;
        }
        Ok(())
    }

    fn sync_lwc_time_scale_from_core(&mut self) -> ChartResult<()> {
        let viewport_width = f64::from(self.core.model.viewport.width);
        if !viewport_width.is_finite() || viewport_width <= 0.0 {
            return Ok(());
        }

        let sorted_times =
            collect_sorted_unique_times(&self.core.model.points, &self.core.model.candles);
        let points = sorted_times
            .iter()
            .map(|time| crate::lwc::model::TimeScalePoint {
                time: *time,
                original_time: Some(*time),
            })
            .collect::<Vec<_>>();
        let base_index = sorted_times
            .len()
            .checked_sub(1)
            .map(|index| index as crate::lwc::model::TimePointIndex);
        let reference_step = estimate_positive_time_step(&sorted_times).or_else(|| {
            Some(fallback_reference_step(
                self.core.model.time_scale.full_range(),
                self.core.model.time_scale.visible_range(),
            ))
        });
        let derived_bar_spacing_and_offset = reference_step.and_then(|step| {
            self.core
                .model
                .time_scale
                .derive_visible_bar_spacing_and_right_offset(step, viewport_width)
                .ok()
        });

        let lwc_time_scale = self.core.lwc_model.time_scale_mut();
        lwc_time_scale.set_width(viewport_width)?;
        lwc_time_scale.set_points(points);
        lwc_time_scale.set_base_index(base_index)?;
        if let Some((bar_spacing, right_offset)) = derived_bar_spacing_and_offset {
            lwc_time_scale.set_bar_spacing(bar_spacing)?;
            lwc_time_scale.set_right_offset(right_offset)?;
        }
        self.core.runtime.last_lwc_time_scale_state =
            Some(super::chart_runtime::LwcTimeScaleStateSnapshot {
                bar_spacing: lwc_time_scale.bar_spacing(),
                right_offset: lwc_time_scale.right_offset(),
            });
        Ok(())
    }

    fn sync_lwc_price_scales_from_core(&mut self) -> ChartResult<()> {
        let viewport_height = f64::from(self.core.model.viewport.height);
        if !viewport_height.is_finite() || viewport_height <= 0.0 {
            return Ok(());
        }

        let pane_ids = self
            .core
            .model
            .pane_collection
            .panes()
            .iter()
            .map(|pane| pane.id)
            .collect::<Vec<_>>();
        let pane_regions = self
            .core
            .model
            .pane_collection
            .layout_regions(0.0, viewport_height);

        let (price_min, price_max) = self.core.model.price_scale.domain();
        let (top_margin_ratio, bottom_margin_ratio) = self.core.model.price_scale.margins();
        let snapshot = PriceScaleSyncSnapshot {
            price_min,
            price_max,
            top_margin_ratio,
            bottom_margin_ratio,
            inverted: self.core.model.price_scale.is_inverted(),
            mode: map_price_scale_mode(self.core.model.price_scale_mode),
            base_value: self.core.model.price_scale.base_value(),
        };

        self.core.lwc_model.sync_panes(&pane_ids);
        for region in pane_regions {
            if let Some(pane) = self.core.lwc_model.pane_by_id_mut(region.pane_id) {
                pane.set_height(region.height());
                sync_single_price_scale(pane.left_price_scale_mut(), snapshot)?;
                sync_single_price_scale(pane.right_price_scale_mut(), snapshot)?;
            }
        }
        Ok(())
    }
}

fn sync_single_price_scale(
    scale: &mut crate::lwc::model::PriceScale,
    snapshot: PriceScaleSyncSnapshot,
) -> ChartResult<()> {
    let mut options = scale.options();
    options.auto_scale = false;
    options.scale_margins = crate::lwc::model::PriceScaleMargins {
        top: snapshot.top_margin_ratio,
        bottom: snapshot.bottom_margin_ratio,
    };
    scale.apply_options(options)?;

    scale.set_custom_price_range(Some(crate::lwc::model::PriceRange::new(
        snapshot.price_min,
        snapshot.price_max,
    )));
    scale.set_mode(crate::lwc::model::PriceScaleStateChange {
        mode: Some(snapshot.mode),
        is_inverted: Some(snapshot.inverted),
        auto_scale: None,
    });

    if let Some(transformed_range) = transformed_custom_range(snapshot) {
        scale.set_custom_price_range(Some(transformed_range));
    }

    scale.set_mode(crate::lwc::model::PriceScaleStateChange {
        auto_scale: Some(false),
        ..crate::lwc::model::PriceScaleStateChange::default()
    });
    Ok(())
}

fn collect_sorted_unique_times(points: &[DataPoint], candles: &[OhlcBar]) -> Vec<f64> {
    let mut times = Vec::with_capacity(points.len() + candles.len());
    for point in points {
        if point.x.is_finite() {
            times.push(point.x);
        }
    }
    for candle in candles {
        if candle.time.is_finite() {
            times.push(candle.time);
        }
    }
    times.sort_by(|left, right| left.total_cmp(right));
    times.dedup_by(|left, right| (*left - *right).abs() <= 1e-9);
    times
}

fn estimate_positive_time_step(sorted_times: &[f64]) -> Option<f64> {
    if sorted_times.len() < 2 {
        return None;
    }

    let mut deltas = Vec::with_capacity(sorted_times.len().saturating_sub(1));
    for window in sorted_times.windows(2) {
        let delta = window[1] - window[0];
        if delta.is_finite() && delta > 0.0 {
            deltas.push(delta);
        }
    }
    if deltas.is_empty() {
        return None;
    }

    deltas.sort_by(|left, right| left.total_cmp(right));
    let mid = deltas.len() / 2;
    if deltas.len() % 2 == 1 {
        Some(deltas[mid])
    } else {
        Some((deltas[mid - 1] + deltas[mid]) * 0.5)
    }
}

fn fallback_reference_step(full_range: (f64, f64), visible_range: (f64, f64)) -> f64 {
    let full_span = (full_range.1 - full_range.0).abs();
    if full_span.is_finite() && full_span > 0.0 {
        return full_span;
    }

    let visible_span = (visible_range.1 - visible_range.0).abs();
    if visible_span.is_finite() && visible_span > 0.0 {
        return visible_span;
    }

    1.0
}

fn map_price_scale_mode(mode: CorePriceScaleMode) -> crate::lwc::model::PriceScaleMode {
    match mode {
        CorePriceScaleMode::Linear => crate::lwc::model::PriceScaleMode::Normal,
        CorePriceScaleMode::Log => crate::lwc::model::PriceScaleMode::Logarithmic,
        CorePriceScaleMode::Percentage => crate::lwc::model::PriceScaleMode::Percentage,
        CorePriceScaleMode::IndexedTo100 => crate::lwc::model::PriceScaleMode::IndexedTo100,
    }
}

fn transformed_custom_range(
    snapshot: PriceScaleSyncSnapshot,
) -> Option<crate::lwc::model::PriceRange> {
    let base = snapshot.base_value?;
    if !base.is_finite() || base == 0.0 {
        return None;
    }

    let (a, b) = match snapshot.mode {
        crate::lwc::model::PriceScaleMode::Percentage => (
            to_percent(snapshot.price_min, base),
            to_percent(snapshot.price_max, base),
        ),
        crate::lwc::model::PriceScaleMode::IndexedTo100 => (
            to_indexed_to_100(snapshot.price_min, base),
            to_indexed_to_100(snapshot.price_max, base),
        ),
        crate::lwc::model::PriceScaleMode::Normal
        | crate::lwc::model::PriceScaleMode::Logarithmic => {
            return None;
        }
    };
    Some(crate::lwc::model::PriceRange::new(a.min(b), a.max(b)))
}

fn to_percent(value: f64, base_value: f64) -> f64 {
    let result = 100.0 * (value - base_value) / base_value;
    if base_value < 0.0 { -result } else { result }
}

fn to_indexed_to_100(value: f64, base_value: f64) -> f64 {
    let result = 100.0 * (value - base_value) / base_value + 100.0;
    if base_value < 0.0 { -result } else { result }
}

#[cfg(test)]
mod tests {
    use super::{
        PriceScaleSyncSnapshot, collect_sorted_unique_times, estimate_positive_time_step,
        transformed_custom_range,
    };
    use crate::core::{DataPoint, OhlcBar};

    #[test]
    fn collect_sorted_unique_times_merges_points_and_candles() {
        let points = vec![DataPoint::new(3.0, 10.0), DataPoint::new(1.0, 11.0)];
        let candles = vec![
            OhlcBar::new(2.0, 1.0, 2.0, 0.5, 1.5).expect("bar"),
            OhlcBar::new(3.0, 1.0, 2.0, 0.5, 1.5).expect("bar"),
        ];

        let times = collect_sorted_unique_times(&points, &candles);
        assert_eq!(times, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn estimate_positive_time_step_uses_median_delta() {
        let sorted = vec![1.0, 2.0, 4.0, 7.0];
        let step = estimate_positive_time_step(&sorted).expect("step");
        assert!((step - 2.0).abs() <= 1e-9);
    }

    #[test]
    fn transformed_custom_range_maps_percentage_and_indexed_modes() {
        let percent = transformed_custom_range(PriceScaleSyncSnapshot {
            price_min: 90.0,
            price_max: 110.0,
            top_margin_ratio: 0.2,
            bottom_margin_ratio: 0.1,
            inverted: false,
            mode: crate::lwc::model::PriceScaleMode::Percentage,
            base_value: Some(100.0),
        })
        .expect("range");
        assert!((percent.min() + 10.0).abs() <= 1e-9);
        assert!((percent.max() - 10.0).abs() <= 1e-9);

        let indexed = transformed_custom_range(PriceScaleSyncSnapshot {
            price_min: 90.0,
            price_max: 110.0,
            top_margin_ratio: 0.2,
            bottom_margin_ratio: 0.1,
            inverted: false,
            mode: crate::lwc::model::PriceScaleMode::IndexedTo100,
            base_value: Some(100.0),
        })
        .expect("range");
        assert!((indexed.min() - 90.0).abs() <= 1e-9);
        assert!((indexed.max() - 110.0).abs() <= 1e-9);
    }
}
