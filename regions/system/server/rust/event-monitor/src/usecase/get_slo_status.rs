use std::sync::Arc;

use crate::domain::repository::{FlowDefinitionRepository, FlowInstanceRepository};
use crate::domain::service::slo_calculation::SloCalculationService;

#[derive(Debug)]
pub struct SloFlowStatusItem {
    pub flow_id: String,
    pub flow_name: String,
    pub is_violated: bool,
    pub burn_rate: f64,
    pub error_budget_remaining: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum GetSloStatusError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetSloStatusUseCase {
    flow_def_repo: Arc<dyn FlowDefinitionRepository>,
    flow_inst_repo: Arc<dyn FlowInstanceRepository>,
}

impl GetSloStatusUseCase {
    pub fn new(
        flow_def_repo: Arc<dyn FlowDefinitionRepository>,
        flow_inst_repo: Arc<dyn FlowInstanceRepository>,
    ) -> Self {
        Self {
            flow_def_repo,
            flow_inst_repo,
        }
    }

    pub async fn execute(&self) -> Result<Vec<SloFlowStatusItem>, GetSloStatusError> {
        let flows = self
            .flow_def_repo
            .find_all()
            .await
            .map_err(|e| GetSloStatusError::Internal(e.to_string()))?;

        let mut items = Vec::new();

        for flow in &flows {
            let (instances, _) = self
                .flow_inst_repo
                .find_by_flow_id_paginated(&flow.id, 1, 10000)
                .await
                .map_err(|e| GetSloStatusError::Internal(e.to_string()))?;

            let slo = SloCalculationService::calculate(flow, &instances);

            let error_budget = 1.0 - flow.slo.target_success_rate;
            let actual_error_rate = 1.0 - slo.current_success_rate;
            let error_budget_remaining = if error_budget > 0.0 {
                (1.0 - (actual_error_rate / error_budget)).max(0.0)
            } else {
                1.0
            };

            items.push(SloFlowStatusItem {
                flow_id: flow.id.to_string(),
                flow_name: flow.name.clone(),
                is_violated: slo.is_violated,
                burn_rate: slo.burn_rate,
                error_budget_remaining,
            });
        }

        Ok(items)
    }
}
