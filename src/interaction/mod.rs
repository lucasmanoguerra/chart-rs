use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    Idle,
    Panning,
}

/// Public crosshair state exposed to host applications.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CrosshairState {
    pub visible: bool,
    pub x: f64,
    pub y: f64,
    pub snapped_x: Option<f64>,
    pub snapped_y: Option<f64>,
}

impl Default for CrosshairState {
    fn default() -> Self {
        Self {
            visible: false,
            x: 0.0,
            y: 0.0,
            snapped_x: None,
            snapped_y: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InteractionState {
    mode: InteractionMode,
    cursor_x: f64,
    cursor_y: f64,
    crosshair: CrosshairState,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            mode: InteractionMode::Idle,
            cursor_x: 0.0,
            cursor_y: 0.0,
            crosshair: CrosshairState::default(),
        }
    }
}

impl InteractionState {
    #[must_use]
    pub fn mode(self) -> InteractionMode {
        self.mode
    }

    #[must_use]
    pub fn cursor(self) -> (f64, f64) {
        (self.cursor_x, self.cursor_y)
    }

    #[must_use]
    pub fn crosshair(self) -> CrosshairState {
        self.crosshair
    }

    pub fn on_pointer_move(&mut self, x: f64, y: f64) {
        self.cursor_x = x;
        self.cursor_y = y;
        self.crosshair.visible = true;
        self.crosshair.x = x;
        self.crosshair.y = y;
    }

    pub fn on_pointer_leave(&mut self) {
        self.crosshair.visible = false;
        self.crosshair.snapped_x = None;
        self.crosshair.snapped_y = None;
    }

    pub fn set_crosshair_snap(&mut self, snap: Option<(f64, f64)>) {
        match snap {
            Some((x, y)) => {
                self.crosshair.snapped_x = Some(x);
                self.crosshair.snapped_y = Some(y);
            }
            None => {
                self.crosshair.snapped_x = None;
                self.crosshair.snapped_y = None;
            }
        }
    }

    pub fn on_pan_start(&mut self) {
        self.mode = InteractionMode::Panning;
    }

    pub fn on_pan_end(&mut self) {
        self.mode = InteractionMode::Idle;
    }
}
