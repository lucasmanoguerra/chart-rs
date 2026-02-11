use smallvec::SmallVec;

use crate::core::{
    CandleGeometry, DataPoint, OhlcBar, PriceScale, PriceScaleTuning, TimeScale, TimeScaleTuning,
    Viewport, project_candles,
};
use crate::error::{ChartError, ChartResult};
use crate::interaction::{CrosshairState, InteractionMode, InteractionState};
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
    candles: Vec<OhlcBar>,
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
            candles: Vec::new(),
        })
    }

    pub fn set_data(&mut self, points: Vec<DataPoint>) {
        self.points = points;
    }

    pub fn append_point(&mut self, point: DataPoint) {
        self.points.push(point);
    }

    pub fn set_candles(&mut self, candles: Vec<OhlcBar>) {
        self.candles = candles;
    }

    pub fn append_candle(&mut self, candle: OhlcBar) {
        self.candles.push(candle);
    }

    #[must_use]
    pub fn points(&self) -> &[DataPoint] {
        &self.points
    }

    #[must_use]
    pub fn candles(&self) -> &[OhlcBar] {
        &self.candles
    }

    #[must_use]
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    #[must_use]
    pub fn interaction_mode(&self) -> InteractionMode {
        self.interaction.mode()
    }

    #[must_use]
    pub fn crosshair_state(&self) -> CrosshairState {
        self.interaction.crosshair()
    }

    pub fn pointer_move(&mut self, x: f64, y: f64) {
        self.interaction.on_pointer_move(x, y);
        self.interaction.set_crosshair_snap(self.snap_at_x(x));
    }

    pub fn pointer_leave(&mut self) {
        self.interaction.on_pointer_leave();
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

    #[must_use]
    pub fn time_visible_range(&self) -> (f64, f64) {
        self.time_scale.visible_range()
    }

    #[must_use]
    pub fn time_full_range(&self) -> (f64, f64) {
        self.time_scale.full_range()
    }

    pub fn set_time_visible_range(&mut self, start: f64, end: f64) -> ChartResult<()> {
        self.time_scale.set_visible_range(start, end)
    }

    pub fn reset_time_visible_range(&mut self) {
        self.time_scale.reset_visible_range_to_full();
    }

    pub fn fit_time_to_data(&mut self, tuning: TimeScaleTuning) -> ChartResult<()> {
        if self.points.is_empty() && self.candles.is_empty() {
            return Ok(());
        }

        self.time_scale
            .fit_to_mixed_data(&self.points, &self.candles, tuning)
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
        self.autoscale_price_from_data_tuned(PriceScaleTuning::default())
    }

    pub fn autoscale_price_from_data_tuned(&mut self, tuning: PriceScaleTuning) -> ChartResult<()> {
        if self.points.is_empty() {
            return Ok(());
        }
        self.price_scale = PriceScale::from_data_tuned(&self.points, tuning)?;
        Ok(())
    }

    pub fn autoscale_price_from_candles(&mut self) -> ChartResult<()> {
        self.autoscale_price_from_candles_tuned(PriceScaleTuning::default())
    }

    pub fn autoscale_price_from_candles_tuned(
        &mut self,
        tuning: PriceScaleTuning,
    ) -> ChartResult<()> {
        if self.candles.is_empty() {
            return Ok(());
        }
        self.price_scale = PriceScale::from_ohlc_tuned(&self.candles, tuning)?;
        Ok(())
    }

    pub fn project_candles(&self, body_width_px: f64) -> ChartResult<Vec<CandleGeometry>> {
        project_candles(
            &self.candles,
            self.time_scale,
            self.price_scale,
            self.viewport,
            body_width_px,
        )
    }

    pub fn render(&mut self) -> ChartResult<()> {
        let frame = RenderFrame::new(self.viewport, self.points.clone());
        self.renderer.render(&frame)
    }

    #[must_use]
    pub fn into_renderer(self) -> R {
        self.renderer
    }

    fn snap_at_x(&self, pointer_x: f64) -> Option<(f64, f64)> {
        let mut candidates: SmallVec<[(f64, f64, f64); 2]> = SmallVec::new();
        if let Some(snap) = self.nearest_data_snap(pointer_x) {
            candidates.push(snap);
        }
        if let Some(snap) = self.nearest_candle_snap(pointer_x) {
            candidates.push(snap);
        }

        candidates
            .into_iter()
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, sx, sy)| (sx, sy))
    }

    fn nearest_data_snap(&self, pointer_x: f64) -> Option<(f64, f64, f64)> {
        let mut best: Option<(f64, f64, f64)> = None;
        for point in &self.points {
            let x_px = match self.time_scale.time_to_pixel(point.x, self.viewport) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let y_px = match self.price_scale.price_to_pixel(point.y, self.viewport) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let dist = (x_px - pointer_x).abs();
            match best {
                Some((current, _, _)) if current <= dist => {}
                _ => best = Some((dist, x_px, y_px)),
            }
        }
        best
    }

    fn nearest_candle_snap(&self, pointer_x: f64) -> Option<(f64, f64, f64)> {
        let mut best: Option<(f64, f64, f64)> = None;
        for candle in &self.candles {
            let x_px = match self.time_scale.time_to_pixel(candle.time, self.viewport) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let y_px = match self.price_scale.price_to_pixel(candle.close, self.viewport) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let dist = (x_px - pointer_x).abs();
            match best {
                Some((current, _, _)) if current <= dist => {}
                _ => best = Some((dist, x_px, y_px)),
            }
        }
        best
    }
}
