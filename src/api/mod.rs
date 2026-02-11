use crate::core::{DataPoint, LinearScale, Viewport};
use crate::error::{ChartError, ChartResult};
use crate::interaction::{InteractionMode, InteractionState};
use crate::render::{RenderFrame, Renderer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartEngineConfig {
    pub viewport: Viewport,
    pub domain_start: f64,
    pub domain_end: f64,
}

impl ChartEngineConfig {
    #[must_use]
    pub fn new(viewport: Viewport, domain_start: f64, domain_end: f64) -> Self {
        Self {
            viewport,
            domain_start,
            domain_end,
        }
    }
}

pub struct ChartEngine<R: Renderer> {
    renderer: R,
    viewport: Viewport,
    scale: LinearScale,
    interaction: InteractionState,
    points: Vec<DataPoint>,
}

impl<R: Renderer> ChartEngine<R> {
    pub fn new(renderer: R, config: ChartEngineConfig) -> ChartResult<Self> {
        if !config.viewport.is_valid() {
            return Err(ChartError::InvalidViewport {
                width: config.viewport.width,
                height: config.viewport.height,
            });
        }

        let scale = LinearScale::new(config.domain_start, config.domain_end)?;

        Ok(Self {
            renderer,
            viewport: config.viewport,
            scale,
            interaction: InteractionState::default(),
            points: Vec::new(),
        })
    }

    pub fn set_data(&mut self, points: Vec<DataPoint>) {
        self.points = points;
    }

    pub fn append_point(&mut self, point: DataPoint) {
        self.points.push(point);
    }

    #[must_use]
    pub fn points(&self) -> &[DataPoint] {
        &self.points
    }

    #[must_use]
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    #[must_use]
    pub fn interaction_mode(&self) -> InteractionMode {
        self.interaction.mode()
    }

    pub fn pointer_move(&mut self, x: f64, y: f64) {
        self.interaction.on_pointer_move(x, y);
    }

    pub fn pan_start(&mut self) {
        self.interaction.on_pan_start();
    }

    pub fn pan_end(&mut self) {
        self.interaction.on_pan_end();
    }

    pub fn map_x_to_pixel(&self, x: f64) -> ChartResult<f64> {
        self.scale.domain_to_pixel(x, self.viewport)
    }

    pub fn map_pixel_to_x(&self, pixel: f64) -> ChartResult<f64> {
        self.scale.pixel_to_domain(pixel, self.viewport)
    }

    pub fn render(&mut self) -> ChartResult<()> {
        let frame = RenderFrame::new(self.viewport, self.points.clone());
        self.renderer.render(&frame)
    }

    #[must_use]
    pub fn into_renderer(self) -> R {
        self.renderer
    }
}
