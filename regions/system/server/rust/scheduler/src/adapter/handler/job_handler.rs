use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use super::AppState;
use crate::domain::entity::scheduler_execution::SchedulerExecution;
use crate::usecase::create_job::CreateJobError;
use crate::usecase::create_job::CreateJobInput;
use k1s0_auth::Claims;

/// CRIT-005 対応: Option<Extension<Claims>> からテナント ID を抽出するヘルパー関数。
/// Claims が存在しない場合（認証なし環境）はデフォルト値 "system" を返す。
fn extract_tenant_id(claims: Option<&Extension<Claims>>) -> String {
    claims
        .map_or_else(|| "system".to_string(), |ext| ext.tenant_id().to_string())
}

/// GET /api/v1/jobs
pub async fn list_jobs(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Query(params): Query<ListJobsParams>,
) -> impl IntoResponse {
    use crate::usecase::list_jobs::ListJobsInput;
    // CRIT-005 対応: テナント ID を ListJobsInput に設定してテナントでフィルタリングする
    let tenant_id = extract_tenant_id(claims.as_ref());
    let input = ListJobsInput {
        status: params.status,
        name_prefix: params.name_prefix,
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
        tenant_id,
    };
    match state.list_jobs_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "jobs": output.jobs,
                "pagination": {
                    "total_count": output.total_count,
                    "page": output.page,
                    "page_size": output.page_size,
                    "has_next": output.has_next
                }
            })),
        )
            .into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCHED_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/jobs/:id
pub async fn get_job(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // CRIT-005 対応: テナント ID を渡して RLS セッション変数を設定する
    let tenant_id = extract_tenant_id(claims.as_ref());
    match state.get_job_uc.execute(&id, &tenant_id).await {
        Ok(job) => (StatusCode::OK, Json(job)).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_SCHED_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_SCHED_INTERNAL_ERROR", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// POST /api/v1/jobs
pub async fn create_job(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(req): Json<CreateJobRequest>,
) -> impl IntoResponse {
    if req.target_type.trim().is_empty() {
        let err = ErrorResponse::new("SYS_SCHED_VALIDATION_ERROR", "target_type is required");
        return (StatusCode::BAD_REQUEST, Json(err)).into_response();
    }
    if req.payload.is_null() {
        let err = ErrorResponse::new("SYS_SCHED_VALIDATION_ERROR", "payload is required");
        return (StatusCode::BAD_REQUEST, Json(err)).into_response();
    }

    // CRIT-005 対応: テナント ID を CreateJobInput に設定する
    let tenant_id = extract_tenant_id(claims.as_ref());
    let input = CreateJobInput {
        name: req.name,
        description: req.description,
        cron_expression: req.cron_expression,
        timezone: req.timezone.unwrap_or_else(|| "UTC".to_string()),
        target_type: req.target_type,
        target: req.target,
        payload: req.payload,
        tenant_id,
    };

    match state.create_job_uc.execute(&input).await {
        Ok(job) => (StatusCode::CREATED, Json(job)).into_response(),
        Err(CreateJobError::InvalidCron(expr)) => {
            let err = ErrorResponse::new(
                "SYS_SCHED_INVALID_CRON",
                &format!("invalid cron expression: {expr}"),
            );
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(CreateJobError::InvalidTimezone(tz)) => {
            let err = ErrorResponse::new(
                "SYS_SCHED_INVALID_TIMEZONE",
                &format!("invalid timezone: {tz}"),
            );
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(CreateJobError::Internal(msg)) => {
            if msg.contains("already exists") || msg.contains("duplicate") {
                let err = ErrorResponse::new("SYS_SCHED_ALREADY_EXISTS", &msg);
                (StatusCode::CONFLICT, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_SCHED_INTERNAL_ERROR", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// DELETE /api/v1/jobs/:id
pub async fn delete_job(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    use crate::usecase::delete_job::DeleteJobError;

    // CRIT-005 対応: テナント ID を渡してテナント分離を行う
    let tenant_id = extract_tenant_id(claims.as_ref());
    match state.delete_job_uc.execute(&id, &tenant_id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("job {} deleted", id)
            })),
        )
            .into_response(),
        Err(DeleteJobError::NotFound(_)) => {
            let err = ErrorResponse::new("SYS_SCHED_NOT_FOUND", &format!("job not found: {id}"));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(DeleteJobError::JobRunning(_)) => {
            let err = ErrorResponse::new(
                "SYS_SCHED_JOB_RUNNING",
                &format!("job is currently running: {id}"),
            );
            (StatusCode::CONFLICT, Json(err)).into_response()
        }
        Err(DeleteJobError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_SCHED_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// PUT /api/v1/jobs/:id/pause
pub async fn pause_job(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // CRIT-005 対応: テナント ID を渡してテナント分離を行う
    let tenant_id = extract_tenant_id(claims.as_ref());
    match state.pause_job_uc.execute(&id, &tenant_id).await {
        Ok(job) => (StatusCode::OK, Json(job)).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_SCHED_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_SCHED_INTERNAL_ERROR", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// PUT /api/v1/jobs/:id/resume
pub async fn resume_job(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // CRIT-005 対応: テナント ID を渡してテナント分離を行う
    let tenant_id = extract_tenant_id(claims.as_ref());
    match state.resume_job_uc.execute(&id, &tenant_id).await {
        Ok(job) => (StatusCode::OK, Json(job)).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_SCHED_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_SCHED_INTERNAL_ERROR", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// PUT /api/v1/jobs/:id
pub async fn update_job(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateJobRequest>,
) -> impl IntoResponse {
    use crate::usecase::update_job::{UpdateJobError, UpdateJobInput};

    if req.target_type.trim().is_empty() {
        let err = ErrorResponse::new("SYS_SCHED_VALIDATION_ERROR", "target_type is required");
        return (StatusCode::BAD_REQUEST, Json(err)).into_response();
    }
    if req.payload.is_null() {
        let err = ErrorResponse::new("SYS_SCHED_VALIDATION_ERROR", "payload is required");
        return (StatusCode::BAD_REQUEST, Json(err)).into_response();
    }

    // CRIT-005 対応: テナント ID を UpdateJobInput に設定してテナント分離を行う
    let tenant_id = extract_tenant_id(claims.as_ref());
    let input = UpdateJobInput {
        id,
        name: req.name,
        description: req.description,
        cron_expression: req.cron_expression,
        timezone: req.timezone.unwrap_or_else(|| "UTC".to_string()),
        target_type: req.target_type,
        target: req.target,
        payload: req.payload,
        tenant_id,
    };

    match state.update_job_uc.execute(&input).await {
        Ok(job) => (StatusCode::OK, Json(job)).into_response(),
        Err(UpdateJobError::NotFound(id)) => {
            let err = ErrorResponse::new("SYS_SCHED_NOT_FOUND", &format!("job not found: {id}"));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(UpdateJobError::InvalidCron(expr)) => {
            let err = ErrorResponse::new(
                "SYS_SCHED_INVALID_CRON",
                &format!("invalid cron expression: {expr}"),
            );
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(UpdateJobError::InvalidTimezone(tz)) => {
            let err = ErrorResponse::new(
                "SYS_SCHED_INVALID_TIMEZONE",
                &format!("invalid timezone: {tz}"),
            );
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(UpdateJobError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_SCHED_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/jobs/:id/trigger
pub async fn trigger_job(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    use crate::usecase::trigger_job::TriggerJobError;

    // CRIT-005 対応: テナント ID を渡してテナント分離を行う
    let tenant_id = extract_tenant_id(claims.as_ref());
    match state.trigger_job_uc.execute(&id, &tenant_id).await {
        Ok(execution) => (StatusCode::CREATED, Json(execution)).into_response(),
        Err(TriggerJobError::NotFound(id)) => {
            let err = ErrorResponse::new("SYS_SCHED_NOT_FOUND", &format!("job not found: {id}"));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(TriggerJobError::NotActive(id)) => {
            let err =
                ErrorResponse::new("SYS_SCHED_NOT_ACTIVE", &format!("job is not active: {id}"));
            (StatusCode::CONFLICT, Json(err)).into_response()
        }
        Err(TriggerJobError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_SCHED_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/jobs/:id/executions
pub async fn list_executions(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<ListExecutionsParams>,
) -> impl IntoResponse {
    use crate::usecase::list_executions::ListExecutionsError;
    use chrono::{DateTime, Utc};

    match state.list_executions_uc.execute(&id).await {
        Ok(mut executions) => {
            if let Some(status) = params.status.as_deref() {
                executions.retain(|exec| {
                    exec.status == status || normalize_status(&exec.status) == status
                });
            }

            let from = match params.from {
                Some(from) => {
                    if let Ok(v) = DateTime::parse_from_rfc3339(&from) {
                        Some(v.with_timezone(&Utc))
                    } else {
                        let err = ErrorResponse::new(
                            "SYS_SCHED_VALIDATION_ERROR",
                            "invalid from timestamp; use RFC3339",
                        );
                        return (StatusCode::BAD_REQUEST, Json(err)).into_response();
                    }
                }
                None => None,
            };
            let to = match params.to {
                Some(to) => {
                    if let Ok(v) = DateTime::parse_from_rfc3339(&to) {
                        Some(v.with_timezone(&Utc))
                    } else {
                        let err = ErrorResponse::new(
                            "SYS_SCHED_VALIDATION_ERROR",
                            "invalid to timestamp; use RFC3339",
                        );
                        return (StatusCode::BAD_REQUEST, Json(err)).into_response();
                    }
                }
                None => None,
            };

            if let Some(from) = from {
                executions.retain(|exec| exec.started_at >= from);
            }
            if let Some(to) = to {
                executions.retain(|exec| exec.started_at <= to);
            }

            let page = params.page.unwrap_or(1).max(1);
            let page_size = params.page_size.unwrap_or(20).clamp(1, 200);
            // LOW-008: 安全な型変換（オーバーフロー防止）
            let total_count = u64::try_from(executions.len()).unwrap_or(u64::MAX);
            let start = usize::try_from((page - 1) * page_size).unwrap_or(0);
            let take_count = usize::try_from(page_size).unwrap_or(0);
            let page_items: Vec<SchedulerExecution> = executions
                .into_iter()
                .skip(start)
                .take(take_count)
                .collect();
            let has_next = start + page_items.len() < usize::try_from(total_count).unwrap_or(usize::MAX);

            let executions: Vec<serde_json::Value> =
                page_items.into_iter().map(execution_to_response).collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "executions": executions,
                    "pagination": {
                        "total_count": total_count,
                        "page": page,
                        "page_size": page_size,
                        "has_next": has_next
                    }
                })),
            )
                .into_response()
        }
        Err(ListExecutionsError::NotFound(id)) => {
            let err = ErrorResponse::new("SYS_SCHED_NOT_FOUND", &format!("job not found: {id}"));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(ListExecutionsError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_SCHED_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

fn execution_to_response(execution: SchedulerExecution) -> serde_json::Value {
    let finished_at = execution.finished_at;
    let duration_ms = finished_at.as_ref().and_then(|finished_at| {
        let duration = finished_at.signed_duration_since(execution.started_at);
        if duration.num_milliseconds() >= 0 {
            // LOW-008: 安全な型変換（オーバーフロー防止、符号確認済み）
            Some(u64::try_from(duration.num_milliseconds()).unwrap_or(0))
        } else {
            None
        }
    });

    serde_json::json!({
        "id": execution.id.clone(),
        "job_id": execution.job_id.clone(),
        "status": normalize_status(&execution.status),
        "triggered_by": execution.triggered_by,
        "started_at": execution.started_at.to_rfc3339(),
        "finished_at": finished_at.map(|t| t.to_rfc3339()),
        "duration_ms": duration_ms,
        "error_message": execution.error_message,
    })
}

fn normalize_status(status: &str) -> String {
    status.to_string()
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct ListJobsParams {
    pub status: Option<String>,
    pub name_prefix: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ListExecutionsParams {
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub name: String,
    pub description: Option<String>,
    pub cron_expression: String,
    pub timezone: Option<String>,
    pub target_type: String,
    pub target: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJobRequest {
    pub name: String,
    pub description: Option<String>,
    pub cron_expression: String,
    pub timezone: Option<String>,
    pub target_type: String,
    pub target: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub request_id: String,
    pub details: Vec<String>,
}

impl ErrorResponse {
    #[must_use]
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
                details: vec![],
            },
        }
    }
}
