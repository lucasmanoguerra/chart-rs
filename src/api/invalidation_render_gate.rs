use crate::error::ChartResult;
use crate::render::{RenderFrame, Renderer};

use super::ChartEngine;

pub(super) fn build_render_frame_if_invalidated<R: Renderer>(
    engine: &mut ChartEngine<R>,
) -> ChartResult<Option<RenderFrame>> {
    if !engine.has_pending_invalidation() {
        return Ok(None);
    }
    engine.build_render_frame().map(Some)
}

pub(super) fn render_if_invalidated<R: Renderer>(engine: &mut ChartEngine<R>) -> ChartResult<bool> {
    if !engine.has_pending_invalidation() {
        return Ok(false);
    }
    engine.render()?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{build_render_frame_if_invalidated, render_if_invalidated};
    use crate::api::{ChartEngine, ChartEngineConfig};
    use crate::core::{DataPoint, Viewport};
    use crate::render::NullRenderer;

    fn build_engine() -> ChartEngine<NullRenderer> {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 100.0)
            .with_price_domain(0.0, 10.0);
        ChartEngine::new(renderer, config).expect("engine init")
    }

    #[test]
    fn gate_build_returns_none_without_pending_invalidation() {
        let mut engine = build_engine();
        engine.clear_pending_invalidation();

        let frame = build_render_frame_if_invalidated(&mut engine).expect("gate build");
        assert!(frame.is_none());
    }

    #[test]
    fn gate_render_returns_false_without_pending_invalidation() {
        let mut engine = build_engine();
        engine.clear_pending_invalidation();

        let rendered = render_if_invalidated(&mut engine).expect("gate render");
        assert!(!rendered);
    }

    #[test]
    fn gate_render_returns_true_with_pending_invalidation() {
        let mut engine = build_engine();
        engine.clear_pending_invalidation();
        engine.set_data(vec![DataPoint::new(10.0, 1.0), DataPoint::new(20.0, 2.0)]);

        let rendered = render_if_invalidated(&mut engine).expect("gate render");
        assert!(rendered);
        assert!(!engine.has_pending_invalidation());
    }
}
