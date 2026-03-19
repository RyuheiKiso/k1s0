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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::flow_definition::{FlowDefinition, FlowSlo, FlowStep};
    use crate::domain::entity::flow_instance::{FlowInstance, FlowInstanceStatus};
    use crate::domain::repository::flow_definition_repository::MockFlowDefinitionRepository;
    use crate::domain::repository::flow_instance_repository::MockFlowInstanceRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_flow(name: &str) -> FlowDefinition {
        FlowDefinition::new(
            name.to_string(),
            "test".to_string(),
            "service.order".to_string(),
            vec![FlowStep {
                event_type: "OrderCreated".to_string(),
                source: "order-service".to_string(),
                source_filter: None,
                timeout_seconds: 30,
                description: String::new(),
            }],
            FlowSlo {
                target_completion_seconds: 120,
                target_success_rate: 0.99,
                alert_on_violation: true,
            },
        )
    }

    fn make_completed_instance(flow_id: Uuid) -> FlowInstance {
        let mut inst = FlowInstance::new(flow_id, "corr-1".to_string());
        inst.status = FlowInstanceStatus::Completed;
        inst.completed_at = Some(Utc::now());
        inst.duration_ms = Some(1000);
        inst
    }

    #[tokio::test]
    async fn with_flows() {
        let mut def_mock = MockFlowDefinitionRepository::new();
        def_mock
            .expect_find_all()
            .returning(move || Ok(vec![make_flow("flow_a"), make_flow("flow_b")]));

        let mut inst_mock = MockFlowInstanceRepository::new();
        inst_mock
            .expect_find_by_flow_id_paginated()
            .returning(move |id, _, _| Ok((vec![make_completed_instance(*id)], 1)));

        let uc = GetKpiSummaryUseCase::new(Arc::new(def_mock), Arc::new(inst_mock));
        let result = uc.execute("24h").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.total_flows, 2);
        assert_eq!(output.flows.len(), 2);
        assert!(output.overall_completion_rate > 0.0);
    }

    #[tokio::test]
    async fn empty_flows() {
        let mut def_mock = MockFlowDefinitionRepository::new();
        def_mock.expect_find_all().returning(|| Ok(vec![]));

        let inst_mock = MockFlowInstanceRepository::new();

        let uc = GetKpiSummaryUseCase::new(Arc::new(def_mock), Arc::new(inst_mock));
        let result = uc.execute("24h").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.flows.is_empty());
        assert_eq!(output.total_flows, 0);
        assert_eq!(output.overall_completion_rate, 0.0);
    }
}
