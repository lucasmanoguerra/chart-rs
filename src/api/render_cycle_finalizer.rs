use crate::extensions::PluginEvent;
use crate::render::Renderer;

use super::ChartEngine;

pub(super) fn finalize_render_cycle<R: Renderer>(engine: &mut ChartEngine<R>) {
    engine.clear_pending_invalidation();
    engine.emit_plugin_event(PluginEvent::Rendered);
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::finalize_render_cycle;
    use crate::api::{ChartEngine, ChartEngineConfig};
    use crate::core::Viewport;
    use crate::extensions::{ChartPlugin, PluginContext, PluginEvent};
    use crate::render::NullRenderer;

    #[derive(Clone)]
    struct RecordingPlugin {
        id: String,
        events: Rc<RefCell<Vec<PluginEvent>>>,
    }

    impl RecordingPlugin {
        fn new(id: impl Into<String>, events: Rc<RefCell<Vec<PluginEvent>>>) -> Self {
            Self {
                id: id.into(),
                events,
            }
        }
    }

    impl ChartPlugin for RecordingPlugin {
        fn id(&self) -> &str {
            &self.id
        }

        fn on_event(&mut self, event: PluginEvent, _context: PluginContext) {
            self.events.borrow_mut().push(event);
        }
    }

    fn build_engine() -> ChartEngine<NullRenderer> {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 100.0)
            .with_price_domain(0.0, 10.0);
        ChartEngine::new(renderer, config).expect("engine init")
    }

    #[test]
    fn finalize_cycle_clears_pending_invalidation() {
        let mut engine = build_engine();
        engine.invalidate_full();
        assert!(engine.has_pending_invalidation());

        finalize_render_cycle(&mut engine);

        assert!(!engine.has_pending_invalidation());
    }

    #[test]
    fn finalize_cycle_emits_rendered_plugin_event() {
        let mut engine = build_engine();
        let events = Rc::new(RefCell::new(Vec::<PluginEvent>::new()));
        engine
            .register_plugin(Box::new(RecordingPlugin::new("recorder", events.clone())))
            .expect("register plugin");

        finalize_render_cycle(&mut engine);

        let last = events.borrow().last().copied().expect("rendered event");
        assert!(matches!(last, PluginEvent::Rendered));
    }
}
