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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::flow_definition::{FlowDefinition, FlowSlo, FlowStep};
    use crate::domain::entity::flow_instance::{FlowInstance, FlowInstanceStatus};
    use crate::domain::repository::flow_definition_repository::MockFlowDefinitionRepository;
    use crate::domain::repository::flow_instance_repository::MockFlowInstanceRepository;
    use chrono::Utc;

    fn make_flow() -> FlowDefinition {
        FlowDefinition::new(
            "order_flow".to_string(),
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

    fn make_instance(flow_id: Uuid, status: FlowInstanceStatus) -> FlowInstance {
        let mut inst = FlowInstance::new(flow_id, format!("corr-{}", Uuid::new_v4()));
        inst.status = status.clone();
        if status == FlowInstanceStatus::Completed || status == FlowInstanceStatus::Failed {
            inst.completed_at = Some(Utc::now());
            inst.duration_ms = Some(1000);
        }
        inst
    }

    #[tokio::test]
    async fn success_ok() {
        let flow = make_flow();
        let flow_id = flow.id;
        let flow_clone = flow.clone();

        let mut def_mock = MockFlowDefinitionRepository::new();
        def_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(flow_clone.clone())));

        let mut inst_mock = MockFlowInstanceRepository::new();
        inst_mock
            .expect_find_by_flow_id_paginated()
            .returning(move |id, _, _| {
                Ok((
                    vec![
                        make_instance(*id, FlowInstanceStatus::Completed),
                        make_instance(*id, FlowInstanceStatus::Completed),
                        make_instance(*id, FlowInstanceStatus::Completed),
                    ],
                    3,
                ))
            });

        let uc = GetSloBurnRateUseCase::new(Arc::new(def_mock), Arc::new(inst_mock));
        let result = uc.execute(&flow_id).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.alert_status, "ok");
        assert!(output.alert_fired_at.is_none());
    }

    #[tokio::test]
    async fn success_firing() {
        let flow = make_flow();
        let flow_id = flow.id;
        let flow_clone = flow.clone();

        let mut def_mock = MockFlowDefinitionRepository::new();
        def_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(flow_clone.clone())));

        let mut inst_mock = MockFlowInstanceRepository::new();
        inst_mock
            .expect_find_by_flow_id_paginated()
            .returning(move |id, _, _| {
                Ok((
                    vec![
                        make_instance(*id, FlowInstanceStatus::Failed),
                        make_instance(*id, FlowInstanceStatus::Failed),
                        make_instance(*id, FlowInstanceStatus::Failed),
                        make_instance(*id, FlowInstanceStatus::Failed),
                        make_instance(*id, FlowInstanceStatus::Completed),
                    ],
                    5,
                ))
            });

        let uc = GetSloBurnRateUseCase::new(Arc::new(def_mock), Arc::new(inst_mock));
        let result = uc.execute(&flow_id).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.alert_status, "firing");
        assert!(output.alert_fired_at.is_some());
    }

    #[tokio::test]
    async fn not_found() {
        let mut def_mock = MockFlowDefinitionRepository::new();
        def_mock.expect_find_by_id().returning(|_| Ok(None));

        let inst_mock = MockFlowInstanceRepository::new();

        let uc = GetSloBurnRateUseCase::new(Arc::new(def_mock), Arc::new(inst_mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(matches!(result, Err(GetSloBurnRateError::NotFound(_))));
    }
}
