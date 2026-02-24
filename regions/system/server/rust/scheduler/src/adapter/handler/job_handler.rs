use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use crate::usecase::create_job::CreateJobInput;

/// GET /api/v1/jobs
pub async fn list_jobs(State(state): State<AppState>) -> impl IntoResponse {
    match state.job_repo.find_all().await {
        Ok(jobs) => (StatusCode::OK, Json(serde_json::json!({ "jobs": jobs }))).into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCHED_LIST_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/jobs/:id
pub async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.get_job_uc.execute(&id).await {
        Ok(job) => (StatusCode::OK, Json(serde_json::to_value(job).unwrap())).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_SCHED_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_SCHED_GET_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// POST /api/v1/jobs
pub async fn create_job(
    State(state): State<AppState>,
    Json(req): Json<CreateJobRequest>,
) -> impl IntoResponse {
    let input = CreateJobInput {
        name: req.name,
        cron_expression: req.cron_expression,
        payload: req.payload,
    };

    match state.create_job_uc.execute(&input).await {
        Ok(job) => {
            (StatusCode::CREATED, Json(serde_json::to_value(job).unwrap())).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("invalid cron") {
                let err = ErrorResponse::new("SYS_SCHED_INVALID_CRON", &msg);
                (StatusCode::BAD_REQUEST, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_SCHED_CREATE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// DELETE /api/v1/jobs/:id
pub async fn delete_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.job_repo.delete(&id).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("job {} deleted", id)
            })),
        )
            .into_response(),
        Ok(false) => {
            let err =
                ErrorResponse::new("SYS_SCHED_NOT_FOUND", &format!("job not found: {}", id));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCHED_DELETE_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// PUT /api/v1/jobs/:id/pause
pub async fn pause_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.pause_job_uc.execute(&id).await {
        Ok(job) => (StatusCode::OK, Json(serde_json::to_value(job).unwrap())).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_SCHED_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_SCHED_PAUSE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// PUT /api/v1/jobs/:id/resume
pub async fn resume_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.resume_job_uc.execute(&id).await {
        Ok(job) => (StatusCode::OK, Json(serde_json::to_value(job).unwrap())).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_SCHED_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_SCHED_RESUME_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub name: String,
    pub cron_expression: String,
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
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
            },
        }
    }
}
