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
            "system".to_string(),
            name.to_string(),
            "test".to_string(),
            "service.task".to_string(),
            vec![FlowStep {
                event_type: "TaskCreated".to_string(),
                source: "task-server".to_string(),
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
        let mut inst = FlowInstance::new("system".to_string(), flow_id, "corr-1".to_string());
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
            .returning(|| Ok(vec![make_flow("flow_a"), make_flow("flow_b")]));

        let mut inst_mock = MockFlowInstanceRepository::new();
        inst_mock
            .expect_find_by_flow_id_paginated()
            .returning(move |id, _, _| Ok((vec![make_completed_instance(*id)], 1)));

        let uc = GetSloStatusUseCase::new(Arc::new(def_mock), Arc::new(inst_mock));
        let result = uc.execute().await;
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 2);
        for item in &items {
            assert!(!item.flow_name.is_empty());
        }
    }

    #[tokio::test]
    async fn empty_flows() {
        let mut def_mock = MockFlowDefinitionRepository::new();
        def_mock.expect_find_all().returning(|| Ok(vec![]));

        let inst_mock = MockFlowInstanceRepository::new();

        let uc = GetSloStatusUseCase::new(Arc::new(def_mock), Arc::new(inst_mock));
        let result = uc.execute().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
