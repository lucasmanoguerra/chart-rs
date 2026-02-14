use crate::core::{PaneId, Viewport};

use super::{
    CanvasLayerKind, LinePrimitive, PaneLayerStack, RectPrimitive, RenderFrame, TextPrimitive,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LayerPrimitives {
    pub kind: CanvasLayerKind,
    pub lines: Vec<LinePrimitive>,
    pub rects: Vec<RectPrimitive>,
    pub texts: Vec<TextPrimitive>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaneLayerFrame {
    pub pane_id: PaneId,
    pub plot_top: f64,
    pub plot_bottom: f64,
    pub layers: Vec<LayerPrimitives>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredRenderFrame {
    pub viewport: Viewport,
    pub panes: Vec<PaneLayerFrame>,
}

impl LayeredRenderFrame {
    #[must_use]
    pub fn from_stacks(viewport: Viewport, stacks: Vec<PaneLayerStack>) -> Self {
        let default_bottom = f64::from(viewport.height);
        let panes = stacks
            .into_iter()
            .map(|stack| {
                let layers = stack
                    .layers
                    .into_iter()
                    .map(|kind| LayerPrimitives {
                        kind,
                        lines: Vec::new(),
                        rects: Vec::new(),
                        texts: Vec::new(),
                    })
                    .collect();
                PaneLayerFrame {
                    pane_id: stack.pane_id,
                    plot_top: 0.0,
                    plot_bottom: default_bottom,
                    layers,
                }
            })
            .collect();
        Self { viewport, panes }
    }

    #[must_use]
    pub fn with_pane_regions(mut self, regions: &[(PaneId, f64, f64)]) -> Self {
        for pane in &mut self.panes {
            if let Some((_, top, bottom)) = regions.iter().find(|(id, _, _)| *id == pane.pane_id) {
                pane.plot_top = *top;
                pane.plot_bottom = *bottom;
            }
        }
        self
    }

    pub fn push_line(&mut self, pane_id: PaneId, kind: CanvasLayerKind, line: LinePrimitive) {
        if let Some(layer) = self.layer_mut(pane_id, kind) {
            layer.lines.push(line);
        }
    }

    pub fn push_rect(&mut self, pane_id: PaneId, kind: CanvasLayerKind, rect: RectPrimitive) {
        if let Some(layer) = self.layer_mut(pane_id, kind) {
            layer.rects.push(rect);
        }
    }

    pub fn push_text(&mut self, pane_id: PaneId, kind: CanvasLayerKind, text: TextPrimitive) {
        if let Some(layer) = self.layer_mut(pane_id, kind) {
            layer.texts.push(text);
        }
    }

    #[must_use]
    pub fn flatten(&self) -> RenderFrame {
        let mut frame = RenderFrame::new(self.viewport);
        for pane in &self.panes {
            for layer in &pane.layers {
                frame.lines.extend(layer.lines.iter().copied());
                frame.rects.extend(layer.rects.iter().copied());
                frame.texts.extend(layer.texts.iter().cloned());
            }
        }
        frame
    }

    #[must_use]
    pub fn flatten_pane(&self, pane_id: PaneId) -> Option<RenderFrame> {
        let pane = self.panes.iter().find(|pane| pane.pane_id == pane_id)?;
        let mut frame = RenderFrame::new(self.viewport);
        for layer in &pane.layers {
            frame.lines.extend(layer.lines.iter().copied());
            frame.rects.extend(layer.rects.iter().copied());
            frame.texts.extend(layer.texts.iter().cloned());
        }
        Some(frame)
    }

    #[must_use]
    pub fn flatten_pane_layers(
        &self,
        pane_id: PaneId,
        include_layers: &[CanvasLayerKind],
    ) -> Option<RenderFrame> {
        let pane = self.panes.iter().find(|pane| pane.pane_id == pane_id)?;
        let mut frame = RenderFrame::new(self.viewport);
        for layer in &pane.layers {
            if !include_layers.contains(&layer.kind) {
                continue;
            }
            frame.lines.extend(layer.lines.iter().copied());
            frame.rects.extend(layer.rects.iter().copied());
            frame.texts.extend(layer.texts.iter().cloned());
        }
        Some(frame)
    }

    pub fn remap_plot_layers_to_pane_region(
        &mut self,
        pane_id: PaneId,
        source_plot_top: f64,
        source_plot_bottom: f64,
    ) {
        let Some(pane) = self.panes.iter_mut().find(|pane| pane.pane_id == pane_id) else {
            return;
        };
        let source_span = source_plot_bottom - source_plot_top;
        if !source_span.is_finite() || source_span <= 0.0 {
            return;
        }
        let target_top = pane.plot_top;
        let target_bottom = pane.plot_bottom;
        let target_span = target_bottom - target_top;
        if !target_span.is_finite() || target_span <= 0.0 {
            return;
        }

        for layer in &mut pane.layers {
            if matches!(
                layer.kind,
                CanvasLayerKind::Axis | CanvasLayerKind::Background
            ) {
                continue;
            }
            for line in &mut layer.lines {
                line.y1 = remap_scalar(
                    line.y1,
                    source_plot_top,
                    source_span,
                    target_top,
                    target_span,
                );
                line.y2 = remap_scalar(
                    line.y2,
                    source_plot_top,
                    source_span,
                    target_top,
                    target_span,
                );
            }
            for rect in &mut layer.rects {
                let top = remap_scalar(
                    rect.y,
                    source_plot_top,
                    source_span,
                    target_top,
                    target_span,
                );
                let bottom = remap_scalar(
                    rect.y + rect.height,
                    source_plot_top,
                    source_span,
                    target_top,
                    target_span,
                );
                rect.y = top.min(bottom);
                rect.height = (bottom - top).abs();
            }
            for text in &mut layer.texts {
                text.y = remap_scalar(
                    text.y,
                    source_plot_top,
                    source_span,
                    target_top,
                    target_span,
                );
            }
        }
    }

    fn layer_mut(
        &mut self,
        pane_id: PaneId,
        kind: CanvasLayerKind,
    ) -> Option<&mut LayerPrimitives> {
        let pane = self.panes.iter_mut().find(|pane| pane.pane_id == pane_id)?;
        pane.layers.iter_mut().find(|layer| layer.kind == kind)
    }
}

fn remap_scalar(
    value: f64,
    source_top: f64,
    source_span: f64,
    target_top: f64,
    target_span: f64,
) -> f64 {
    target_top + ((value - source_top) / source_span) * target_span
}

#[cfg(test)]
mod tests {
    use super::LayeredRenderFrame;
    use crate::core::{PaneId, Viewport};
    use crate::render::{
        CanvasLayerKind, Color, LinePrimitive, PaneLayerStack, TextHAlign, TextPrimitive,
    };

    #[test]
    fn layered_render_frame_flattens_in_pane_layer_order() {
        let pane_id = PaneId::new(0);
        let mut layered = LayeredRenderFrame::from_stacks(
            Viewport::new(100, 50),
            vec![PaneLayerStack::canonical_for_pane(pane_id)],
        );

        layered.push_line(
            pane_id,
            CanvasLayerKind::Grid,
            LinePrimitive::new(0.0, 1.0, 5.0, 1.0, 1.0, Color::rgb(0.2, 0.2, 0.2)),
        );
        layered.push_line(
            pane_id,
            CanvasLayerKind::Series,
            LinePrimitive::new(0.0, 2.0, 5.0, 3.0, 1.0, Color::rgb(0.8, 0.2, 0.2)),
        );
        layered.push_text(
            pane_id,
            CanvasLayerKind::Axis,
            TextPrimitive::new(
                "x",
                2.0,
                4.0,
                10.0,
                Color::rgb(1.0, 1.0, 1.0),
                TextHAlign::Right,
            ),
        );

        let flattened = layered.flatten();
        assert_eq!(flattened.lines.len(), 2);
        assert_eq!(flattened.texts.len(), 1);
        // Grid layer comes before Series in canonical stack.
        assert_eq!(flattened.lines[0].y1, 1.0);
        assert_eq!(flattened.lines[1].y1, 2.0);
    }
}
