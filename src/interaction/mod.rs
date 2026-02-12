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
    /// Crosshair remains hidden regardless of pointer movement.
    Hidden,
}

/// Tuning for deterministic kinetic pan stepping.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KineticPanConfig {
    /// Multiplicative velocity decay per second.
    pub decay_per_second: f64,
    /// Kinetic pan stops when `abs(velocity)` drops below this threshold.
    pub stop_velocity_abs: f64,
}

impl Default for KineticPanConfig {
    fn default() -> Self {
        Self {
            decay_per_second: 0.85,
            stop_velocity_abs: 0.01,
        }
    }
}

/// Public kinetic pan runtime state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KineticPanState {
    pub active: bool,
    pub velocity_time_per_sec: f64,
}

impl Default for KineticPanState {
    fn default() -> Self {
        Self {
            active: false,
            velocity_time_per_sec: 0.0,
        }
    }
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
    kinetic_pan_config: KineticPanConfig,
    kinetic_pan: KineticPanState,
    cursor_x: f64,
    cursor_y: f64,
    crosshair: CrosshairState,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            mode: InteractionMode::Idle,
            crosshair_mode: CrosshairMode::Magnet,
            kinetic_pan_config: KineticPanConfig::default(),
            kinetic_pan: KineticPanState::default(),
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
    pub fn kinetic_pan_config(self) -> KineticPanConfig {
        self.kinetic_pan_config
    }

    pub fn set_kinetic_pan_config(&mut self, config: KineticPanConfig) {
        self.kinetic_pan_config = config;
    }

    #[must_use]
    pub fn kinetic_pan_state(self) -> KineticPanState {
        self.kinetic_pan
    }

    pub fn start_kinetic_pan(&mut self, velocity_time_per_sec: f64) {
        self.kinetic_pan.active = true;
        self.kinetic_pan.velocity_time_per_sec = velocity_time_per_sec;
    }

    pub fn stop_kinetic_pan(&mut self) {
        self.kinetic_pan.active = false;
        self.kinetic_pan.velocity_time_per_sec = 0.0;
    }

    /// Advances kinetic pan and returns the time displacement to apply.
    ///
    /// Returns `None` when kinetic pan is not active.
    pub fn step_kinetic_pan(&mut self, delta_seconds: f64) -> Option<f64> {
        if !self.kinetic_pan.active {
            return None;
        }

        let displacement = self.kinetic_pan.velocity_time_per_sec * delta_seconds;
        let decay = self.kinetic_pan_config.decay_per_second.powf(delta_seconds);
        self.kinetic_pan.velocity_time_per_sec *= decay;

        if self.kinetic_pan.velocity_time_per_sec.abs() < self.kinetic_pan_config.stop_velocity_abs
        {
            self.stop_kinetic_pan();
        }

        Some(displacement)
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
