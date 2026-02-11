use crate::core::{DataPoint, PriceScale, TimeScale, Viewport};
use crate::error::{ChartError, ChartResult};
use crate::interaction::{InteractionMode, InteractionState};
use crate::render::{RenderFrame, Renderer};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartEngineConfig {
    pub viewport: Viewport,
    pub time_start: f64,
    pub time_end: f64,
    pub price_min: f64,
    pub price_max: f64,
}

impl ChartEngineConfig {
    #[must_use]
    pub fn new(viewport: Viewport, time_start: f64, time_end: f64) -> Self {
        Self {
            viewport,
            time_start,
            time_end,
            price_min: 0.0,
            price_max: 1.0,
        }
    }

    #[must_use]
    pub fn with_price_domain(mut self, price_min: f64, price_max: f64) -> Self {
        self.price_min = price_min;
        self.price_max = price_max;
        self
    }
}

pub struct ChartEngine<R: Renderer> {
    renderer: R,
    viewport: Viewport,
    time_scale: TimeScale,
    price_scale: PriceScale,
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

        let time_scale = TimeScale::new(config.time_start, config.time_end)?;
        let price_scale = PriceScale::new(config.price_min, config.price_max)?;

        Ok(Self {
            renderer,
            viewport: config.viewport,
            time_scale,
            price_scale,
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
        self.time_scale.time_to_pixel(x, self.viewport)
    }

    pub fn map_pixel_to_x(&self, pixel: f64) -> ChartResult<f64> {
        self.time_scale.pixel_to_time(pixel, self.viewport)
    }

    pub fn map_price_to_pixel(&self, price: f64) -> ChartResult<f64> {
        self.price_scale.price_to_pixel(price, self.viewport)
    }

    pub fn map_pixel_to_price(&self, pixel: f64) -> ChartResult<f64> {
        self.price_scale.pixel_to_price(pixel, self.viewport)
    }

    #[must_use]
    pub fn price_domain(&self) -> (f64, f64) {
        self.price_scale.domain()
    }

    pub fn autoscale_price_from_data(&mut self) -> ChartResult<()> {
        if self.points.is_empty() {
            return Ok(());
        }
        self.price_scale = PriceScale::from_data(&self.points)?;
        Ok(())
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
