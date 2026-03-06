use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::flow_kpi::{FlowKpi, SloStatus};
use crate::domain::repository::{FlowDefinitionRepository, FlowInstanceRepository};
use crate::domain::service::kpi_aggregation::KpiAggregationService;
use crate::domain::service::slo_calculation::SloCalculationService;
use crate::infrastructure::cache::KpiCache;

#[derive(Debug)]
pub struct GetFlowKpiOutput {
    pub flow_id: Uuid,
    pub flow_name: String,
    pub period: String,
    pub kpi: FlowKpi,
    pub slo_status: SloStatus,
}

#[derive(Debug, thiserror::Error)]
pub enum GetFlowKpiError {
    #[error("flow not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetFlowKpiUseCase {
    flow_def_repo: Arc<dyn FlowDefinitionRepository>,
    flow_inst_repo: Arc<dyn FlowInstanceRepository>,
}

impl GetFlowKpiUseCase {
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
        period: &str,
    ) -> Result<GetFlowKpiOutput, GetFlowKpiError> {
        let flow = self
            .flow_def_repo
            .find_by_id(flow_id)
            .await
            .map_err(|e| GetFlowKpiError::Internal(e.to_string()))?
            .ok_or_else(|| GetFlowKpiError::NotFound(flow_id.to_string()))?;

        // Get all instances for this flow (simplified; in production, filter by period)
        let (instances, _) = self
            .flow_inst_repo
            .find_by_flow_id_paginated(flow_id, 1, 10000)
            .await
            .map_err(|e| GetFlowKpiError::Internal(e.to_string()))?;

        let kpi = KpiAggregationService::aggregate(&instances);
        let slo_status = SloCalculationService::calculate(&flow, &instances);

        Ok(GetFlowKpiOutput {
            flow_id: flow.id,
            flow_name: flow.name,
            period: period.to_string(),
            kpi,
            slo_status,
        })
    }
}
