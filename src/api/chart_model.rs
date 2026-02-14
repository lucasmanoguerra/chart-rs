use indexmap::IndexMap;

use crate::core::{
    DataPoint, OhlcBar, PaneCollection, PaneId, PriceScale, PriceScaleMode, TimeScale, Viewport,
};
use crate::interaction::InteractionState;

use super::CandlestickBarStyleOverride;

/// Core chart domain state modeled after Lightweight Charts `ChartModel`.
///
/// This struct intentionally groups mutable chart state (scales, series, panes,
/// viewport, interaction) so the engine can evolve toward model-centric
/// orchestration while preserving the current public API surface.
pub struct ChartModel {
    pub(super) viewport: Viewport,
    pub(super) time_scale: TimeScale,
    pub(super) price_scale: PriceScale,
    pub(super) price_scale_mode: PriceScaleMode,
    pub(super) interaction: InteractionState,
    pub(super) points: Vec<DataPoint>,
    pub(super) candles: Vec<OhlcBar>,
    pub(super) candle_style_overrides: Vec<Option<CandlestickBarStyleOverride>>,
    pub(super) points_pane_id: PaneId,
    pub(super) candles_pane_id: PaneId,
    pub(super) series_metadata: IndexMap<String, String>,
    pub(super) pane_collection: PaneCollection,
}

pub struct ChartModelBootstrap {
    pub viewport: Viewport,
    pub time_scale: TimeScale,
    pub price_scale: PriceScale,
    pub price_scale_mode: PriceScaleMode,
    pub interaction: InteractionState,
    pub pane_collection: PaneCollection,
    pub points_pane_id: PaneId,
    pub candles_pane_id: PaneId,
}

impl ChartModel {
    #[must_use]
    pub fn new(bootstrap: ChartModelBootstrap) -> Self {
        Self {
            viewport: bootstrap.viewport,
            time_scale: bootstrap.time_scale,
            price_scale: bootstrap.price_scale,
            price_scale_mode: bootstrap.price_scale_mode,
            interaction: bootstrap.interaction,
            points: Vec::new(),
            candles: Vec::new(),
            candle_style_overrides: Vec::new(),
            points_pane_id: bootstrap.points_pane_id,
            candles_pane_id: bootstrap.candles_pane_id,
            series_metadata: IndexMap::new(),
            pane_collection: bootstrap.pane_collection,
        }
    }
}
