use tracing::{debug, trace};

use crate::render::Renderer;

use super::{ChartEngine, PluginEvent};

impl<R: Renderer> ChartEngine<R> {
    /// Replaces line/point data series.
    pub fn set_data(&mut self, points: Vec<crate::core::DataPoint>) {
        debug!(count = points.len(), "set data points");
        self.points = points;
        self.emit_plugin_event(PluginEvent::DataUpdated {
            points_len: self.points.len(),
        });
    }

    /// Appends a single line/point sample.
    pub fn append_point(&mut self, point: crate::core::DataPoint) {
        self.points.push(point);
        trace!(count = self.points.len(), "append data point");
        self.emit_plugin_event(PluginEvent::DataUpdated {
            points_len: self.points.len(),
        });
    }

    /// Replaces candlestick series.
    pub fn set_candles(&mut self, candles: Vec<crate::core::OhlcBar>) {
        debug!(count = candles.len(), "set candles");
        self.candles = candles;
        self.emit_plugin_event(PluginEvent::CandlesUpdated {
            candles_len: self.candles.len(),
        });
    }

    /// Appends a single OHLC bar.
    pub fn append_candle(&mut self, candle: crate::core::OhlcBar) {
        self.candles.push(candle);
        trace!(count = self.candles.len(), "append candle");
        self.emit_plugin_event(PluginEvent::CandlesUpdated {
            candles_len: self.candles.len(),
        });
    }
}
