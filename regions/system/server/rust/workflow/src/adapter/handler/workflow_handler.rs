use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::usecase::create_workflow::{CreateWorkflowError, CreateWorkflowInput};
use crate::usecase::update_workflow::{UpdateWorkflowError, UpdateWorkflowInput};
use crate::usecase::delete_workflow::{DeleteWorkflowError, DeleteWorkflowInput};
use crate::usecase::get_workflow::{GetWorkflowError, GetWorkflowInput};
use crate::usecase::list_workflows::{ListWorkflowsError, ListWorkflowsInput};
use crate::usecase::start_instance::{StartInstanceError, StartInstanceInput};
use crate::usecase::get_instance::{GetInstanceError, GetInstanceInput};
use crate::usecase::list_instances::{ListInstancesError, ListInstancesInput};
use crate::usecase::cancel_instance::{CancelInstanceError, CancelInstanceInput};
use crate::usecase::list_tasks::{ListTasksError, ListTasksInput};
use crate::usecase::approve_task::{ApproveTaskError, ApproveTaskInput};
use crate::usecase::reject_task::{RejectTaskError, RejectTaskInput};
use crate::usecase::reassign_task::{ReassignTaskError, ReassignTaskInput};
use crate::adapter::middleware::auth::WorkflowAuthState;
use crate::usecase::{
    ApproveTaskUseCase, CancelInstanceUseCase, CreateWorkflowUseCase, DeleteWorkflowUseCase,
    GetInstanceUseCase, GetWorkflowUseCase, ListInstancesUseCase, ListTasksUseCase,
    ListWorkflowsUseCase, ReassignTaskUseCase, RejectTaskUseCase, StartInstanceUseCase,
    UpdateWorkflowUseCase,
};

#[derive(Clone)]
pub struct AppState {
    pub create_workflow_uc: Arc<CreateWorkflowUseCase>,
    pub update_workflow_uc: Arc<UpdateWorkflowUseCase>,
    pub delete_workflow_uc: Arc<DeleteWorkflowUseCase>,
    pub get_workflow_uc: Arc<GetWorkflowUseCase>,
    pub list_workflows_uc: Arc<ListWorkflowsUseCase>,
    pub start_instance_uc: Arc<StartInstanceUseCase>,
    pub get_instance_uc: Arc<GetInstanceUseCase>,
    pub list_instances_uc: Arc<ListInstancesUseCase>,
    pub cancel_instance_uc: Arc<CancelInstanceUseCase>,
    pub list_tasks_uc: Arc<ListTasksUseCase>,
    pub approve_task_uc: Arc<ApproveTaskUseCase>,
    pub reject_task_uc: Arc<RejectTaskUseCase>,
    pub reassign_task_uc: Arc<ReassignTaskUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<WorkflowAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: WorkflowAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
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

#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    #[serde(default)]
    pub steps: Option<Vec<StepRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct ListInstancesQuery {
    pub status: Option<String>,
    pub workflow_id: Option<String>,
    pub initiator_id: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

#[derive(Debug, Deserialize)]
pub struct CancelInstanceRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub assignee_id: Option<String>,
    pub status: Option<String>,
    pub instance_id: Option<String>,
    #[serde(default = "default_false")]
    pub overdue_only: bool,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

#[derive(Debug, Deserialize)]
pub struct ApproveTaskRequest {
    pub actor_id: String,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RejectTaskRequest {
    pub actor_id: String,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReassignTaskRequest {
    pub new_assignee_id: String,
    pub reason: Option<String>,
    pub actor_id: String,
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

/// PUT /api/v1/workflows/:id
pub async fn update_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateWorkflowRequest>,
) -> impl IntoResponse {
    use crate::domain::entity::workflow_step::WorkflowStep;

    let steps = req.steps.map(|steps| {
        steps
            .into_iter()
            .map(|s| {
                WorkflowStep::new(
                    s.step_id,
                    s.name,
                    s.step_type,
                    s.assignee_role,
                    s.timeout_hours,
                    s.on_approve,
                    s.on_reject,
                )
            })
            .collect()
    });

    let input = UpdateWorkflowInput {
        id: id.clone(),
        name: req.name,
        description: req.description,
        enabled: req.enabled,
        steps,
    };

    match state.update_workflow_uc.execute(&input).await {
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
        Err(UpdateWorkflowError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("workflow not found: {}", id)})),
        )
            .into_response(),
        Err(UpdateWorkflowError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// DELETE /api/v1/workflows/:id
pub async fn delete_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let input = DeleteWorkflowInput { id: id.clone() };

    match state.delete_workflow_uc.execute(&input).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"success": true, "message": format!("workflow {} deleted", id)})),
        )
            .into_response(),
        Err(DeleteWorkflowError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("workflow not found: {}", id)})),
        )
            .into_response(),
        Err(DeleteWorkflowError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// GET /api/v1/instances
pub async fn list_instances(
    State(state): State<AppState>,
    Query(query): Query<ListInstancesQuery>,
) -> impl IntoResponse {
    let input = ListInstancesInput {
        status: query.status,
        workflow_id: query.workflow_id,
        initiator_id: query.initiator_id,
        page: query.page,
        page_size: query.page_size,
    };

    match state.list_instances_uc.execute(&input).await {
        Ok(output) => {
            let instances: Vec<serde_json::Value> = output
                .instances
                .into_iter()
                .map(|inst| {
                    serde_json::json!({
                        "id": inst.id,
                        "workflow_id": inst.workflow_id,
                        "workflow_name": inst.workflow_name,
                        "title": inst.title,
                        "status": inst.status,
                        "current_step_id": inst.current_step_id,
                        "started_at": inst.started_at.to_rfc3339(),
                        "completed_at": inst.completed_at.map(|t| t.to_rfc3339()),
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "instances": instances,
                    "total_count": output.total_count,
                    "page": output.page,
                    "page_size": output.page_size,
                    "has_next": output.has_next
                })),
            )
                .into_response()
        }
        Err(ListInstancesError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// GET /api/v1/instances/:id
pub async fn get_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
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

/// POST /api/v1/instances/:id/cancel
pub async fn cancel_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CancelInstanceRequest>,
) -> impl IntoResponse {
    let input = CancelInstanceInput {
        id: id.clone(),
        reason: req.reason,
    };

    match state.cancel_instance_uc.execute(&input).await {
        Ok(inst) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": inst.id,
                "status": inst.status,
                "message": "instance cancelled"
            })),
        )
            .into_response(),
        Err(CancelInstanceError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("instance not found: {}", id)})),
        )
            .into_response(),
        Err(CancelInstanceError::InvalidStatus(_, status)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": format!("cannot cancel instance with status: {}", status)})),
        )
            .into_response(),
        Err(CancelInstanceError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// GET /api/v1/tasks
pub async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<ListTasksQuery>,
) -> impl IntoResponse {
    let input = ListTasksInput {
        assignee_id: query.assignee_id,
        status: query.status,
        instance_id: query.instance_id,
        overdue_only: query.overdue_only,
        page: query.page,
        page_size: query.page_size,
    };

    match state.list_tasks_uc.execute(&input).await {
        Ok(output) => {
            let tasks: Vec<serde_json::Value> = output
                .tasks
                .into_iter()
                .map(|t| {
                    serde_json::json!({
                        "id": t.id,
                        "instance_id": t.instance_id,
                        "step_id": t.step_id,
                        "name": t.step_name,
                        "assignee_id": t.assignee_id,
                        "status": t.status,
                        "due_at": t.due_at.map(|d| d.to_rfc3339()),
                        "created_at": t.created_at.to_rfc3339(),
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "tasks": tasks,
                    "total_count": output.total_count,
                    "page": output.page,
                    "page_size": output.page_size,
                    "has_next": output.has_next
                })),
            )
                .into_response()
        }
        Err(ListTasksError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// POST /api/v1/tasks/:id/approve
pub async fn approve_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ApproveTaskRequest>,
) -> impl IntoResponse {
    let input = ApproveTaskInput {
        task_id: id.clone(),
        actor_id: req.actor_id,
        comment: req.comment,
    };

    match state.approve_task_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "task_id": output.task.id,
                "status": output.task.status,
                "instance_status": output.instance_status,
                "next_task_id": output.next_task.map(|t| t.id)
            })),
        )
            .into_response(),
        Err(ApproveTaskError::TaskNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("task not found: {}", id)})),
        )
            .into_response(),
        Err(ApproveTaskError::InvalidStatus(status)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": format!("invalid task status: {}", status)})),
        )
            .into_response(),
        Err(ApproveTaskError::InstanceNotFound(inst_id)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("instance not found: {}", inst_id)})),
        )
            .into_response(),
        Err(ApproveTaskError::DefinitionNotFound(def_id)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("definition not found: {}", def_id)})),
        )
            .into_response(),
        Err(ApproveTaskError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// POST /api/v1/tasks/:id/reject
pub async fn reject_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<RejectTaskRequest>,
) -> impl IntoResponse {
    let input = RejectTaskInput {
        task_id: id.clone(),
        actor_id: req.actor_id,
        comment: req.comment,
    };

    match state.reject_task_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "task_id": output.task.id,
                "status": output.task.status,
                "instance_status": output.instance_status,
                "next_task_id": output.next_task.map(|t| t.id)
            })),
        )
            .into_response(),
        Err(RejectTaskError::TaskNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("task not found: {}", id)})),
        )
            .into_response(),
        Err(RejectTaskError::InvalidStatus(status)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": format!("invalid task status: {}", status)})),
        )
            .into_response(),
        Err(RejectTaskError::InstanceNotFound(inst_id)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("instance not found: {}", inst_id)})),
        )
            .into_response(),
        Err(RejectTaskError::DefinitionNotFound(def_id)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("definition not found: {}", def_id)})),
        )
            .into_response(),
        Err(RejectTaskError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// POST /api/v1/tasks/:id/reassign
pub async fn reassign_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ReassignTaskRequest>,
) -> impl IntoResponse {
    let input = ReassignTaskInput {
        task_id: id.clone(),
        new_assignee_id: req.new_assignee_id,
        reason: req.reason,
        actor_id: req.actor_id,
    };

    match state.reassign_task_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "task_id": output.task.id,
                "assignee_id": output.task.assignee_id,
                "previous_assignee_id": output.previous_assignee_id,
                "message": "task reassigned"
            })),
        )
            .into_response(),
        Err(ReassignTaskError::TaskNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("task not found: {}", id)})),
        )
            .into_response(),
        Err(ReassignTaskError::InvalidStatus(status)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": format!("invalid task status for reassignment: {}", status)})),
        )
            .into_response(),
        Err(ReassignTaskError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}
