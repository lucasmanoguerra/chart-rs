use crate::render::{LayeredRenderFrame, Renderer};

use super::ChartEngine;
use super::render_partial_scheduler::PartialCairoRenderPlan;
use super::render_partial_task::PanePartialRenderTask;
use super::render_partial_task_collectors::{collect_main_axis_task, collect_plot_tasks};

pub(super) struct PaneRenderExecutor;

impl PaneRenderExecutor {
    #[must_use]
    pub(super) fn collect_partial_tasks<R: Renderer>(
        engine: &ChartEngine<R>,
        layered: &LayeredRenderFrame,
        plan: &PartialCairoRenderPlan,
    ) -> Vec<PanePartialRenderTask> {
        let viewport_width = f64::from(engine.core.model.viewport.width);
        let mut tasks = collect_plot_tasks(layered, plan, viewport_width);
        if let Some(axis_task) = collect_main_axis_task(engine.main_pane_id(), layered) {
            tasks.push(axis_task);
        }

        tasks
    }
}

#[cfg(test)]
mod tests {
    use super::PaneRenderExecutor;
    use crate::api::{
        ChartEngine, ChartEngineConfig, InvalidationLevel, InvalidationMask, InvalidationTopic,
        InvalidationTopics,
    };
    use crate::core::Viewport;
    use crate::render::{
        CanvasLayerKind, Color, LayeredRenderFrame, NullRenderer, PaneLayerStack, TextHAlign,
        TextPrimitive,
    };

    use super::super::render_partial_scheduler::PartialCairoRenderPlan;

    fn build_engine() -> ChartEngine<NullRenderer> {
        let renderer = NullRenderer::default();
        let config = ChartEngineConfig::new(Viewport::new(800, 500), 0.0, 100.0)
            .with_price_domain(0.0, 10.0);
        ChartEngine::new(renderer, config).expect("engine init")
    }

    fn build_layered(engine: &ChartEngine<NullRenderer>) -> LayeredRenderFrame {
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
    fn collect_partial_tasks_builds_plot_tasks_and_axis_task() {
        let engine = build_engine();
        let layered = build_layered(&engine);
        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Cursor,
            InvalidationTopics::from_topic(InvalidationTopic::Cursor),
        );
        let plan = PartialCairoRenderPlan::build(pending, &[], &layered).expect("plan");

        let tasks = PaneRenderExecutor::collect_partial_tasks(&engine, &layered, &plan);
        assert_eq!(tasks.len(), 3);

        // Pane plot tasks clear clipped regions.
        assert!(tasks[0].clear_region);
        assert!(tasks[0].clip_rect.is_some());
        assert!(tasks[1].clear_region);
        assert!(tasks[1].clip_rect.is_some());

        // Main axis task is un-clipped and never clears.
        assert!(!tasks[2].clear_region);
        assert!(tasks[2].clip_rect.is_none());
    }

    #[test]
    fn collect_partial_tasks_respects_api_pane_targets() {
        let engine = build_engine();
        let layered = build_layered(&engine);
        let aux = crate::core::PaneId::new(1);
        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::Series),
        );
        let plan = PartialCairoRenderPlan::build(pending, &[aux], &layered).expect("plan");

        let tasks = PaneRenderExecutor::collect_partial_tasks(&engine, &layered, &plan);
        // Aux plot task + main axis task.
        assert_eq!(tasks.len(), 2);
        assert!(tasks[0].clear_region);
        assert!(!tasks[1].clear_region);
    }

    #[test]
    fn collect_partial_tasks_respects_multiple_lwc_explicit_panes() {
        let engine = build_engine();
        let main = engine.main_pane_id();
        let pane_a = crate::core::PaneId::new(1);
        let pane_b = crate::core::PaneId::new(2);
        let mut layered = LayeredRenderFrame::from_stacks(
            engine.viewport(),
            vec![
                PaneLayerStack::canonical_for_pane(main),
                PaneLayerStack::canonical_for_pane(pane_a),
                PaneLayerStack::canonical_for_pane(pane_b),
            ],
        )
        .with_pane_regions(&[
            (main, 0.0, 160.0),
            (pane_a, 160.0, 320.0),
            (pane_b, 320.0, 500.0),
        ]);

        for (pane_id, y) in [(main, 80.0), (pane_a, 240.0), (pane_b, 420.0)] {
            layered.push_text(
                pane_id,
                CanvasLayerKind::Crosshair,
                TextPrimitive::new(
                    "cursor",
                    200.0,
                    y,
                    11.0,
                    Color::rgb(1.0, 1.0, 1.0),
                    TextHAlign::Left,
                ),
            );
        }
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

        let legacy_pending = InvalidationMask::light();
        let lwc_pane_ids = vec![main, pane_a, pane_b];
        let mut lwc_pending =
            crate::lwc::model::InvalidateMask::new(crate::lwc::model::InvalidationLevel::Light);
        lwc_pending.invalidate_pane(
            1,
            crate::lwc::model::PaneInvalidation {
                level: crate::lwc::model::InvalidationLevel::Light,
                auto_scale: false,
            },
        );
        lwc_pending.invalidate_pane(
            2,
            crate::lwc::model::PaneInvalidation {
                level: crate::lwc::model::InvalidationLevel::Light,
                auto_scale: false,
            },
        );
        let plan = PartialCairoRenderPlan::build_from_masks(
            legacy_pending,
            &[],
            Some(&lwc_pending),
            &lwc_pane_ids,
            &layered,
        )
        .expect("plan");

        let tasks = PaneRenderExecutor::collect_partial_tasks(&engine, &layered, &plan);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks.iter().filter(|task| task.clear_region).count(), 2);
        assert_eq!(tasks.iter().filter(|task| !task.clear_region).count(), 1);
    }

    #[test]
    fn collect_partial_tasks_axis_only_plan_produces_axis_task_only() {
        let engine = build_engine();
        let layered = build_layered(&engine);
        let pending = InvalidationMask::with_level_and_topics(
            InvalidationLevel::Light,
            InvalidationTopics::from_topic(InvalidationTopic::Axis),
        );
        let plan = PartialCairoRenderPlan::build(pending, &[], &layered).expect("plan");

        let tasks = PaneRenderExecutor::collect_partial_tasks(&engine, &layered, &plan);
        assert_eq!(tasks.len(), 1);
        assert!(!tasks[0].clear_region);
        assert!(tasks[0].clip_rect.is_none());
    }
}
