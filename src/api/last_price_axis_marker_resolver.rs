use crate::error::ChartResult;
use crate::render::Renderer;

use super::last_price_axis_scene_builder::LastPriceMarker;
use super::{ChartEngine, RenderStyle};

impl<R: Renderer> ChartEngine<R> {
    pub(super) fn resolve_last_price_marker_for_axis(
        &self,
        style: RenderStyle,
        visible_start: f64,
        visible_end: f64,
        plot_bottom: f64,
    ) -> ChartResult<Option<LastPriceMarker>> {
        let Some((last_price, previous_price)) = self.resolve_latest_and_previous_price_values(
            style.last_price_source_mode,
            visible_start,
            visible_end,
        ) else {
            return Ok(None);
        };

        let py = self
            .core
            .model
            .price_scale
            .price_to_pixel(last_price, self.core.model.viewport)?
            .clamp(0.0, plot_bottom);
        let (marker_line_color, marker_label_color) =
            self.resolve_last_price_marker_colors(last_price, previous_price);

        Ok(Some(LastPriceMarker {
            last_price,
            py,
            marker_line_color,
            marker_label_color,
        }))
    }
}
