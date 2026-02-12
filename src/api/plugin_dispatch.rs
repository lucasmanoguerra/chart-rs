use crate::render::Renderer;

use super::{ChartEngine, PluginContext, PluginEvent};

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

    pub(super) fn emit_visible_range_changed(&mut self) {
        let (start, end) = self.time_scale.visible_range();
        self.emit_plugin_event(PluginEvent::VisibleRangeChanged { start, end });
    }
}
