use crate::error::ChartResult;
use crate::render::Renderer;

use super::{CandlestickStyleBehavior, ChartEngine};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn candlestick_style_behavior(&self) -> CandlestickStyleBehavior {
        let style = self.render_style();
        let wick_color = (style.candlestick_wick_up_color == style.candlestick_wick_down_color)
            .then_some(style.candlestick_wick_up_color);
        let border_color = (style.candlestick_border_up_color
            == style.candlestick_border_down_color)
            .then_some(style.candlestick_border_up_color);
        CandlestickStyleBehavior {
            up_color: style.candlestick_up_color,
            down_color: style.candlestick_down_color,
            wick_color,
            wick_up_color: style.candlestick_wick_up_color,
            wick_down_color: style.candlestick_wick_down_color,
            border_color,
            border_up_color: style.candlestick_border_up_color,
            border_down_color: style.candlestick_border_down_color,
            body_mode: style.candlestick_body_mode,
            wick_width_px: style.candlestick_wick_width_px,
            border_width_px: style.candlestick_border_width_px,
            show_wicks: style.show_candlestick_wicks,
            show_borders: style.show_candlestick_borders,
        }
    }

    pub fn set_candlestick_style_behavior(
        &mut self,
        behavior: CandlestickStyleBehavior,
    ) -> ChartResult<()> {
        let mut style = self.render_style();
        style.candlestick_up_color = behavior.up_color;
        style.candlestick_down_color = behavior.down_color;
        style.candlestick_wick_up_color = behavior.wick_up_color;
        style.candlestick_wick_down_color = behavior.wick_down_color;
        if let Some(shared_wick_color) = behavior.wick_color {
            style.candlestick_wick_up_color = shared_wick_color;
            style.candlestick_wick_down_color = shared_wick_color;
        }
        style.candlestick_border_up_color = behavior.border_up_color;
        style.candlestick_border_down_color = behavior.border_down_color;
        if let Some(shared_border_color) = behavior.border_color {
            style.candlestick_border_up_color = shared_border_color;
            style.candlestick_border_down_color = shared_border_color;
        }
        style.candlestick_body_mode = behavior.body_mode;
        style.candlestick_wick_width_px = behavior.wick_width_px;
        style.candlestick_border_width_px = behavior.border_width_px;
        style.show_candlestick_wicks = behavior.show_wicks;
        style.show_candlestick_borders = behavior.show_borders;
        self.set_render_style(style)
    }
}
