use crate::error::ChartResult;
use crate::render::Renderer;

use super::{ChartEngine, LastPriceBehavior};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn last_price_behavior(&self) -> LastPriceBehavior {
        let style = self.render_style();
        LastPriceBehavior {
            show_line: style.show_last_price_line,
            show_label: style.show_last_price_label,
            use_trend_color: style.last_price_use_trend_color,
            source_mode: style.last_price_source_mode,
        }
    }

    pub fn set_last_price_behavior(&mut self, behavior: LastPriceBehavior) -> ChartResult<()> {
        let mut style = self.render_style();
        style.show_last_price_line = behavior.show_line;
        style.show_last_price_label = behavior.show_label;
        style.last_price_use_trend_color = behavior.use_trend_color;
        style.last_price_source_mode = behavior.source_mode;
        self.set_render_style(style)
    }
}
