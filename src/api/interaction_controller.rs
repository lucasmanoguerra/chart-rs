use crate::error::{ChartError, ChartResult};
use crate::interaction::{
    CrosshairMode, CrosshairState, InteractionMode, KineticPanConfig, KineticPanState,
};
use crate::render::Renderer;

use super::interaction_validation::validate_kinetic_pan_config;
use super::{ChartEngine, InteractionInputBehavior, PluginEvent};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn interaction_mode(&self) -> InteractionMode {
        self.interaction.mode()
    }

    #[must_use]
    pub fn interaction_input_behavior(&self) -> InteractionInputBehavior {
        self.interaction_input_behavior
    }

    pub fn set_interaction_input_behavior(&mut self, behavior: InteractionInputBehavior) {
        self.interaction_input_behavior = behavior;
    }

    #[must_use]
    pub fn crosshair_mode(&self) -> CrosshairMode {
        self.interaction.crosshair_mode()
    }

    pub fn set_crosshair_mode(&mut self, mode: CrosshairMode) {
        self.clear_crosshair_context_formatter_caches_if_needed();
        self.interaction.set_crosshair_mode(mode);
        if mode == CrosshairMode::Hidden {
            self.interaction.on_pointer_leave();
        }
    }

    #[must_use]
    pub fn kinetic_pan_config(&self) -> KineticPanConfig {
        self.interaction.kinetic_pan_config()
    }

    pub fn set_kinetic_pan_config(&mut self, config: KineticPanConfig) -> ChartResult<()> {
        validate_kinetic_pan_config(config)?;
        self.interaction.set_kinetic_pan_config(config);
        Ok(())
    }

    #[must_use]
    pub fn kinetic_pan_state(&self) -> KineticPanState {
        self.interaction.kinetic_pan_state()
    }

    /// Starts kinetic pan with signed velocity in time-units per second.
    pub fn start_kinetic_pan(&mut self, velocity_time_per_sec: f64) -> ChartResult<()> {
        if !velocity_time_per_sec.is_finite() {
            return Err(ChartError::InvalidData(
                "kinetic pan velocity must be finite".to_owned(),
            ));
        }
        if velocity_time_per_sec == 0.0 {
            self.stop_kinetic_pan();
            return Ok(());
        }
        self.interaction.start_kinetic_pan(velocity_time_per_sec);
        self.emit_plugin_event(PluginEvent::PanStarted);
        Ok(())
    }

    pub fn stop_kinetic_pan(&mut self) {
        if self.interaction.kinetic_pan_state().active {
            self.interaction.stop_kinetic_pan();
            self.emit_plugin_event(PluginEvent::PanEnded);
        }
    }

    #[must_use]
    pub fn crosshair_state(&self) -> CrosshairState {
        self.interaction.crosshair()
    }

    /// Handles pointer movement and updates crosshair snapping in one step.
    pub fn pointer_move(&mut self, x: f64, y: f64) {
        self.interaction.on_pointer_move(x, y);
        match self.interaction.crosshair_mode() {
            CrosshairMode::Magnet => self.interaction.set_crosshair_snap(self.snap_at_x(x)),
            CrosshairMode::Normal => self.interaction.set_crosshair_snap(None),
            CrosshairMode::Hidden => self.interaction.on_pointer_leave(),
        }
        self.emit_plugin_event(PluginEvent::PointerMoved { x, y });
    }

    /// Marks pointer as outside chart bounds.
    pub fn pointer_leave(&mut self) {
        self.interaction.on_pointer_leave();
        self.emit_plugin_event(PluginEvent::PointerLeft);
    }

    pub fn pan_start(&mut self) {
        if !self.interaction_input_behavior.allows_drag_pan() {
            return;
        }
        self.interaction.on_pan_start();
        self.emit_plugin_event(PluginEvent::PanStarted);
    }

    pub fn pan_end(&mut self) {
        if !self.interaction_input_behavior.allows_drag_pan() {
            return;
        }
        self.interaction.on_pan_end();
        self.emit_plugin_event(PluginEvent::PanEnded);
    }
}
