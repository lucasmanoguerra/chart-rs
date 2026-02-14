#[cfg(feature = "cairo-backend")]
use crate::error::ChartResult;
#[cfg(feature = "cairo-backend")]
use crate::render::{CairoContextRenderer, LayeredRenderFrame, Renderer};

#[cfg(feature = "cairo-backend")]
use super::ChartEngine;
#[cfg(feature = "cairo-backend")]
use super::pane_render_executor::PaneRenderExecutor;
#[cfg(feature = "cairo-backend")]
use super::render_partial_scheduler::PartialCairoRenderPlan;

#[cfg(feature = "cairo-backend")]
pub(super) fn render_partial_on_cairo_context<R: Renderer + CairoContextRenderer>(
    engine: &mut ChartEngine<R>,
    context: &cairo::Context,
    layered: &LayeredRenderFrame,
    plan: &PartialCairoRenderPlan,
) -> ChartResult<()> {
    let tasks = PaneRenderExecutor::collect_partial_tasks(engine, layered, plan);
    for task in tasks {
        engine.renderer.render_on_cairo_context_partial(
            context,
            &task.frame,
            task.clip_rect,
            task.clear_region,
        )?;
    }

    Ok(())
}

#[cfg(all(test, feature = "cairo-backend"))]
mod tests {
    use cairo::{Format, ImageSurface};

    use super::render_partial_on_cairo_context;
    use crate::api::{
        ChartEngine, ChartEngineConfig, InvalidationLevel, InvalidationMask, InvalidationTopic,
        InvalidationTopics,
    };
    use crate::core::Viewport;
    use crate::error::ChartResult;
    use crate::render::{
        CairoContextRenderer, CanvasLayerKind, Color, LayeredRenderFrame, PaneLayerStack,
        RenderFrame, Renderer, TextHAlign, TextPrimitive,
    };

    use super::super::render_partial_scheduler::PartialCairoRenderPlan;

    type PartialCall = (Option<(f64, f64, f64, f64)>, bool);

    #[derive(Default)]
    struct RecordingRenderer {
        partial_calls: Vec<PartialCall>,
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
            Ok(())
        }

        fn render_on_cairo_context_partial(
            &mut self,
            _context: &cairo::Context,
            _frame: &RenderFrame,
            clip_rect: Option<(f64, f64, f64, f64)>,
            clear: bool,
        ) -> ChartResult<()> {
            self.partial_calls.push((clip_rect, clear));
            Ok(())
        }
    }

    fn build_engine() -> ChartEngine<RecordingRenderer> {
        let renderer = RecordingRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 100.0)
            .with_price_domain(0.0, 10.0);
        ChartEngine::new(renderer, config).expect("engine init")
    }

    fn build_layered(engine: &ChartEngine<RecordingRenderer>) -> LayeredRenderFrame {
        let main = engine.main_pane_id();
        let aux = crate::core::PaneId::new(1);
        let mut layered = LayeredRenderFrame::from_stacks(
            engine.viewport(),
            vec![
                PaneLayerStack::canonical_for_pane(main),
                PaneLayerStack::canonical_for_pane(aux),
            ],
        )
        .with_pane_regions(&[(main, 0.0, 250.0), (aux, 250.0, 500.0)]);

        layered.push_text(
            main,
            CanvasLayerKind::Axis,
            TextPrimitive::new(
                "main-axis",
                790.0,
                20.0,
                11.0,
                Color::rgb(1.0, 1.0, 1.0),
                TextHAlign::Right,
            ),
        );
        layered.push_text(
            main,
            CanvasLayerKind::Crosshair,
            TextPrimitive::new(
                "main-cursor",
                200.0,
                100.0,
                11.0,
                Color::rgb(1.0, 1.0, 1.0),
                TextHAlign::Left,
            ),
        );
        layered.push_text(
            aux,
            CanvasLayerKind::Crosshair,
            TextPrimitive::new(
                "aux-cursor",
                200.0,
                320.0,
                11.0,
                Color::rgb(1.0, 1.0, 1.0),
                TextHAlign::Left,
            ),
        );

        layered
    }

    #[test]
    fn render_partial_pass_submits_plot_and_axis_tasks() {
        let mut engine = build_engine();
        let layered = build_layered(&engine);
        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Cursor,
            InvalidationTopics::from_topic(InvalidationTopic::Cursor),
        );
        let plan = PartialCairoRenderPlan::build(pending, &[], &layered).expect("plan");

        let surface = ImageSurface::create(Format::ARgb32, 800, 500).expect("surface");
        let context = cairo::Context::new(&surface).expect("context");
        render_partial_on_cairo_context(&mut engine, &context, &layered, &plan)
            .expect("partial pass");

        assert_eq!(engine.renderer.partial_calls.len(), 3);
        assert!(engine.renderer.partial_calls[0].1);
        assert!(engine.renderer.partial_calls[1].1);
        assert!(!engine.renderer.partial_calls[2].1);
        assert!(engine.renderer.partial_calls[2].0.is_none());
    }
}
