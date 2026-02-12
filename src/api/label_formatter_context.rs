use std::sync::Arc;

/// Source mode used to derive the current crosshair axis-label value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrosshairLabelSourceMode {
    /// Label value comes from a snapped data sample in magnet mode.
    SnappedData,
    /// Label value comes from raw pointer projection in normal mode.
    PointerProjected,
}

/// Context passed to crosshair time-axis label formatter overrides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrosshairTimeLabelFormatterContext {
    /// Absolute visible time span for the current frame.
    pub visible_span_abs: f64,
    /// Source mode used for crosshair label value resolution.
    pub source_mode: CrosshairLabelSourceMode,
}

/// Context passed to crosshair price-axis label formatter overrides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrosshairPriceLabelFormatterContext {
    /// Absolute visible time span for the current frame.
    pub visible_span_abs: f64,
    /// Source mode used for crosshair label value resolution.
    pub source_mode: CrosshairLabelSourceMode,
}

pub type CrosshairTimeLabelFormatterWithContextFn =
    Arc<dyn Fn(f64, CrosshairTimeLabelFormatterContext) -> String + Send + Sync + 'static>;
pub type CrosshairPriceLabelFormatterWithContextFn =
    Arc<dyn Fn(f64, CrosshairPriceLabelFormatterContext) -> String + Send + Sync + 'static>;
