#[cfg(feature = "cairo-backend")]
use crate::error::ChartResult;
#[cfg(feature = "cairo-backend")]
use crate::render::{CairoContextRenderer, Renderer};

#[cfg(feature = "cairo-backend")]
use super::ChartEngine;
#[cfg(feature = "cairo-backend")]
use super::render_cairo_execution_path_resolver::CairoRenderExecutionPath;
#[cfg(feature = "cairo-backend")]
use super::render_cairo_full_pass_executor::render_full_on_cairo_context;
#[cfg(feature = "cairo-backend")]
use super::render_cairo_partial_pass_executor::render_partial_on_cairo_context;
#[cfg(feature = "cairo-backend")]
use super::render_cycle_finalizer::finalize_render_cycle;

#[cfg(feature = "cairo-backend")]
pub(super) fn render_on_cairo_context<R: Renderer + CairoContextRenderer>(
    engine: &mut ChartEngine<R>,
    context: &cairo::Context,
) -> ChartResult<()> {
    match CairoRenderExecutionPath::resolve(engine)? {
        CairoRenderExecutionPath::Partial { layered, plan } => {
            render_partial_on_cairo_context(engine, context, &layered, &plan)?;
            finalize_render_cycle(engine);
            Ok(())
        }
        CairoRenderExecutionPath::Full => {
            render_full_on_cairo_context(engine, context)?;
            finalize_render_cycle(engine);
            Ok(())
        }
    }
}

#[cfg(all(test, feature = "cairo-backend"))]
mod tests {
    use cairo::{Format, ImageSurface};

    use super::render_on_cairo_context;
    use crate::api::{ChartEngine, ChartEngineConfig};
    use crate::core::Viewport;
    use crate::error::ChartResult;
    use crate::render::{CairoContextRenderer, RenderFrame, Renderer};

    #[derive(Default)]
    struct RecordingRenderer {
        full_calls: usize,
        partial_calls: usize,
    }

    impl Renderer for RecordingRenderer {
        fn render(&mut self, _frame: &RenderFrame) -> ChartResult<()> {
            Ok(())
        }
    }

    impl CairoContextRenderer for RecordingRenderer {
        fn render_on_cairo_context(
            &mut self,
            _context: &cairo::Context,
            _frame: &RenderFrame,
        ) -> ChartResult<()> {
            self.full_calls += 1;
            Ok(())
        }

        fn render_on_cairo_context_partial(
            &mut self,
            _context: &cairo::Context,
            _frame: &RenderFrame,
            _clip_rect: Option<(f64, f64, f64, f64)>,
            _clear: bool,
        ) -> ChartResult<()> {
            self.partial_calls += 1;
            Ok(())
        }
    }

    fn build_engine() -> ChartEngine<RecordingRenderer> {
        let renderer = RecordingRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 100.0)
            .with_price_domain(0.0, 10.0);
        ChartEngine::new(renderer, config).expect("engine init")
    }

    #[test]
    fn single_pane_routes_to_full_pass() {
        let mut engine = build_engine();
        engine.invalidate_full();

        let surface = ImageSurface::create(Format::ARgb32, 800, 500).expect("surface");
        let context = cairo::Context::new(&surface).expect("context");
        render_on_cairo_context(&mut engine, &context).expect("cairo render");

        assert_eq!(engine.renderer.full_calls, 1);
        assert_eq!(engine.renderer.partial_calls, 0);
        assert!(!engine.has_pending_invalidation());
    }

    #[test]
    fn multi_pane_cursor_invalidation_routes_to_partial_pass() {
        let mut engine = build_engine();
        let _ = engine.create_pane(1.0).expect("create pane");
        engine.clear_pending_invalidation();
        engine.invalidate_cursor();

        let surface = ImageSurface::create(Format::ARgb32, 800, 500).expect("surface");
        let context = cairo::Context::new(&surface).expect("context");
        render_on_cairo_context(&mut engine, &context).expect("cairo render");

        assert_eq!(engine.renderer.full_calls, 0);
        assert!(engine.renderer.partial_calls > 0);
        assert!(!engine.has_pending_invalidation());
    }
}
