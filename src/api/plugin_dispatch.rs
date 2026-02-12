use crate::extensions::PluginContext;
use crate::render::Renderer;
use tracing::warn;

use super::{ChartEngine, PluginEvent};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn plugin_context(&self) -> PluginContext {
        PluginContext {
            viewport: self.viewport,
            time_visible_range: self.time_scale.visible_range(),
            price_domain: self.price_scale.domain(),
            points_len: self.points.len(),
            candles_len: self.candles.len(),
            interaction_mode: self.interaction.mode(),
            crosshair: self.interaction.crosshair(),
        }
    }

    pub(super) fn emit_plugin_event(&mut self, event: PluginEvent) {
        let context = self.plugin_context();
        for plugin in &mut self.plugins {
            plugin.on_event(event, context);
        }
    }

    fn maybe_autoscale_price_after_time_range_change(&mut self) {
        if !self
            .price_scale_realtime_behavior
            .autoscale_on_time_range_change
        {
            return;
        }

        let autoscale_result = if !self.candles.is_empty() {
            self.autoscale_price_from_visible_candles()
        } else if !self.points.is_empty() {
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
        self.clear_crosshair_context_formatter_caches_if_needed();
        self.maybe_autoscale_price_after_time_range_change();
        let (start, end) = self.time_scale.visible_range();
        self.emit_plugin_event(PluginEvent::VisibleRangeChanged { start, end });
    }
}
