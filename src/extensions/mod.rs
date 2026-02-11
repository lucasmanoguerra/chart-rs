//! Optional feature modules live here.
//!
//! Keep extensions feature-gated and avoid coupling them into core paths.

pub mod markers;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionStatus {
    Planned,
    Experimental,
    Stable,
}

pub use markers::{
    MarkerLabelGeometry, MarkerPlacementConfig, MarkerPosition, MarkerSide, PlacedMarker,
    SeriesMarker, place_markers_on_candles,
};
