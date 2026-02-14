use crate::error::ChartResult;
use crate::render::Renderer;

use super::ChartEngine;

pub(super) fn render_full_pass<R: Renderer>(engine: &mut ChartEngine<R>) -> ChartResult<()> {
    let frame = engine.build_render_frame()?;
    engine.renderer.render(&frame)
}

#[cfg(test)]
mod tests {
    use super::render_full_pass;
    use crate::api::{ChartEngine, ChartEngineConfig};
    use crate::core::Viewport;
    use crate::error::ChartResult;
    use crate::render::{RenderFrame, Renderer};

    #[derive(Default)]
    struct RecordingRenderer {
        calls: usize,
    }

    impl Renderer for RecordingRenderer {
        fn render(&mut self, _frame: &RenderFrame) -> ChartResult<()> {
            self.calls += 1;
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
    fn render_full_pass_submits_single_render_call() {
        let mut engine = build_engine();
        render_full_pass(&mut engine).expect("full pass");
        assert_eq!(engine.renderer.calls, 1);
    }
}
