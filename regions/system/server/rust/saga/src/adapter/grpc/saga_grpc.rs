use std::sync::Arc;

use crate::domain::entity::saga_state::SagaStatus;
use crate::domain::repository::saga_repository::SagaListParams;
use crate::usecase::{
    CancelSagaUseCase, GetSagaUseCase, ListSagasUseCase, ListWorkflowsUseCase,
    RegisterWorkflowUseCase, StartSagaUseCase,
};

// --- Proto 手動型定義 ---

/// StartSagaRequest はSaga開始リクエスト。
#[derive(Debug)]
pub struct StartSagaRequest {
    pub workflow_name: String,
    pub payload: Vec<u8>,
    pub correlation_id: String,
    pub initiated_by: String,
}

/// StartSagaResponse はSaga開始レスポンス。
#[derive(Debug)]
pub struct StartSagaResponse {
    pub saga_id: String,
    pub status: String,
}

/// GetSagaRequest はSaga取得リクエスト。
#[derive(Debug)]
pub struct GetSagaRequest {
    pub saga_id: String,
}

/// GetSagaResponse はSaga取得レスポンス。
#[derive(Debug)]
pub struct GetSagaResponse {
    pub saga: SagaStateProto,
    pub step_logs: Vec<SagaStepLogProto>,
}

/// ListSagasRequest はSaga一覧リクエスト。
#[derive(Debug)]
pub struct ListSagasRequest {
    pub page: i32,
    pub page_size: i32,
    pub workflow_name: String,
    pub status: String,
    pub correlation_id: String,
}

/// ListSagasResponse はSaga一覧レスポンス。
#[derive(Debug)]
pub struct ListSagasResponse {
    pub sagas: Vec<SagaStateProto>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

/// CancelSagaRequest はSagaキャンセルリクエスト。
#[derive(Debug)]
pub struct CancelSagaRequest {
    pub saga_id: String,
}

/// CancelSagaResponse はSagaキャンセルレスポンス。
#[derive(Debug)]
pub struct CancelSagaResponse {
    pub success: bool,
    pub message: String,
}

/// RegisterWorkflowRequest はワークフロー登録リクエスト。
#[derive(Debug)]
pub struct RegisterWorkflowRequest {
    pub workflow_yaml: String,
}

/// RegisterWorkflowResponse はワークフロー登録レスポンス。
#[derive(Debug)]
pub struct RegisterWorkflowResponse {
    pub name: String,
    pub step_count: i32,
}

/// ListWorkflowsRequest はワークフロー一覧リクエスト。
#[derive(Debug)]
pub struct ListWorkflowsRequest {}

/// ListWorkflowsResponse はワークフロー一覧レスポンス。
#[derive(Debug)]
pub struct ListWorkflowsResponse {
    pub workflows: Vec<WorkflowSummary>,
}

/// WorkflowSummary はワークフローの概要情報。
#[derive(Debug)]
pub struct WorkflowSummary {
    pub name: String,
    pub step_count: i32,
    pub step_names: Vec<String>,
}

/// SagaStateProto はSaga状態のProto相当型。
#[derive(Debug)]
pub struct SagaStateProto {
    pub id: String,
    pub workflow_name: String,
    pub current_step: i32,
    pub status: String,
    pub payload: Vec<u8>,
    pub correlation_id: String,
    pub initiated_by: String,
    pub error_message: String,
    pub created_at: String,
    pub updated_at: String,
}

/// SagaStepLogProto はSagaステップログのProto相当型。
#[derive(Debug)]
pub struct SagaStepLogProto {
    pub id: String,
    pub saga_id: String,
    pub step_index: i32,
    pub step_name: String,
    pub action: String,
    pub status: String,
    pub request_payload: Vec<u8>,
    pub response_payload: Vec<u8>,
    pub error_message: String,
    pub started_at: String,
    pub completed_at: String,
}

// --- GrpcError ---

/// GrpcError はgRPC層のエラー型。
#[derive(Debug)]
pub enum GrpcError {
    NotFound(String),
    InvalidArgument(String),
    Internal(String),
}

// --- SagaGrpcService ---

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

    /// Saga開始。
    pub async fn start_saga(
        &self,
        req: StartSagaRequest,
    ) -> Result<StartSagaResponse, GrpcError> {
        let json_payload: serde_json::Value = if req.payload.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&req.payload)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid payload: {}", e)))?
        };

        let correlation = if req.correlation_id.is_empty() {
            None
        } else {
            Some(req.correlation_id)
        };
        let initiator = if req.initiated_by.is_empty() {
            None
        } else {
            Some(req.initiated_by)
        };

        let saga_id = self
            .start_saga_uc
            .execute(req.workflow_name, json_payload, correlation, initiator)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("not found") {
                    GrpcError::NotFound(msg)
                } else {
                    GrpcError::Internal(format!("failed to start saga: {}", msg))
                }
            })?;

        Ok(StartSagaResponse {
            saga_id: saga_id.to_string(),
            status: "STARTED".to_string(),
        })
    }

    /// Saga取得。
    pub async fn get_saga(&self, req: GetSagaRequest) -> Result<GetSagaResponse, GrpcError> {
        let id = uuid::Uuid::parse_str(&req.saga_id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid saga_id: {}", req.saga_id)))?;

        let (saga, step_logs) = self
            .get_saga_uc
            .execute(id)
            .await
            .map_err(|e| GrpcError::Internal(format!("failed to get saga: {}", e)))?
            .ok_or_else(|| GrpcError::NotFound(format!("saga not found: {}", req.saga_id)))?;

        let saga_proto = SagaStateProto {
            id: saga.saga_id.to_string(),
            workflow_name: saga.workflow_name.clone(),
            current_step: saga.current_step,
            status: saga.status.to_string(),
            payload: serde_json::to_vec(&saga.payload)
                .unwrap_or_default(),
            correlation_id: saga.correlation_id.unwrap_or_default(),
            initiated_by: saga.initiated_by.unwrap_or_default(),
            error_message: saga.error_message.unwrap_or_default(),
            created_at: saga.created_at.to_rfc3339(),
            updated_at: saga.updated_at.to_rfc3339(),
        };

        let step_log_protos = step_logs
            .into_iter()
            .map(|log| SagaStepLogProto {
                id: log.id.to_string(),
                saga_id: log.saga_id.to_string(),
                step_index: log.step_index,
                step_name: log.step_name.clone(),
                action: log.action.to_string(),
                status: log.status.to_string(),
                request_payload: log
                    .request_payload
                    .as_ref()
                    .and_then(|v| serde_json::to_vec(v).ok())
                    .unwrap_or_default(),
                response_payload: log
                    .response_payload
                    .as_ref()
                    .and_then(|v| serde_json::to_vec(v).ok())
                    .unwrap_or_default(),
                error_message: log.error_message.unwrap_or_default(),
                started_at: log.started_at.to_rfc3339(),
                completed_at: log
                    .completed_at
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_default(),
            })
            .collect();

        Ok(GetSagaResponse {
            saga: saga_proto,
            step_logs: step_log_protos,
        })
    }

    /// Saga一覧取得。
    pub async fn list_sagas(
        &self,
        req: ListSagasRequest,
    ) -> Result<ListSagasResponse, GrpcError> {
        let status_filter = if req.status.is_empty() {
            None
        } else {
            let s = SagaStatus::from_str_value(&req.status).map_err(|_| {
                GrpcError::InvalidArgument(format!("invalid status: {}", req.status))
            })?;
            Some(s)
        };

        let params = SagaListParams {
            page: req.page,
            page_size: req.page_size,
            workflow_name: if req.workflow_name.is_empty() {
                None
            } else {
                Some(req.workflow_name)
            },
            status: status_filter,
            correlation_id: if req.correlation_id.is_empty() {
                None
            } else {
                Some(req.correlation_id)
            },
        };

        let (sagas, total_count) = self
            .list_sagas_uc
            .execute(params)
            .await
            .map_err(|e| GrpcError::Internal(format!("failed to list sagas: {}", e)))?;

        let page = req.page;
        let page_size = req.page_size;
        let has_next = (page as i64 * page_size as i64) < total_count;

        let saga_protos = sagas
            .into_iter()
            .map(|saga| SagaStateProto {
                id: saga.saga_id.to_string(),
                workflow_name: saga.workflow_name.clone(),
                current_step: saga.current_step,
                status: saga.status.to_string(),
                payload: serde_json::to_vec(&saga.payload).unwrap_or_default(),
                correlation_id: saga.correlation_id.unwrap_or_default(),
                initiated_by: saga.initiated_by.unwrap_or_default(),
                error_message: saga.error_message.unwrap_or_default(),
                created_at: saga.created_at.to_rfc3339(),
                updated_at: saga.updated_at.to_rfc3339(),
            })
            .collect();

        Ok(ListSagasResponse {
            sagas: saga_protos,
            total_count,
            page,
            page_size,
            has_next,
        })
    }

    /// Sagaキャンセル。
    pub async fn cancel_saga(
        &self,
        req: CancelSagaRequest,
    ) -> Result<CancelSagaResponse, GrpcError> {
        let id = uuid::Uuid::parse_str(&req.saga_id).map_err(|_| {
            GrpcError::InvalidArgument(format!("invalid saga_id: {}", req.saga_id))
        })?;

        self.cancel_saga_uc
            .execute(id)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("not found") {
                    GrpcError::NotFound(msg)
                } else if msg.contains("terminal") {
                    GrpcError::InvalidArgument(msg)
                } else {
                    GrpcError::Internal(format!("failed to cancel saga: {}", msg))
                }
            })?;

        Ok(CancelSagaResponse {
            success: true,
            message: format!("saga {} cancelled successfully", req.saga_id),
        })
    }

    /// ワークフロー登録。
    pub async fn register_workflow(
        &self,
        req: RegisterWorkflowRequest,
    ) -> Result<RegisterWorkflowResponse, GrpcError> {
        if req.workflow_yaml.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "workflow_yaml must not be empty".to_string(),
            ));
        }

        let (name, step_count) = self
            .register_workflow_uc
            .execute(req.workflow_yaml)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("name") || msg.contains("step") || msg.contains("service") || msg.contains("method") {
                    GrpcError::InvalidArgument(format!("invalid workflow definition: {}", msg))
                } else {
                    GrpcError::Internal(format!("failed to register workflow: {}", msg))
                }
            })?;

        Ok(RegisterWorkflowResponse {
            name,
            step_count: step_count as i32,
        })
    }

    /// ワークフロー一覧取得。
    pub async fn list_workflows(
        &self,
        _req: ListWorkflowsRequest,
    ) -> Result<ListWorkflowsResponse, GrpcError> {
        let workflows = self
            .list_workflows_uc
            .execute()
            .await
            .map_err(|e| GrpcError::Internal(format!("failed to list workflows: {}", e)))?;

        let summaries = workflows
            .into_iter()
            .map(|wf| {
                let step_names: Vec<String> = wf.steps.iter().map(|s| s.name.clone()).collect();
                let step_count = step_names.len() as i32;
                WorkflowSummary {
                    name: wf.name,
                    step_count,
                    step_names,
                }
            })
            .collect();

        Ok(ListWorkflowsResponse {
            workflows: summaries,
        })
    }
}
