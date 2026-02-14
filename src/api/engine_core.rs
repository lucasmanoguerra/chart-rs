use super::{
    chart_behavior::ChartBehaviorState, chart_model::ChartModel,
    chart_presentation::ChartPresentationState, chart_runtime::ChartRuntimeState,
};

/// Internal engine core state used by the public facade (`ChartEngine`).
pub(super) struct EngineCore {
    pub(super) model: ChartModel,
    pub(super) lwc_model: crate::lwc::model::ChartModel,
    pub(super) behavior: ChartBehaviorState,
    pub(super) presentation: ChartPresentationState,
    pub(super) runtime: ChartRuntimeState,
}
