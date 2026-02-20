use std::sync::Arc;

use crate::usecase::{
    CancelSagaUseCase, GetSagaUseCase, ListSagasUseCase, ListWorkflowsUseCase,
    RegisterWorkflowUseCase, StartSagaUseCase,
};

/// SagaGrpcService はgRPC Sagaサービスの実装。
pub struct SagaGrpcService {
    pub start_saga_uc: Arc<StartSagaUseCase>,
    pub get_saga_uc: Arc<GetSagaUseCase>,
    pub list_sagas_uc: Arc<ListSagasUseCase>,
    pub cancel_saga_uc: Arc<CancelSagaUseCase>,
    pub register_workflow_uc: Arc<RegisterWorkflowUseCase>,
    pub list_workflows_uc: Arc<ListWorkflowsUseCase>,
}

impl SagaGrpcService {
    pub fn new(
        start_saga_uc: Arc<StartSagaUseCase>,
        get_saga_uc: Arc<GetSagaUseCase>,
        list_sagas_uc: Arc<ListSagasUseCase>,
        cancel_saga_uc: Arc<CancelSagaUseCase>,
        register_workflow_uc: Arc<RegisterWorkflowUseCase>,
        list_workflows_uc: Arc<ListWorkflowsUseCase>,
    ) -> Self {
        Self {
            start_saga_uc,
            get_saga_uc,
            list_sagas_uc,
            cancel_saga_uc,
            register_workflow_uc,
            list_workflows_uc,
        }
    }

    /// Saga開始（proto生成後にtonic traitに接続）。
    pub async fn start_saga(
        &self,
        workflow_name: String,
        payload: Vec<u8>,
        correlation_id: String,
        initiated_by: String,
    ) -> Result<(String, String), String> {
        let json_payload: serde_json::Value = if payload.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&payload).map_err(|e| format!("invalid payload: {}", e))?
        };

        let correlation = if correlation_id.is_empty() {
            None
        } else {
            Some(correlation_id)
        };
        let initiator = if initiated_by.is_empty() {
            None
        } else {
            Some(initiated_by)
        };

        let saga_id = self
            .start_saga_uc
            .execute(workflow_name, json_payload, correlation, initiator)
            .await
            .map_err(|e| format!("failed to start saga: {}", e))?;

        Ok((saga_id.to_string(), "STARTED".to_string()))
    }

    /// Saga取得。
    pub async fn get_saga(&self, saga_id: &str) -> Result<(), String> {
        let id =
            uuid::Uuid::parse_str(saga_id).map_err(|_| format!("invalid saga_id: {}", saga_id))?;

        self.get_saga_uc
            .execute(id)
            .await
            .map_err(|e| format!("failed to get saga: {}", e))?;

        Ok(())
    }
}
