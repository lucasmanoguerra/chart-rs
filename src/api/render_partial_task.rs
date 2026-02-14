use crate::render::RenderFrame;

pub(super) struct PanePartialRenderTask {
    pub(super) frame: RenderFrame,
    pub(super) clip_rect: Option<(f64, f64, f64, f64)>,
    pub(super) clear_region: bool,
}

impl PanePartialRenderTask {
    #[must_use]
    pub(super) fn for_plot(frame: RenderFrame, clip_rect: (f64, f64, f64, f64)) -> Self {
        Self {
            frame,
            clip_rect: Some(clip_rect),
            clear_region: true,
        }
    }

    #[must_use]
    pub(super) fn for_axis(frame: RenderFrame) -> Self {
        Self {
            frame,
            clip_rect: None,
            clear_region: false,
        }
    }
}
