use crate::error::{ChartError, ChartResult};
use crate::interaction::CrosshairMode;
use crate::render::Renderer;

use super::{ChartEngine, PluginEvent};

pub(super) struct InteractionCoordinator;

impl InteractionCoordinator {
    pub(super) fn set_crosshair_mode<R: Renderer>(
        engine: &mut ChartEngine<R>,
        mode: CrosshairMode,
    ) {
        engine.clear_crosshair_context_formatter_caches_if_needed();
        engine.core.model.interaction.set_crosshair_mode(mode);
        if mode == CrosshairMode::Hidden {
            engine.core.model.interaction.on_pointer_leave();
        }
        engine.invalidate_cursor();
    }

    pub(super) fn start_kinetic_pan<R: Renderer>(
        engine: &mut ChartEngine<R>,
        velocity_time_per_sec: f64,
    ) -> ChartResult<()> {
        if !velocity_time_per_sec.is_finite() {
            return Err(ChartError::InvalidData(
                "kinetic pan velocity must be finite".to_owned(),
            ));
        }
        if velocity_time_per_sec == 0.0 {
            Self::stop_kinetic_pan(engine);
            return Ok(());
        }
        engine
            .core
            .model
            .interaction
            .start_kinetic_pan(velocity_time_per_sec);
        engine.emit_plugin_event(PluginEvent::PanStarted);
        Ok(())
    }

    pub(super) fn stop_kinetic_pan<R: Renderer>(engine: &mut ChartEngine<R>) {
        if engine.core.model.interaction.kinetic_pan_state().active {
            engine.core.model.interaction.stop_kinetic_pan();
            engine.emit_plugin_event(PluginEvent::PanEnded);
        }
    }

    pub(super) fn step_kinetic_pan<R: Renderer>(
        engine: &mut ChartEngine<R>,
        delta_seconds: f64,
    ) -> ChartResult<bool> {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return Err(ChartError::InvalidData(
                "kinetic pan delta seconds must be finite and > 0".to_owned(),
            ));
        }

        let was_active = engine.core.model.interaction.kinetic_pan_state().active;
        let Some(displacement) = engine
            .core
            .model
            .interaction
            .step_kinetic_pan(delta_seconds)
        else {
            return Ok(false);
        };

        engine.pan_time_visible_by(displacement)?;

        if was_active && !engine.core.model.interaction.kinetic_pan_state().active {
            engine.emit_plugin_event(PluginEvent::PanEnded);
        }
        Ok(true)
    }

    pub(super) fn pointer_move<R: Renderer>(engine: &mut ChartEngine<R>, x: f64, y: f64) {
        engine.core.model.interaction.on_pointer_move(x, y);
        let crosshair_mode = engine.core.model.interaction.crosshair_mode();
        match crosshair_mode {
            CrosshairMode::Magnet => {
                let snap = engine.snap_at_x(x);
                engine.core.model.interaction.set_crosshair_snap(snap);
            }
            CrosshairMode::Normal => engine.core.model.interaction.set_crosshair_snap(None),
            CrosshairMode::Hidden => engine.core.model.interaction.on_pointer_leave(),
        }
        engine.emit_plugin_event(PluginEvent::PointerMoved { x, y });
    }

    pub(super) fn pointer_leave<R: Renderer>(engine: &mut ChartEngine<R>) {
        engine.core.model.interaction.on_pointer_leave();
        engine.emit_plugin_event(PluginEvent::PointerLeft);
    }

    pub(super) fn pan_start<R: Renderer>(engine: &mut ChartEngine<R>) {
        if !engine
            .core
            .behavior
            .interaction_input_behavior
            .allows_drag_pan()
        {
            return;
        }
        engine.core.model.interaction.on_pan_start();
        engine.emit_plugin_event(PluginEvent::PanStarted);
    }

    pub(super) fn pan_end<R: Renderer>(engine: &mut ChartEngine<R>) {
        if !engine
            .core
            .behavior
            .interaction_input_behavior
            .allows_drag_pan()
        {
            return;
        }
        engine.core.model.interaction.on_pan_end();
        engine.emit_plugin_event(PluginEvent::PanEnded);
    }
}
