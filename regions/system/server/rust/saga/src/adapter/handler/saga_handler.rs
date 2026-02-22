use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::SagaError;
use super::AppState;
use crate::domain::entity::saga_state::SagaStatus;
use crate::domain::repository::saga_repository::SagaListParams;
use crate::usecase::CancelSagaError;

// --- Request / Response DTOs ---

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct StartSagaRequest {
    pub workflow_name: String,
    #[serde(default)]
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
    pub initiated_by: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StartSagaResponse {
    pub saga_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ListSagasQuery {
    pub workflow_name: Option<String>,
    pub status: Option<String>,
    pub correlation_id: Option<String>,
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_page_size")]
    pub page_size: i32,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SagaResponse {
    pub saga_id: String,
    pub workflow_name: String,
    pub current_step: i32,
    pub status: String,
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
    pub initiated_by: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SagaDetailResponse {
    pub saga: SagaResponse,
    pub step_logs: Vec<StepLogResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StepLogResponse {
    pub id: String,
    pub step_index: i32,
    pub step_name: String,
    pub action: String,
    pub status: String,
    pub request_payload: Option<serde_json::Value>,
    pub response_payload: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ListSagasResponse {
    pub sagas: Vec<SagaResponse>,
    pub pagination: PaginationResponse,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginationResponse {
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RegisterWorkflowRequest {
    pub workflow_yaml: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RegisterWorkflowResponse {
    pub name: String,
    pub step_count: usize,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkflowSummaryResponse {
    pub name: String,
    pub step_count: usize,
    pub step_names: Vec<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ListWorkflowsResponse {
    pub workflows: Vec<WorkflowSummaryResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CancelSagaResponse {
    pub success: bool,
    pub message: String,
}

// --- Handlers ---

#[utoipa::path(get, path = "/healthz", responses((status = 200, description = "Health check OK")))]
pub async fn healthz() -> &'static str {
    "ok"
}

#[utoipa::path(get, path = "/readyz", responses((status = 200, description = "Ready")))]
pub async fn readyz() -> &'static str {
    "ok"
}

#[utoipa::path(get, path = "/metrics", responses((status = 200, description = "Prometheus metrics")))]
pub async fn metrics(State(state): State<AppState>) -> String {
    state.metrics.gather_metrics()
}

#[utoipa::path(
    post,
    path = "/api/v1/sagas",
    request_body = StartSagaRequest,
    responses(
        (status = 201, description = "Saga started", body = StartSagaResponse),
        (status = 400, description = "Validation error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn start_saga(
    State(state): State<AppState>,
    Json(req): Json<StartSagaRequest>,
) -> Result<(StatusCode, Json<StartSagaResponse>), SagaError> {
    if req.workflow_name.is_empty() {
        return Err(SagaError::Validation(
            "workflow_name is required".to_string(),
        ));
    }

    let saga_id = state
        .start_saga_uc
        .execute(
            req.workflow_name,
            req.payload,
            req.correlation_id,
            req.initiated_by,
        )
        .await
        .map_err(|e| SagaError::Internal(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(StartSagaResponse {
            saga_id: saga_id.to_string(),
            status: "STARTED".to_string(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/sagas",
    params(
        ("workflow_name" = Option<String>, Query, description = "Filter by workflow"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("page" = Option<i32>, Query, description = "Page number"),
        ("page_size" = Option<i32>, Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "Saga list", body = ListSagasResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_sagas(
    State(state): State<AppState>,
    Query(query): Query<ListSagasQuery>,
) -> Result<Json<ListSagasResponse>, SagaError> {
    let status = if let Some(ref s) = query.status {
        Some(SagaStatus::from_str_value(s).map_err(|e| SagaError::Validation(e.to_string()))?)
    } else {
        None
    };

    let params = SagaListParams {
        workflow_name: query.workflow_name,
        status,
        correlation_id: query.correlation_id,
        page: query.page,
        page_size: query.page_size,
    };

    let (sagas, total) = state
        .list_sagas_uc
        .execute(params)
        .await
        .map_err(|e| SagaError::Internal(e.to_string()))?;

    let saga_responses: Vec<SagaResponse> = sagas
        .into_iter()
        .map(|s| SagaResponse {
            saga_id: s.saga_id.to_string(),
            workflow_name: s.workflow_name,
            current_step: s.current_step,
            status: s.status.to_string(),
            payload: s.payload,
            correlation_id: s.correlation_id,
            initiated_by: s.initiated_by,
            error_message: s.error_message,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        })
        .collect();

    let has_next = (query.page as i64 * query.page_size as i64) < total;

    Ok(Json(ListSagasResponse {
        sagas: saga_responses,
        pagination: PaginationResponse {
            total_count: total,
            page: query.page,
            page_size: query.page_size,
            has_next,
        },
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/sagas/{saga_id}",
    params(("saga_id" = String, Path, description = "Saga ID")),
    responses(
        (status = 200, description = "Saga detail", body = SagaDetailResponse),
        (status = 404, description = "Saga not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_saga(
    State(state): State<AppState>,
    Path(saga_id): Path<String>,
) -> Result<Json<SagaDetailResponse>, SagaError> {
    let id = Uuid::parse_str(&saga_id)
        .map_err(|_| SagaError::Validation(format!("invalid saga_id: {}", saga_id)))?;

    let (saga, step_logs) = state
        .get_saga_uc
        .execute(id)
        .await
        .map_err(|e| SagaError::Internal(e.to_string()))?
        .ok_or_else(|| SagaError::NotFound(format!("saga not found: {}", saga_id)))?;

    let step_log_responses: Vec<StepLogResponse> = step_logs
        .into_iter()
        .map(|l| StepLogResponse {
            id: l.id.to_string(),
            step_index: l.step_index,
            step_name: l.step_name,
            action: l.action.to_string(),
            status: l.status.to_string(),
            request_payload: l.request_payload,
            response_payload: l.response_payload,
            error_message: l.error_message,
            started_at: l.started_at.to_rfc3339(),
            completed_at: l.completed_at.map(|t| t.to_rfc3339()),
        })
        .collect();

    Ok(Json(SagaDetailResponse {
        saga: SagaResponse {
            saga_id: saga.saga_id.to_string(),
            workflow_name: saga.workflow_name,
            current_step: saga.current_step,
            status: saga.status.to_string(),
            payload: saga.payload,
            correlation_id: saga.correlation_id,
            initiated_by: saga.initiated_by,
            error_message: saga.error_message,
            created_at: saga.created_at.to_rfc3339(),
            updated_at: saga.updated_at.to_rfc3339(),
        },
        step_logs: step_log_responses,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/sagas/{saga_id}/cancel",
    params(("saga_id" = String, Path, description = "Saga ID")),
    responses(
        (status = 200, description = "Saga cancelled", body = CancelSagaResponse),
        (status = 404, description = "Saga not found"),
        (status = 409, description = "Already terminal"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn cancel_saga(
    State(state): State<AppState>,
    Path(saga_id): Path<String>,
) -> Result<Json<CancelSagaResponse>, SagaError> {
    let id = Uuid::parse_str(&saga_id)
        .map_err(|_| SagaError::Validation(format!("invalid saga_id: {}", saga_id)))?;

    state.cancel_saga_uc.execute(id).await.map_err(|e| match e {
        CancelSagaError::NotFound(_) => SagaError::NotFound(e.to_string()),
        CancelSagaError::AlreadyTerminal(_) => SagaError::Conflict(e.to_string()),
        CancelSagaError::Internal(_) => SagaError::Internal(e.to_string()),
    })?;

    Ok(Json(CancelSagaResponse {
        success: true,
        message: format!("saga {} cancelled", saga_id),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/workflows",
    request_body = RegisterWorkflowRequest,
    responses(
        (status = 201, description = "Workflow registered", body = RegisterWorkflowResponse),
        (status = 400, description = "Validation error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn register_workflow(
    State(state): State<AppState>,
    Json(req): Json<RegisterWorkflowRequest>,
) -> Result<(StatusCode, Json<RegisterWorkflowResponse>), SagaError> {
    if req.workflow_yaml.is_empty() {
        return Err(SagaError::Validation(
            "workflow_yaml is required".to_string(),
        ));
    }

    let (name, step_count) = state
        .register_workflow_uc
        .execute(req.workflow_yaml)
        .await
        .map_err(|e| SagaError::Validation(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterWorkflowResponse { name, step_count }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows",
    responses((status = 200, description = "Workflow list", body = ListWorkflowsResponse)),
    security(("bearer_auth" = []))
)]
pub async fn list_workflows(
    State(state): State<AppState>,
) -> Result<Json<ListWorkflowsResponse>, SagaError> {
    let workflows = state
        .list_workflows_uc
        .execute()
        .await
        .map_err(|e| SagaError::Internal(e.to_string()))?;

    let summaries: Vec<WorkflowSummaryResponse> = workflows
        .into_iter()
        .map(|w| WorkflowSummaryResponse {
            step_count: w.steps.len(),
            step_names: w.steps.iter().map(|s| s.name.clone()).collect(),
            name: w.name,
        })
        .collect();

    Ok(Json(ListWorkflowsResponse {
        workflows: summaries,
    }))
}
