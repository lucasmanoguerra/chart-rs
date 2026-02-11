use serde::{Deserialize, Serialize};

use crate::core::Viewport;
use crate::interaction::{CrosshairState, InteractionMode};

/// Read-only state snapshot passed to plugin hooks.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PluginContext {
    pub viewport: Viewport,
    pub time_visible_range: (f64, f64),
    pub price_domain: (f64, f64),
    pub points_len: usize,
    pub candles_len: usize,
    pub interaction_mode: InteractionMode,
    pub crosshair: CrosshairState,
}

/// Event stream exposed to plugins.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PluginEvent {
    DataUpdated { points_len: usize },
    CandlesUpdated { candles_len: usize },
    PointerMoved { x: f64, y: f64 },
    PointerLeft,
    VisibleRangeChanged { start: f64, end: f64 },
    PanStarted,
    PanEnded,
    Rendered,
}

/// Extension hook interface for bounded custom logic.
///
/// Plugins can observe events and read engine context without mutating core
/// internals directly.
pub trait ChartPlugin {
    fn id(&self) -> &str;
    fn on_event(&mut self, event: PluginEvent, context: PluginContext);
}
