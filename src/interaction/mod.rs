use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionMode {
    Idle,
    Panning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrosshairMode {
    /// Crosshair follows nearest data sample (current default behavior).
    Magnet,
    /// Crosshair follows raw pointer position without snapping.
    Normal,
}

/// Deterministic snap candidate used to drive crosshair visuals and labels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CrosshairSnap {
    pub x: f64,
    pub y: f64,
    pub time: f64,
    pub price: f64,
}

/// Public crosshair state exposed to host applications.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CrosshairState {
    pub visible: bool,
    pub x: f64,
    pub y: f64,
    pub snapped_x: Option<f64>,
    pub snapped_y: Option<f64>,
    pub snapped_time: Option<f64>,
    pub snapped_price: Option<f64>,
}

impl Default for CrosshairState {
    fn default() -> Self {
        Self {
            visible: false,
            x: 0.0,
            y: 0.0,
            snapped_x: None,
            snapped_y: None,
            snapped_time: None,
            snapped_price: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InteractionState {
    mode: InteractionMode,
    crosshair_mode: CrosshairMode,
    cursor_x: f64,
    cursor_y: f64,
    crosshair: CrosshairState,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            mode: InteractionMode::Idle,
            crosshair_mode: CrosshairMode::Magnet,
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
    pub fn crosshair_mode(self) -> CrosshairMode {
        self.crosshair_mode
    }

    pub fn set_crosshair_mode(&mut self, mode: CrosshairMode) {
        self.crosshair_mode = mode;
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
        self.crosshair.snapped_time = None;
        self.crosshair.snapped_price = None;
    }

    pub fn set_crosshair_snap(&mut self, snap: Option<CrosshairSnap>) {
        match snap {
            Some(snap) => {
                self.crosshair.snapped_x = Some(snap.x);
                self.crosshair.snapped_y = Some(snap.y);
                self.crosshair.snapped_time = Some(snap.time);
                self.crosshair.snapped_price = Some(snap.price);
            }
            None => {
                self.crosshair.snapped_x = None;
                self.crosshair.snapped_y = None;
                self.crosshair.snapped_time = None;
                self.crosshair.snapped_price = None;
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
