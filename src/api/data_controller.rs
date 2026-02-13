use std::cmp::Ordering;

use tracing::{debug, trace, warn};

use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::{ChartEngine, PluginEvent};

impl<R: Renderer> ChartEngine<R> {
    /// Replaces line/point data series.
    pub fn set_data(&mut self, points: Vec<crate::core::DataPoint>) {
        let original_count = points.len();
        let points = canonicalize_points(points);
        debug!(
            original_count,
            canonical_count = points.len(),
            "set data points"
        );
        self.points = points;
        self.maybe_autoscale_price_after_data_set_points();
        if let Err(err) = self.refresh_price_scale_transformed_base() {
            warn!(
                error = %err,
                "skipping transformed-base refresh after set_data"
            );
        }
        self.emit_plugin_event(PluginEvent::DataUpdated {
            points_len: self.points.len(),
        });
    }

    /// Appends a single line/point sample.
    pub fn append_point(&mut self, point: crate::core::DataPoint) {
        self.points.push(point);
        trace!(count = self.points.len(), "append data point");
        let visible_range_changed = self.handle_realtime_time_append(point.x);
        self.maybe_autoscale_price_after_realtime_data_update();
        if let Err(err) = self.refresh_price_scale_transformed_base() {
            warn!(
                error = %err,
                "skipping transformed-base refresh after append_point"
            );
        }
        self.emit_point_data_updated(visible_range_changed);
    }

    /// Updates point series using realtime-update semantics:
    /// - appends when `point.x` is newer than the latest sample
    /// - replaces the latest sample when `point.x` is equal
    /// - rejects out-of-order updates (`point.x` older than latest sample)
    pub fn update_point(&mut self, point: crate::core::DataPoint) -> ChartResult<()> {
        if !point.x.is_finite() {
            return Err(ChartError::InvalidData(
                "point time must be finite".to_owned(),
            ));
        }

        let mut visible_range_changed = false;
        match self
            .points
            .last()
            .map_or(Ordering::Greater, |last| point.x.total_cmp(&last.x))
        {
            Ordering::Less => {
                return Err(ChartError::InvalidData(
                    "point update time must be >= latest point time".to_owned(),
                ));
            }
            Ordering::Equal => {
                if let Some(last) = self.points.last_mut() {
                    *last = point;
                } else {
                    self.points.push(point);
                    visible_range_changed = self.handle_realtime_time_append(point.x);
                }
            }
            Ordering::Greater => {
                self.points.push(point);
                visible_range_changed = self.handle_realtime_time_append(point.x);
            }
        }

        trace!(count = self.points.len(), "update data point");
        self.maybe_autoscale_price_after_realtime_data_update();
        if let Err(err) = self.refresh_price_scale_transformed_base() {
            warn!(
                error = %err,
                "skipping transformed-base refresh after update_point"
            );
        }
        self.emit_point_data_updated(visible_range_changed);
        Ok(())
    }

    /// Replaces candlestick series.
    pub fn set_candles(&mut self, candles: Vec<crate::core::OhlcBar>) {
        let original_count = candles.len();
        let candles = canonicalize_candles(candles);
        debug!(
            original_count,
            canonical_count = candles.len(),
            "set candles"
        );
        self.candles = candles;
        self.maybe_autoscale_price_after_data_set_candles();
        if let Err(err) = self.refresh_price_scale_transformed_base() {
            warn!(
                error = %err,
                "skipping transformed-base refresh after set_candles"
            );
        }
        self.emit_plugin_event(PluginEvent::CandlesUpdated {
            candles_len: self.candles.len(),
        });
    }

    /// Appends a single OHLC bar.
    pub fn append_candle(&mut self, candle: crate::core::OhlcBar) {
        self.candles.push(candle);
        trace!(count = self.candles.len(), "append candle");
        let visible_range_changed = self.handle_realtime_time_append(candle.time);
        self.maybe_autoscale_price_after_realtime_data_update();
        if let Err(err) = self.refresh_price_scale_transformed_base() {
            warn!(
                error = %err,
                "skipping transformed-base refresh after append_candle"
            );
        }
        self.emit_candle_data_updated(visible_range_changed);
    }

    /// Updates candle series using realtime-update semantics:
    /// - appends when `candle.time` is newer than the latest sample
    /// - replaces the latest sample when `candle.time` is equal
    /// - rejects out-of-order updates (`candle.time` older than latest sample)
    pub fn update_candle(&mut self, candle: crate::core::OhlcBar) -> ChartResult<()> {
        if !candle.time.is_finite() {
            return Err(ChartError::InvalidData(
                "candle time must be finite".to_owned(),
            ));
        }

        let mut visible_range_changed = false;
        match self
            .candles
            .last()
            .map_or(Ordering::Greater, |last| candle.time.total_cmp(&last.time))
        {
            Ordering::Less => {
                return Err(ChartError::InvalidData(
                    "candle update time must be >= latest candle time".to_owned(),
                ));
            }
            Ordering::Equal => {
                if let Some(last) = self.candles.last_mut() {
                    *last = candle;
                } else {
                    self.candles.push(candle);
                    visible_range_changed = self.handle_realtime_time_append(candle.time);
                }
            }
            Ordering::Greater => {
                self.candles.push(candle);
                visible_range_changed = self.handle_realtime_time_append(candle.time);
            }
        }

        trace!(count = self.candles.len(), "update candle");
        self.maybe_autoscale_price_after_realtime_data_update();
        if let Err(err) = self.refresh_price_scale_transformed_base() {
            warn!(
                error = %err,
                "skipping transformed-base refresh after update_candle"
            );
        }
        self.emit_candle_data_updated(visible_range_changed);
        Ok(())
    }

    fn maybe_autoscale_price_after_realtime_data_update(&mut self) {
        if !self.price_scale_realtime_behavior.autoscale_on_data_update {
            return;
        }

        let autoscale_result = if !self.candles.is_empty() {
            self.autoscale_price_from_candles()
        } else if !self.points.is_empty() {
            self.autoscale_price_from_data()
        } else {
            Ok(())
        };

        if let Err(err) = autoscale_result {
            warn!(
                error = %err,
                "skipping realtime price autoscale due to invalid data/mode combination"
            );
        }
    }

    fn maybe_autoscale_price_after_data_set_points(&mut self) {
        if !self.price_scale_realtime_behavior.autoscale_on_data_set || self.points.is_empty() {
            return;
        }
        if let Err(err) = self.autoscale_price_from_data() {
            warn!(
                error = %err,
                "skipping set-data price autoscale due to invalid data/mode combination"
            );
        }
    }

    fn maybe_autoscale_price_after_data_set_candles(&mut self) {
        if !self.price_scale_realtime_behavior.autoscale_on_data_set || self.candles.is_empty() {
            return;
        }
        if let Err(err) = self.autoscale_price_from_candles() {
            warn!(
                error = %err,
                "skipping set-candles price autoscale due to invalid data/mode combination"
            );
        }
    }

    fn emit_point_data_updated(&mut self, visible_range_changed: bool) {
        self.emit_plugin_event(PluginEvent::DataUpdated {
            points_len: self.points.len(),
        });
        if visible_range_changed {
            self.emit_visible_range_changed();
        }
    }

    fn emit_candle_data_updated(&mut self, visible_range_changed: bool) {
        self.emit_plugin_event(PluginEvent::CandlesUpdated {
            candles_len: self.candles.len(),
        });
        if visible_range_changed {
            self.emit_visible_range_changed();
        }
    }
}

fn canonicalize_points(mut points: Vec<crate::core::DataPoint>) -> Vec<crate::core::DataPoint> {
    let original_len = points.len();
    points.retain(|point| point.x.is_finite() && point.y.is_finite());
    points.sort_by(|a, b| a.x.total_cmp(&b.x));

    let mut deduped: Vec<crate::core::DataPoint> = Vec::with_capacity(points.len());
    let mut duplicate_count = 0_usize;
    for point in points {
        if let Some(last) = deduped.last_mut() {
            if point.x.total_cmp(&last.x) == Ordering::Equal {
                *last = point;
                duplicate_count += 1;
                continue;
            }
        }
        deduped.push(point);
    }

    let filtered_count = original_len.saturating_sub(deduped.len() + duplicate_count);
    if filtered_count > 0 || duplicate_count > 0 {
        warn!(
            filtered_count,
            duplicate_count,
            canonical_count = deduped.len(),
            "canonicalized points on set_data"
        );
    }
    deduped
}

fn canonicalize_candles(mut candles: Vec<crate::core::OhlcBar>) -> Vec<crate::core::OhlcBar> {
    let original_len = candles.len();
    candles.retain(is_valid_candle);
    candles.sort_by(|a, b| a.time.total_cmp(&b.time));

    let mut deduped: Vec<crate::core::OhlcBar> = Vec::with_capacity(candles.len());
    let mut duplicate_count = 0_usize;
    for candle in candles {
        if let Some(last) = deduped.last_mut() {
            if candle.time.total_cmp(&last.time) == Ordering::Equal {
                *last = candle;
                duplicate_count += 1;
                continue;
            }
        }
        deduped.push(candle);
    }

    let filtered_count = original_len.saturating_sub(deduped.len() + duplicate_count);
    if filtered_count > 0 || duplicate_count > 0 {
        warn!(
            filtered_count,
            duplicate_count,
            canonical_count = deduped.len(),
            "canonicalized candles on set_candles"
        );
    }
    deduped
}

fn is_valid_candle(candle: &crate::core::OhlcBar) -> bool {
    if !candle.time.is_finite()
        || !candle.open.is_finite()
        || !candle.high.is_finite()
        || !candle.low.is_finite()
        || !candle.close.is_finite()
    {
        return false;
    }
    candle.low <= candle.high
        && candle.open >= candle.low
        && candle.open <= candle.high
        && candle.close >= candle.low
        && candle.close <= candle.high
}
