use crate::core::{PaneId, PriceScale};
use crate::error::ChartResult;
use crate::render::{LayeredRenderFrame, RenderFrame, Renderer};

use super::line_series_render_frame_builder::LineSeriesRenderContext;
use super::{ChartEngine, RenderStyle};

#[derive(Debug, Clone, Copy)]
pub(super) struct PaneSeriesRenderTarget {
    pub pane_id: PaneId,
    pub price_scale: PriceScale,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SeriesSceneTargets {
    pub points: PaneSeriesRenderTarget,
    pub candles: PaneSeriesRenderTarget,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SeriesSceneRenderContext {
    pub main_pane_id: PaneId,
    pub visible_start: f64,
    pub visible_end: f64,
    pub plot_right: f64,
    pub style: RenderStyle,
}

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn resolve_series_scene_targets(
        &self,
        ctx: SeriesSceneRenderContext,
    ) -> SeriesSceneTargets {
        let points = self.resolve_pane_series_render_target(
            self.core.model.points_pane_id,
            ctx.main_pane_id,
            ctx.visible_start,
            ctx.visible_end,
        );
        let candles = self.resolve_pane_series_render_target(
            self.core.model.candles_pane_id,
            ctx.main_pane_id,
            ctx.visible_start,
            ctx.visible_end,
        );

        SeriesSceneTargets { points, candles }
    }

    pub(super) fn append_series_scene_primitives(
        &self,
        frame: &mut RenderFrame,
        layered: &mut LayeredRenderFrame,
        ctx: SeriesSceneRenderContext,
    ) -> ChartResult<()> {
        let targets = self.resolve_series_scene_targets(ctx);

        self.append_line_series_primitives(
            frame,
            layered,
            LineSeriesRenderContext {
                pane_id: targets.points.pane_id,
                price_scale: targets.points.price_scale,
                visible_start: ctx.visible_start,
                visible_end: ctx.visible_end,
                line_color: ctx.style.series_line_color,
            },
        )?;

        self.append_candlestick_series_primitives(
            frame,
            layered,
            (targets.candles.pane_id, targets.candles.price_scale),
            (ctx.visible_start, ctx.visible_end),
            ctx.plot_right,
            ctx.style,
        )?;

        Ok(())
    }

    fn resolve_pane_series_render_target(
        &self,
        preferred_pane_id: PaneId,
        fallback_main_pane_id: PaneId,
        visible_start: f64,
        visible_end: f64,
    ) -> PaneSeriesRenderTarget {
        let pane_id = if self.core.model.pane_collection.contains(preferred_pane_id) {
            preferred_pane_id
        } else {
            fallback_main_pane_id
        };
        let price_scale =
            self.resolve_render_price_scale_for_pane(pane_id, visible_start, visible_end);
        PaneSeriesRenderTarget {
            pane_id,
            price_scale,
        }
    }
}
