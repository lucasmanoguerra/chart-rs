use serde::{Deserialize, Serialize};

use crate::core::PaneId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanvasLayerKind {
    Background,
    Grid,
    Series,
    Overlay,
    Crosshair,
    Axis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneLayerStack {
    pub pane_id: PaneId,
    pub layers: Vec<CanvasLayerKind>,
}

impl PaneLayerStack {
    #[must_use]
    pub fn canonical_for_pane(pane_id: PaneId) -> Self {
        Self {
            pane_id,
            layers: vec![
                CanvasLayerKind::Background,
                CanvasLayerKind::Grid,
                CanvasLayerKind::Series,
                CanvasLayerKind::Overlay,
                CanvasLayerKind::Crosshair,
                CanvasLayerKind::Axis,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CanvasLayerKind, PaneLayerStack};
    use crate::core::PaneId;

    #[test]
    fn pane_layer_stack_uses_canonical_lwc_order() {
        let stack = PaneLayerStack::canonical_for_pane(PaneId::new(7));
        assert_eq!(
            stack.layers,
            vec![
                CanvasLayerKind::Background,
                CanvasLayerKind::Grid,
                CanvasLayerKind::Series,
                CanvasLayerKind::Overlay,
                CanvasLayerKind::Crosshair,
                CanvasLayerKind::Axis,
            ]
        );
    }
}
