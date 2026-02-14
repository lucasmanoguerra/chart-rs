use crate::error::ChartResult;
use crate::interaction::{
    CrosshairMode, CrosshairState, InteractionMode, KineticPanConfig, KineticPanState,
};
use crate::render::Renderer;

use super::interaction_validation::validate_kinetic_pan_config;
use super::{
    ChartEngine, InteractionInputBehavior, interaction_coordinator::InteractionCoordinator,
};

impl<R: Renderer> ChartEngine<R> {
    #[must_use]
    pub fn interaction_mode(&self) -> InteractionMode {
        self.core.model.interaction.mode()
    }

    #[must_use]
    pub fn interaction_input_behavior(&self) -> InteractionInputBehavior {
        self.core.behavior.interaction_input_behavior
    }

    pub fn set_interaction_input_behavior(&mut self, behavior: InteractionInputBehavior) {
        self.core.behavior.interaction_input_behavior = behavior;
    }

    #[must_use]
    pub fn crosshair_mode(&self) -> CrosshairMode {
        self.core.model.interaction.crosshair_mode()
    }

    pub fn set_crosshair_mode(&mut self, mode: CrosshairMode) {
        InteractionCoordinator::set_crosshair_mode(self, mode);
    }

    #[must_use]
    pub fn kinetic_pan_config(&self) -> KineticPanConfig {
        self.core.model.interaction.kinetic_pan_config()
    }

    pub fn set_kinetic_pan_config(&mut self, config: KineticPanConfig) -> ChartResult<()> {
        validate_kinetic_pan_config(config)?;
        self.core.model.interaction.set_kinetic_pan_config(config);
        Ok(())
    }

    #[must_use]
    pub fn kinetic_pan_state(&self) -> KineticPanState {
        self.core.model.interaction.kinetic_pan_state()
    }

    /// Starts kinetic pan with signed velocity in time-units per second.
    pub fn start_kinetic_pan(&mut self, velocity_time_per_sec: f64) -> ChartResult<()> {
        InteractionCoordinator::start_kinetic_pan(self, velocity_time_per_sec)
    }

    pub fn stop_kinetic_pan(&mut self) {
        InteractionCoordinator::stop_kinetic_pan(self);
    }

    #[must_use]
    pub fn crosshair_state(&self) -> CrosshairState {
        self.core.model.interaction.crosshair()
    }

    /// Handles pointer movement and updates crosshair snapping in one step.
    pub fn pointer_move(&mut self, x: f64, y: f64) {
        InteractionCoordinator::pointer_move(self, x, y);
    }

    /// Marks pointer as outside chart bounds.
    pub fn pointer_leave(&mut self) {
        InteractionCoordinator::pointer_leave(self);
    }

    pub fn pan_start(&mut self) {
        InteractionCoordinator::pan_start(self);
    }

    pub fn pan_end(&mut self) {
        InteractionCoordinator::pan_end(self);
    }
}
