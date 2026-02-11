use crate::core::{DataPoint, Viewport};
use crate::error::ChartResult;

#[derive(Debug, Clone, PartialEq)]
pub struct RenderFrame {
    pub viewport: Viewport,
    pub points: Vec<DataPoint>,
}

impl RenderFrame {
    #[must_use]
    pub fn new(viewport: Viewport, points: Vec<DataPoint>) -> Self {
        Self { viewport, points }
    }
}

pub trait Renderer {
    fn render(&mut self, frame: &RenderFrame) -> ChartResult<()>;
}

#[derive(Debug, Default)]
pub struct NullRenderer {
    pub last_point_count: usize,
}

impl Renderer for NullRenderer {
    fn render(&mut self, frame: &RenderFrame) -> ChartResult<()> {
        self.last_point_count = frame.points.len();
        Ok(())
    }
}

#[cfg(feature = "cairo-backend")]
pub mod cairo_backend {
    use cairo;
    use pango as _;
    use pangocairo as _;

    use crate::error::ChartResult;
    use crate::render::{RenderFrame, Renderer};

    #[derive(Debug)]
    pub struct CairoRenderer {
        _surface: cairo::ImageSurface,
    }

    impl CairoRenderer {
        pub fn new(width: i32, height: i32) -> ChartResult<Self> {
            let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)
                .map_err(|e| crate::error::ChartError::InvalidData(e.to_string()))?;
            Ok(Self { _surface: surface })
        }

        #[must_use]
        pub fn backend_name(&self) -> &'static str {
            "cairo+pango+pangocairo"
        }
    }

    impl Renderer for CairoRenderer {
        fn render(&mut self, _frame: &RenderFrame) -> ChartResult<()> {
            Ok(())
        }
    }
}
