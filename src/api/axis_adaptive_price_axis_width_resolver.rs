use crate::error::ChartResult;
use crate::render::Renderer;

use super::layout_helpers::AxisLayout;
use super::{ChartEngine, RenderStyle};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn resolve_adaptive_price_axis_width(
        &self,
        style: RenderStyle,
        axis_layout: AxisLayout,
        visible_start: f64,
        visible_end: f64,
    ) -> ChartResult<f64> {
        self.estimate_required_price_axis_width(
            style,
            axis_layout.plot_bottom,
            visible_start,
            visible_end,
        )
    }
}
