use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::usecase::create_workflow::{CreateWorkflowError, CreateWorkflowInput};
use crate::usecase::get_workflow::{GetWorkflowError, GetWorkflowInput};
use crate::usecase::list_workflows::{ListWorkflowsError, ListWorkflowsInput};
use crate::usecase::start_instance::{StartInstanceError, StartInstanceInput};
use crate::usecase::get_instance::{GetInstanceError, GetInstanceInput};
use crate::usecase::{
    CreateWorkflowUseCase, GetInstanceUseCase, GetWorkflowUseCase, ListWorkflowsUseCase,
    StartInstanceUseCase,
};

#[derive(Clone)]
pub struct AppState {
    pub create_workflow_uc: Arc<CreateWorkflowUseCase>,
    pub get_workflow_uc: Arc<GetWorkflowUseCase>,
    pub list_workflows_uc: Arc<ListWorkflowsUseCase>,
    pub start_instance_uc: Arc<StartInstanceUseCase>,
    pub get_instance_uc: Arc<GetInstanceUseCase>,
}

// --- Request / Response DTOs ---

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub steps: Vec<StepRequest>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct StepRequest {
    pub step_id: String,
    pub name: String,
    pub step_type: String,
    pub assignee_role: Option<String>,
    pub timeout_hours: Option<u32>,
    pub on_approve: Option<String>,
    pub on_reject: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkflowResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub enabled: bool,
    pub step_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListWorkflowsQuery {
    #[serde(default = "default_false")]
    pub enabled_only: bool,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_false() -> bool {
    false
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

#[derive(Debug, Serialize)]
pub struct ListWorkflowsResponse {
    pub workflows: Vec<WorkflowResponse>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteWorkflowRequest {
    pub title: String,
    pub initiator_id: String,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ExecuteWorkflowResponse {
    pub instance_id: String,
    pub status: String,
    pub current_step_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InstanceStatusResponse {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub title: String,
    pub status: String,
    pub current_step_id: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

// --- Handlers ---

/// POST /api/v1/workflows
pub async fn create_workflow(
    State(state): State<AppState>,
    Json(req): Json<CreateWorkflowRequest>,
) -> impl IntoResponse {
    use crate::domain::entity::workflow_step::WorkflowStep;

    let steps: Vec<WorkflowStep> = req
        .steps
        .into_iter()
        .map(|s| WorkflowStep::new(
            s.step_id,
            s.name,
            s.step_type,
            s.assignee_role,
            s.timeout_hours,
            s.on_approve,
            s.on_reject,
        ))
        .collect();

    let input = CreateWorkflowInput {
        name: req.name,
        description: req.description,
        enabled: req.enabled,
        steps,
    };

    match state.create_workflow_uc.execute(&input).await {
        Ok(def) => {
            let step_count = def.step_count();
            let resp = WorkflowResponse {
                id: def.id,
                name: def.name,
                description: def.description,
                version: def.version,
                enabled: def.enabled,
                step_count,
                created_at: def.created_at.to_rfc3339(),
                updated_at: def.updated_at.to_rfc3339(),
            };
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(resp).unwrap()),
            )
                .into_response()
        }
        Err(CreateWorkflowError::AlreadyExists(name)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": format!("workflow already exists: {}", name)})),
        )
            .into_response(),
        Err(CreateWorkflowError::Validation(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
        Err(CreateWorkflowError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// GET /api/v1/workflows/:id
pub async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let input = GetWorkflowInput { id: id.clone() };

    match state.get_workflow_uc.execute(&input).await {
        Ok(def) => {
            let step_count = def.step_count();
            let resp = WorkflowResponse {
                id: def.id,
                name: def.name,
                description: def.description,
                version: def.version,
                enabled: def.enabled,
                step_count,
                created_at: def.created_at.to_rfc3339(),
                updated_at: def.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(GetWorkflowError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("workflow not found: {}", id)})),
        )
            .into_response(),
        Err(GetWorkflowError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// GET /api/v1/workflows
pub async fn list_workflows(
    State(state): State<AppState>,
    Query(query): Query<ListWorkflowsQuery>,
) -> impl IntoResponse {
    let input = ListWorkflowsInput {
        enabled_only: query.enabled_only,
        page: query.page,
        page_size: query.page_size,
    };

    match state.list_workflows_uc.execute(&input).await {
        Ok(output) => {
            let resp = ListWorkflowsResponse {
                workflows: output
                    .workflows
                    .into_iter()
                    .map(|def| {
                        let step_count = def.step_count();
                        WorkflowResponse {
                            id: def.id,
                            name: def.name,
                            description: def.description,
                            version: def.version,
                            enabled: def.enabled,
                            step_count,
                            created_at: def.created_at.to_rfc3339(),
                            updated_at: def.updated_at.to_rfc3339(),
                        }
                    })
                    .collect(),
                total_count: output.total_count,
                page: output.page,
                page_size: output.page_size,
                has_next: output.has_next,
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(ListWorkflowsError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// POST /api/v1/workflows/:id/execute
pub async fn execute_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ExecuteWorkflowRequest>,
) -> impl IntoResponse {
    let input = StartInstanceInput {
        workflow_id: id.clone(),
        title: req.title,
        initiator_id: req.initiator_id,
        context: req.context,
    };

    match state.start_instance_uc.execute(&input).await {
        Ok(output) => {
            let resp = ExecuteWorkflowResponse {
                instance_id: output.instance.id,
                status: output.instance.status,
                current_step_id: output.instance.current_step_id,
            };
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(resp).unwrap()),
            )
                .into_response()
        }
        Err(StartInstanceError::WorkflowNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("workflow not found: {}", id)})),
        )
            .into_response(),
        Err(StartInstanceError::WorkflowDisabled(_)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": format!("workflow is disabled: {}", id)})),
        )
            .into_response(),
        Err(StartInstanceError::NoSteps(_)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("workflow has no steps: {}", id)})),
        )
            .into_response(),
        Err(StartInstanceError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// GET /api/v1/workflows/:id/status
pub async fn get_workflow_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // This returns instance status. The :id here is the instance id.
    let input = GetInstanceInput { id: id.clone() };

    match state.get_instance_uc.execute(&input).await {
        Ok(inst) => {
            let resp = InstanceStatusResponse {
                id: inst.id,
                workflow_id: inst.workflow_id,
                workflow_name: inst.workflow_name,
                title: inst.title,
                status: inst.status,
                current_step_id: inst.current_step_id,
                started_at: inst.started_at.to_rfc3339(),
                completed_at: inst.completed_at.map(|t| t.to_rfc3339()),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(GetInstanceError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("instance not found: {}", id)})),
        )
            .into_response(),
        Err(GetInstanceError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}
