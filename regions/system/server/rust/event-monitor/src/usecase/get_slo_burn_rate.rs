use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::flow_kpi::BurnRateWindow;
use crate::domain::repository::{FlowDefinitionRepository, FlowInstanceRepository};
use crate::domain::service::slo_calculation::SloCalculationService;

#[derive(Debug)]
pub struct GetSloBurnRateOutput {
    pub flow_id: String,
    pub flow_name: String,
    pub windows: Vec<BurnRateWindow>,
    pub alert_status: String,
    pub alert_fired_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum GetSloBurnRateError {
    #[error("flow not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetSloBurnRateUseCase {
    flow_def_repo: Arc<dyn FlowDefinitionRepository>,
    flow_inst_repo: Arc<dyn FlowInstanceRepository>,
}

impl GetSloBurnRateUseCase {
    pub fn new(
        flow_def_repo: Arc<dyn FlowDefinitionRepository>,
        flow_inst_repo: Arc<dyn FlowInstanceRepository>,
    ) -> Self {
        Self {
            flow_def_repo,
            flow_inst_repo,
        }
    }

    pub async fn execute(
        &self,
        flow_id: &Uuid,
    ) -> Result<GetSloBurnRateOutput, GetSloBurnRateError> {
        let flow = self
            .flow_def_repo
            .find_by_id(flow_id)
            .await
            .map_err(|e| GetSloBurnRateError::Internal(e.to_string()))?
            .ok_or_else(|| GetSloBurnRateError::NotFound(flow_id.to_string()))?;

        let (instances, _) = self
            .flow_inst_repo
            .find_by_flow_id_paginated(flow_id, 1, 10000)
            .await
            .map_err(|e| GetSloBurnRateError::Internal(e.to_string()))?;

        // Calculate burn rate for multiple windows using same data (simplified)
        let window_labels = ["1h", "6h", "24h", "30d"];
        let windows_data: Vec<(&str, &[_])> = window_labels
            .iter()
            .map(|w| (*w, instances.as_slice()))
            .collect();

        let windows = SloCalculationService::calculate_burn_rate(&flow, &windows_data);

        let any_firing = windows.iter().any(|w| w.burn_rate > 1.0);
        let alert_status = if any_firing { "firing" } else { "ok" };

        Ok(GetSloBurnRateOutput {
            flow_id: flow.id.to_string(),
            flow_name: flow.name,
            windows,
            alert_status: alert_status.to_string(),
            alert_fired_at: if any_firing {
                Some(chrono::Utc::now())
            } else {
                None
            },
        })
    }
}
