use serde::{Deserialize, Serialize};

use crate::error::{ChartError, ChartResult};
use crate::render::Renderer;

use super::{ChartEngine, CrosshairFormatterDiagnostics, EngineSnapshot};

pub const ENGINE_SNAPSHOT_JSON_SCHEMA_V1: u32 = 1;
pub const CROSSHAIR_DIAGNOSTICS_JSON_SCHEMA_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineSnapshotJsonContractV1 {
    pub schema_version: u32,
    pub snapshot: EngineSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrosshairFormatterDiagnosticsJsonContractV1 {
    pub schema_version: u32,
    pub diagnostics: CrosshairFormatterDiagnostics,
}

impl EngineSnapshot {
    pub fn to_json_contract_v1_pretty(&self) -> ChartResult<String> {
        let payload = EngineSnapshotJsonContractV1 {
            schema_version: ENGINE_SNAPSHOT_JSON_SCHEMA_V1,
            snapshot: self.clone(),
        };
        serde_json::to_string_pretty(&payload).map_err(|e| {
            ChartError::InvalidData(format!("failed to serialize snapshot contract v1: {e}"))
        })
    }

    pub fn from_json_compat_str(input: &str) -> ChartResult<Self> {
        if let Ok(snapshot) = serde_json::from_str::<EngineSnapshot>(input) {
            return Ok(snapshot);
        }
        let payload: EngineSnapshotJsonContractV1 = serde_json::from_str(input).map_err(|e| {
            ChartError::InvalidData(format!("failed to parse snapshot json payload: {e}"))
        })?;
        if payload.schema_version != ENGINE_SNAPSHOT_JSON_SCHEMA_V1 {
            return Err(ChartError::InvalidData(format!(
                "unsupported snapshot schema version: {}",
                payload.schema_version
            )));
        }
        Ok(payload.snapshot)
    }
}

impl CrosshairFormatterDiagnostics {
    pub fn to_json_pretty(self) -> ChartResult<String> {
        serde_json::to_string_pretty(&self).map_err(|e| {
            ChartError::InvalidData(format!("failed to serialize diagnostics json: {e}"))
        })
    }

    pub fn to_json_contract_v1_pretty(self) -> ChartResult<String> {
        let payload = CrosshairFormatterDiagnosticsJsonContractV1 {
            schema_version: CROSSHAIR_DIAGNOSTICS_JSON_SCHEMA_V1,
            diagnostics: self,
        };
        serde_json::to_string_pretty(&payload).map_err(|e| {
            ChartError::InvalidData(format!(
                "failed to serialize crosshair diagnostics contract v1: {e}"
            ))
        })
    }

    pub fn from_json_compat_str(input: &str) -> ChartResult<Self> {
        if let Ok(diagnostics) = serde_json::from_str::<CrosshairFormatterDiagnostics>(input) {
            return Ok(diagnostics);
        }
        let payload: CrosshairFormatterDiagnosticsJsonContractV1 = serde_json::from_str(input)
            .map_err(|e| {
                ChartError::InvalidData(format!("failed to parse diagnostics json payload: {e}"))
            })?;
        if payload.schema_version != CROSSHAIR_DIAGNOSTICS_JSON_SCHEMA_V1 {
            return Err(ChartError::InvalidData(format!(
                "unsupported crosshair diagnostics schema version: {}",
                payload.schema_version
            )));
        }
        Ok(payload.diagnostics)
    }
}

impl<R: Renderer> ChartEngine<R> {
    pub fn snapshot_json_contract_v1_pretty(&self, body_width_px: f64) -> ChartResult<String> {
        self.snapshot(body_width_px)?.to_json_contract_v1_pretty()
    }

    pub fn crosshair_formatter_diagnostics_json_pretty(&self) -> ChartResult<String> {
        self.crosshair_formatter_diagnostics().to_json_pretty()
    }

    pub fn crosshair_formatter_diagnostics_json_contract_v1_pretty(&self) -> ChartResult<String> {
        self.crosshair_formatter_diagnostics()
            .to_json_contract_v1_pretty()
    }
}
