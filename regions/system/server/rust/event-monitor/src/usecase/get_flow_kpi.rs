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
    kpi_cache: Option<Arc<KpiCache>>,
}

impl GetFlowKpiUseCase {
    pub fn new(
        flow_def_repo: Arc<dyn FlowDefinitionRepository>,
        flow_inst_repo: Arc<dyn FlowInstanceRepository>,
    ) -> Self {
        Self {
            flow_def_repo,
            flow_inst_repo,
            kpi_cache: None,
        }
    }

    pub fn with_cache(mut self, cache: Arc<KpiCache>) -> Self {
        self.kpi_cache = Some(cache);
        self
    }

    pub async fn execute(
        &self,
        flow_id: &Uuid,
        period: &str,
    ) -> Result<GetFlowKpiOutput, GetFlowKpiError> {
        // Check cache first
        let cache_key = format!("flow_kpi:{}:{}", flow_id, period);
        if let Some(ref cache) = self.kpi_cache {
            if let Some(cached_kpi) = cache.get(&cache_key).await {
                let flow = self
                    .flow_def_repo
                    .find_by_id(flow_id)
                    .await
                    .map_err(|e| GetFlowKpiError::Internal(e.to_string()))?
                    .ok_or_else(|| GetFlowKpiError::NotFound(flow_id.to_string()))?;

                let slo_status = SloCalculationService::calculate_from_kpi(&flow, &cached_kpi);

                return Ok(GetFlowKpiOutput {
                    flow_id: flow.id,
                    flow_name: flow.name,
                    period: period.to_string(),
                    kpi: (*cached_kpi).clone(),
                    slo_status,
                });
            }
        }

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

        // Store in cache
        if let Some(ref cache) = self.kpi_cache {
            cache.insert(cache_key, Arc::new(kpi.clone())).await;
        }

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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::flow_definition::{FlowDefinition, FlowSlo, FlowStep};
    use crate::domain::entity::flow_instance::{FlowInstance, FlowInstanceStatus};
    use crate::domain::repository::flow_definition_repository::MockFlowDefinitionRepository;
    use crate::domain::repository::flow_instance_repository::MockFlowInstanceRepository;
    use chrono::Utc;

    fn make_flow() -> FlowDefinition {
        FlowDefinition::new(
            "task_flow".to_string(),
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
        let mut inst = FlowInstance::new(flow_id, "corr-1".to_string());
        inst.status = FlowInstanceStatus::Completed;
        inst.completed_at = Some(Utc::now());
        inst.duration_ms = Some(1000);
        inst
    }

    #[tokio::test]
    async fn no_cache() {
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
            .returning(move |id, _, _| Ok((vec![make_completed_instance(*id)], 1)));

        let uc = GetFlowKpiUseCase::new(Arc::new(def_mock), Arc::new(inst_mock));
        let result = uc.execute(&flow_id, "24h").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.flow_name, "task_flow");
        assert_eq!(output.kpi.total_started, 1);
    }

    #[tokio::test]
    async fn cached() {
        let flow = make_flow();
        let flow_id = flow.id;
        let flow_clone = flow.clone();

        let mut def_mock = MockFlowDefinitionRepository::new();
        def_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(flow_clone.clone())));

        let inst_mock = MockFlowInstanceRepository::new();
        // No expectations on inst_mock -- should not be called

        let cache = Arc::new(KpiCache::new(100, 300));
        let cached_kpi = FlowKpi {
            total_started: 10,
            total_completed: 9,
            total_failed: 1,
            total_in_progress: 0,
            completion_rate: 0.9,
            avg_duration_seconds: 5.0,
            p50_duration_seconds: 4.0,
            p95_duration_seconds: 8.0,
            p99_duration_seconds: 10.0,
            bottleneck_step: None,
        };
        let cache_key = format!("flow_kpi:{}:24h", flow_id);
        cache.insert(cache_key, Arc::new(cached_kpi)).await;

        let uc = GetFlowKpiUseCase::new(Arc::new(def_mock), Arc::new(inst_mock)).with_cache(cache);
        let result = uc.execute(&flow_id, "24h").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.kpi.total_started, 10);
        assert_eq!(output.kpi.total_completed, 9);
    }

    #[tokio::test]
    async fn not_found() {
        let mut def_mock = MockFlowDefinitionRepository::new();
        def_mock.expect_find_by_id().returning(|_| Ok(None));

        let mut inst_mock = MockFlowInstanceRepository::new();
        inst_mock
            .expect_find_by_flow_id_paginated()
            .returning(|_, _, _| Ok((vec![], 0)));

        let uc = GetFlowKpiUseCase::new(Arc::new(def_mock), Arc::new(inst_mock));
        let result = uc.execute(&Uuid::new_v4(), "24h").await;
        assert!(matches!(result, Err(GetFlowKpiError::NotFound(_))));
    }
}
