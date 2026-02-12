use crate::error::{ChartError, ChartResult};
use crate::extensions::ChartPlugin;
use crate::render::Renderer;

use super::ChartEngine;

impl<R: Renderer> ChartEngine<R> {
    /// Registers a plugin with unique identifier.
    pub fn register_plugin(&mut self, plugin: Box<dyn ChartPlugin>) -> ChartResult<()> {
        let plugin_id = plugin.id().to_owned();
        if plugin_id.is_empty() {
            return Err(ChartError::InvalidData(
                "plugin id must not be empty".to_owned(),
            ));
        }
        if self.plugins.iter().any(|entry| entry.id() == plugin_id) {
            return Err(ChartError::InvalidData(format!(
                "plugin with id `{plugin_id}` is already registered"
            )));
        }
        self.plugins.push(plugin);
        Ok(())
    }

    /// Unregisters a plugin by id. Returns `true` when removed.
    pub fn unregister_plugin(&mut self, plugin_id: &str) -> bool {
        if let Some(position) = self
            .plugins
            .iter()
            .position(|entry| entry.id() == plugin_id)
        {
            self.plugins.remove(position);
            return true;
        }
        false
    }

    #[must_use]
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    #[must_use]
    pub fn has_plugin(&self, plugin_id: &str) -> bool {
        self.plugins.iter().any(|plugin| plugin.id() == plugin_id)
    }
}
