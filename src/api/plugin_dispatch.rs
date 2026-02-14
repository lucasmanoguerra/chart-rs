use crate::extensions::{PluginContext, PluginEvent};
use crate::render::Renderer;
use tracing::warn;

use super::{ChartEngine, InvalidationLevel, InvalidationTopic, InvalidationTopics};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn plugin_context(&self) -> PluginContext {
        PluginContext {
            viewport: self.core.model.viewport,
            time_visible_range: self.core.model.time_scale.visible_range(),
            price_domain: self.core.model.price_scale.domain(),
            points_len: self.core.model.points.len(),
            candles_len: self.core.model.candles.len(),
            interaction_mode: self.core.model.interaction.mode(),
            crosshair: self.core.model.interaction.crosshair(),
        }
    }

    pub(super) fn emit_plugin_event(&mut self, event: PluginEvent) {
        match event {
            PluginEvent::DataUpdated { .. } | PluginEvent::CandlesUpdated { .. } => {
                self.invalidate_with_detail(
                    InvalidationLevel::Full,
                    InvalidationTopics::from_topic(InvalidationTopic::Series)
                        .with_topic(InvalidationTopic::PriceScale)
                        .with_topic(InvalidationTopic::Axis),
                    None,
                );
            }
            PluginEvent::VisibleRangeChanged { .. } => {}
            PluginEvent::PointerMoved { .. } | PluginEvent::PointerLeft => {
                self.invalidate_cursor();
            }
            PluginEvent::PanStarted | PluginEvent::PanEnded => {
                self.invalidate_cursor();
            }
            PluginEvent::Rendered => {}
        }

        let context = self.plugin_context();
        for plugin in &mut self.core.runtime.plugins {
            plugin.on_event(event, context);
        }
    }

    fn maybe_autoscale_price_after_time_range_change(&mut self) {
        if !self
            .core
            .behavior
            .price_scale_realtime_behavior
            .autoscale_on_time_range_change
        {
            return;
        }

        let autoscale_result = if !self.core.model.candles.is_empty() {
            self.autoscale_price_from_visible_candles()
        } else if !self.core.model.points.is_empty() {
            self.autoscale_price_from_visible_data()
        } else {
            Ok(())
        };

        if let Err(err) = autoscale_result {
            warn!(
                error = %err,
                "skipping visible-range price autoscale due to invalid data/mode combination"
            );
        }
    }

    pub(super) fn emit_visible_range_changed(&mut self) {
        if let Err(err) = self.refresh_price_scale_transformed_base() {
            warn!(
                error = %err,
                "skipping transformed-base refresh on visible-range change"
            );
        }
        self.clear_crosshair_context_formatter_caches_if_needed();
        self.maybe_autoscale_price_after_time_range_change();
        self.invalidate_with_detail(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::TimeScale)
                .with_topic(InvalidationTopic::Axis),
            None,
        );
        let (start, end) = self.core.model.time_scale.visible_range();
        self.emit_plugin_event(PluginEvent::VisibleRangeChanged { start, end });
    }
}
