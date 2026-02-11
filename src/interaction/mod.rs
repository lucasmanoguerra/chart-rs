#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    Idle,
    Panning,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InteractionState {
    mode: InteractionMode,
    cursor_x: f64,
    cursor_y: f64,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            mode: InteractionMode::Idle,
            cursor_x: 0.0,
            cursor_y: 0.0,
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

    pub fn on_pointer_move(&mut self, x: f64, y: f64) {
        self.cursor_x = x;
        self.cursor_y = y;
    }

    pub fn on_pan_start(&mut self) {
        self.mode = InteractionMode::Panning;
    }

    pub fn on_pan_end(&mut self) {
        self.mode = InteractionMode::Idle;
    }
}
