mod frame;
mod layer_stack;
mod layered_frame;
mod null_renderer;
mod primitives;

pub use frame::RenderFrame;
pub use layer_stack::{CanvasLayerKind, PaneLayerStack};
pub use layered_frame::{LayerPrimitives, LayeredRenderFrame, PaneLayerFrame};
pub use null_renderer::NullRenderer;
pub use primitives::{
    Color, LinePrimitive, LineStrokeStyle, RectPrimitive, TextHAlign, TextPrimitive,
};

use crate::error::ChartResult;

/// Contract implemented by any rendering backend.
///
/// Backends receive a fully materialized, deterministic `RenderFrame` so
/// drawing code remains isolated from chart domain and interaction logic.
pub trait Renderer {
    fn render(&mut self, frame: &RenderFrame) -> ChartResult<()>;
}

#[cfg(feature = "cairo-backend")]
mod cairo_backend;
#[cfg(feature = "cairo-backend")]
pub use cairo_backend::{CairoContextRenderer, CairoRenderStats, CairoRenderer};
