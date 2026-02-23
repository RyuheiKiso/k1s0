//! テスト用インメモリリポジトリとヘルパー。
//! 統合テスト（tests/integration_test.rs）から利用する。

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::adapter::handler::AppState;
use crate::domain::entity::saga_state::{SagaState, SagaStatus};
use crate::domain::entity::saga_step_log::SagaStepLog;
use crate::domain::repository::saga_repository::SagaListParams;
use crate::domain::repository::{SagaRepository, WorkflowRepository};
use crate::infrastructure::grpc_caller::GrpcStepCaller;
use crate::infrastructure::kafka_producer::SagaEventPublisher;

// ---------------------------------------------------------------------------
// InMemorySagaRepository
// ---------------------------------------------------------------------------

/// テスト用インメモリ SagaRepository 実装。
pub struct InMemorySagaRepository {
    states: RwLock<Vec<SagaState>>,
    step_logs: RwLock<Vec<SagaStepLog>>,
}

impl InMemorySagaRepository {
    pub fn new() -> Self {
        Self {
            states: RwLock::new(Vec::new()),
            step_logs: RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemorySagaRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SagaRepository for InMemorySagaRepository {
    async fn create(&self, state: &SagaState) -> anyhow::Result<()> {
        self.states.write().await.push(state.clone());
        Ok(())
    }

    async fn update_with_step_log(
        &self,
        state: &SagaState,
        log: &SagaStepLog,
    ) -> anyhow::Result<()> {
        let mut states = self.states.write().await;
        if let Some(s) = states.iter_mut().find(|s| s.saga_id == state.saga_id) {
            *s = state.clone();
        }
        self.step_logs.write().await.push(log.clone());
        Ok(())
    }

    async fn update_status(
        &self,
        saga_id: uuid::Uuid,
        status: &SagaStatus,
        error_message: Option<String>,
    ) -> anyhow::Result<()> {
        let mut states = self.states.write().await;
        if let Some(s) = states.iter_mut().find(|s| s.saga_id == saga_id) {
            s.status = status.clone();
            s.error_message = error_message;
            s.updated_at = chrono::Utc::now();
        }
        Ok(())
    }

    async fn find_by_id(&self, saga_id: uuid::Uuid) -> anyhow::Result<Option<SagaState>> {
        let states = self.states.read().await;
        Ok(states.iter().find(|s| s.saga_id == saga_id).cloned())
    }

    async fn find_step_logs(&self, saga_id: uuid::Uuid) -> anyhow::Result<Vec<SagaStepLog>> {
        let logs = self.step_logs.read().await;
        Ok(logs
            .iter()
            .filter(|l| l.saga_id == saga_id)
            .cloned()
            .collect())
    }

    async fn list(&self, params: &SagaListParams) -> anyhow::Result<(Vec<SagaState>, i64)> {
        let states = self.states.read().await;
        let filtered: Vec<_> = states
            .iter()
            .filter(|s| {
                if let Some(ref wn) = params.workflow_name {
                    if s.workflow_name != *wn {
                        return false;
                    }
                }
                if let Some(ref st) = params.status {
                    if s.status != *st {
                        return false;
                    }
                }
                if let Some(ref ci) = params.correlation_id {
                    if s.correlation_id.as_deref() != Some(ci.as_str()) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let total = filtered.len() as i64;
        let page = params.page.max(1);
        let page_size = params.page_size.max(1);
        let offset = ((page - 1) * page_size) as usize;
        let limit = page_size as usize;
        let paged: Vec<_> = filtered.into_iter().skip(offset).take(limit).collect();

        Ok((paged, total))
    }

    async fn find_incomplete(&self) -> anyhow::Result<Vec<SagaState>> {
        let states = self.states.read().await;
        Ok(states
            .iter()
            .filter(|s| {
                matches!(
                    s.status,
                    SagaStatus::Started | SagaStatus::Running | SagaStatus::Compensating
                )
            })
            .cloned()
            .collect())
    }
}

// ---------------------------------------------------------------------------
// NoOpGrpcCaller
// ---------------------------------------------------------------------------

/// テスト用 NoOp GrpcStepCaller 実装。常に成功レスポンスを返す。
pub struct NoOpGrpcCaller;

#[async_trait]
impl GrpcStepCaller for NoOpGrpcCaller {
    async fn call_step(
        &self,
        _service_name: &str,
        _method: &str,
        _payload: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({"status": "ok"}))
    }
}

// ---------------------------------------------------------------------------
// NoOpPublisher
// ---------------------------------------------------------------------------

/// テスト用 NoOp SagaEventPublisher 実装。
pub struct NoOpPublisher;

#[async_trait]
impl SagaEventPublisher for NoOpPublisher {
    async fn publish_saga_event(
        &self,
        _saga_id: &str,
        _event_type: &str,
        _payload: &serde_json::Value,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// make_test_app_state
// ---------------------------------------------------------------------------

/// テスト用 AppState を構築するヘルパー。
pub fn make_test_app_state(
    saga_repo: Arc<dyn SagaRepository>,
    workflow_repo: Arc<dyn WorkflowRepository>,
) -> AppState {
    let grpc_caller: Arc<dyn GrpcStepCaller> = Arc::new(NoOpGrpcCaller);
    let publisher: Option<Arc<dyn SagaEventPublisher>> = None;

    let execute_saga_uc = Arc::new(crate::usecase::ExecuteSagaUseCase::new(
        saga_repo.clone(),
        grpc_caller,
        publisher,
    ));

    let start_saga_uc = Arc::new(crate::usecase::StartSagaUseCase::new(
        saga_repo.clone(),
        workflow_repo.clone(),
        execute_saga_uc,
    ));
    let get_saga_uc = Arc::new(crate::usecase::GetSagaUseCase::new(saga_repo.clone()));
    let list_sagas_uc = Arc::new(crate::usecase::ListSagasUseCase::new(saga_repo.clone()));
    let cancel_saga_uc = Arc::new(crate::usecase::CancelSagaUseCase::new(saga_repo.clone()));
    let register_workflow_uc = Arc::new(crate::usecase::RegisterWorkflowUseCase::new(
        workflow_repo.clone(),
    ));
    let list_workflows_uc = Arc::new(crate::usecase::ListWorkflowsUseCase::new(workflow_repo));

    AppState {
        start_saga_uc,
        get_saga_uc,
        list_sagas_uc,
        cancel_saga_uc,
        register_workflow_uc,
        list_workflows_uc,
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("test")),
    }
}
