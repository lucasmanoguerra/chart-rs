use std::cell::RefCell;

use super::label_cache::{
    PriceLabelCache, PriceLabelFormatterFn, TimeLabelCache, TimeLabelFormatterFn,
};
use super::{
    CrosshairPriceLabelFormatterWithContextFn, CrosshairTimeLabelFormatterWithContextFn,
    RenderStyle,
};

/// Runtime presentation state grouped separately from core chart model/behavior.
pub(super) struct ChartPresentationState {
    pub(super) time_label_formatter: Option<TimeLabelFormatterFn>,
    pub(super) price_label_formatter: Option<PriceLabelFormatterFn>,
    pub(super) crosshair_time_label_formatter: Option<TimeLabelFormatterFn>,
    pub(super) crosshair_price_label_formatter: Option<PriceLabelFormatterFn>,
    pub(super) crosshair_time_label_formatter_with_context:
        Option<CrosshairTimeLabelFormatterWithContextFn>,
    pub(super) crosshair_price_label_formatter_with_context:
        Option<CrosshairPriceLabelFormatterWithContextFn>,
    pub(super) time_label_formatter_generation: u64,
    pub(super) price_label_formatter_generation: u64,
    pub(super) crosshair_time_label_formatter_generation: u64,
    pub(super) crosshair_price_label_formatter_generation: u64,
    pub(super) time_label_cache: RefCell<TimeLabelCache>,
    pub(super) price_label_cache: RefCell<PriceLabelCache>,
    pub(super) crosshair_time_label_cache: RefCell<TimeLabelCache>,
    pub(super) crosshair_price_label_cache: RefCell<PriceLabelCache>,
    pub(super) render_style: RenderStyle,
}

impl Default for ChartPresentationState {
    fn default() -> Self {
        Self {
            time_label_formatter: None,
            price_label_formatter: None,
            crosshair_time_label_formatter: None,
            crosshair_price_label_formatter: None,
            crosshair_time_label_formatter_with_context: None,
            crosshair_price_label_formatter_with_context: None,
            time_label_formatter_generation: 0,
            price_label_formatter_generation: 0,
            crosshair_time_label_formatter_generation: 0,
            crosshair_price_label_formatter_generation: 0,
            time_label_cache: RefCell::new(TimeLabelCache::default()),
            price_label_cache: RefCell::new(PriceLabelCache::default()),
            crosshair_time_label_cache: RefCell::new(TimeLabelCache::default()),
            crosshair_price_label_cache: RefCell::new(PriceLabelCache::default()),
            render_style: RenderStyle::default(),
        }
    }
}
