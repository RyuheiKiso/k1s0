use std::sync::Arc;

use crate::domain::repository::{FlowDefinitionRepository, FlowInstanceRepository};
use crate::domain::service::kpi_aggregation::KpiAggregationService;
use crate::domain::service::slo_calculation::SloCalculationService;

#[derive(Debug)]
pub struct FlowKpiSummaryItem {
    pub flow_id: String,
    pub flow_name: String,
    pub domain: String,
    pub total_started: i64,
    pub completion_rate: f64,
    pub avg_duration_seconds: f64,
    pub slo_violated: bool,
}

#[derive(Debug)]
pub struct GetKpiSummaryOutput {
    pub period: String,
    pub flows: Vec<FlowKpiSummaryItem>,
    pub total_flows: i32,
    pub flows_with_slo_violation: i32,
    pub overall_completion_rate: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum GetKpiSummaryError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetKpiSummaryUseCase {
    flow_def_repo: Arc<dyn FlowDefinitionRepository>,
    flow_inst_repo: Arc<dyn FlowInstanceRepository>,
}

impl GetKpiSummaryUseCase {
    pub fn new(
        flow_def_repo: Arc<dyn FlowDefinitionRepository>,
        flow_inst_repo: Arc<dyn FlowInstanceRepository>,
    ) -> Self {
        Self {
            flow_def_repo,
            flow_inst_repo,
        }
    }

    pub async fn execute(&self, period: &str) -> Result<GetKpiSummaryOutput, GetKpiSummaryError> {
        let flows = self
            .flow_def_repo
            .find_all()
            .await
            .map_err(|e| GetKpiSummaryError::Internal(e.to_string()))?;

        let mut items = Vec::new();
        let mut total_started_all = 0i64;
        let mut total_completed_all = 0i64;
        let mut slo_violations = 0i32;

        for flow in &flows {
            let (instances, _) = self
                .flow_inst_repo
                .find_by_flow_id_paginated(&flow.id, 1, 10000)
                .await
                .map_err(|e| GetKpiSummaryError::Internal(e.to_string()))?;

            let kpi = KpiAggregationService::aggregate(&instances);
            let slo = SloCalculationService::calculate(flow, &instances);

            total_started_all += kpi.total_started;
            total_completed_all += kpi.total_completed;
            if slo.is_violated {
                slo_violations += 1;
            }

            items.push(FlowKpiSummaryItem {
                flow_id: flow.id.to_string(),
                flow_name: flow.name.clone(),
                domain: flow.domain.clone(),
                total_started: kpi.total_started,
                completion_rate: kpi.completion_rate,
                avg_duration_seconds: kpi.avg_duration_seconds,
                slo_violated: slo.is_violated,
            });
        }

        let overall_completion_rate = if total_started_all > 0 {
            total_completed_all as f64 / total_started_all as f64
        } else {
            0.0
        };

        Ok(GetKpiSummaryOutput {
            period: period.to_string(),
            flows: items,
            total_flows: flows.len() as i32,
            flows_with_slo_violation: slo_violations,
            overall_completion_rate,
        })
    }
}
