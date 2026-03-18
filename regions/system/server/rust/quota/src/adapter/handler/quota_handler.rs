use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use super::AppState;
use crate::usecase::create_quota_policy::CreateQuotaPolicyInput;
use crate::usecase::get_quota_usage::GetQuotaUsageError;
use crate::usecase::increment_quota_usage::IncrementQuotaUsageInput;
use crate::usecase::list_quota_policies::ListQuotaPoliciesInput;
use crate::usecase::reset_quota_usage::ResetQuotaUsageInput;
use crate::usecase::update_quota_policy::UpdateQuotaPolicyInput;

fn classify_internal_error(msg: &str) -> (StatusCode, &'static str) {
    let lower = msg.to_ascii_lowercase();
    if lower.contains("redis") {
        return (StatusCode::BAD_GATEWAY, "SYS_QUOTA_REDIS_ERROR");
    }
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "SYS_QUOTA_INTERNAL_ERROR",
    )
}

fn validation_error_response(message: &str) -> ErrorResponse {
    let mut details = Vec::new();
    let lower = message.to_ascii_lowercase();
    if lower.contains("subject_type") {
        details.push(ErrorDetail {
            field: "subject_type".to_string(),
            message: "must be one of: tenant, user, api_key".to_string(),
        });
    } else if lower.contains("period") {
        details.push(ErrorDetail {
            field: "period".to_string(),
            message: "must be one of: daily, monthly".to_string(),
        });
    } else if lower.contains("limit") {
        details.push(ErrorDetail {
            field: "limit".to_string(),
            message: "must be greater than 0".to_string(),
        });
    } else {
        details.push(ErrorDetail {
            field: "request".to_string(),
            message: message.to_string(),
        });
    }

    ErrorResponse::with_details("SYS_QUOTA_VALIDATION_ERROR", message, details)
}

/// GET /api/v1/quotas
pub async fn list_quotas(
    State(state): State<AppState>,
    Query(params): Query<ListQuotasParams>,
) -> impl IntoResponse {
    let input = ListQuotaPoliciesInput {
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
        subject_type: params.subject_type,
        subject_id: params.subject_id,
        enabled_only: params.enabled_only,
    };

    match state.list_policies_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "quotas": output.quotas,
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
            let msg = e.to_string();
            let (status, code) = classify_internal_error(&msg);
            let err = ErrorResponse::new(code, &msg);
            (status, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/quotas/:id
pub async fn get_quota(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.get_policy_uc.execute(&id).await {
        Ok(policy) => (StatusCode::OK, Json(serde_json::to_value(policy).expect("クォータポリシーのJSON変換に失敗"))).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_QUOTA_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let (status, code) = classify_internal_error(&msg);
                let err = ErrorResponse::new(code, &msg);
                (status, Json(err)).into_response()
            }
        }
    }
}

/// POST /api/v1/quotas
pub async fn create_quota(
    State(state): State<AppState>,
    Json(req): Json<CreateQuotaRequest>,
) -> impl IntoResponse {
    let input = CreateQuotaPolicyInput {
        name: req.name,
        subject_type: req.subject_type,
        subject_id: req.subject_id,
        limit: req.limit,
        period: req.period,
        enabled: req.enabled,
        alert_threshold_percent: req.alert_threshold_percent,
    };

    match state.create_policy_uc.execute(&input).await {
        Ok(policy) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(policy).expect("クォータポリシーのJSON変換に失敗")),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("validation") {
                let err = validation_error_response(&msg);
                (StatusCode::BAD_REQUEST, Json(err)).into_response()
            } else if msg.contains("duplicate")
                || msg.contains("already exists")
                || msg.contains("unique constraint")
            {
                let err = ErrorResponse::new("SYS_QUOTA_ALREADY_EXISTS", &msg);
                (StatusCode::CONFLICT, Json(err)).into_response()
            } else {
                let (status, code) = classify_internal_error(&msg);
                let err = ErrorResponse::new(code, &msg);
                (status, Json(err)).into_response()
            }
        }
    }
}

/// PUT /api/v1/quotas/:id
pub async fn update_quota(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateQuotaRequest>,
) -> impl IntoResponse {
    let input = UpdateQuotaPolicyInput {
        id,
        name: req.name,
        subject_type: req.subject_type,
        subject_id: req.subject_id,
        limit: req.limit,
        period: req.period,
        enabled: req.enabled,
        alert_threshold_percent: req.alert_threshold_percent,
    };

    match state.update_policy_uc.execute(&input).await {
        Ok(policy) => (StatusCode::OK, Json(serde_json::to_value(policy).expect("クォータポリシーのJSON変換に失敗"))).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_QUOTA_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else if msg.contains("validation") {
                let err = validation_error_response(&msg);
                (StatusCode::BAD_REQUEST, Json(err)).into_response()
            } else {
                let (status, code) = classify_internal_error(&msg);
                let err = ErrorResponse::new(code, &msg);
                (status, Json(err)).into_response()
            }
        }
    }
}

/// POST /api/v1/quotas/:id/check - Check quota remaining (read-only)
pub async fn check_quota(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.get_usage_uc.execute(&id).await {
        Ok(usage) => (StatusCode::OK, Json(serde_json::to_value(usage).expect("クォータ使用量のJSON変換に失敗"))).into_response(),
        Err(GetQuotaUsageError::NotFound(id)) => {
            let err =
                ErrorResponse::new("SYS_QUOTA_NOT_FOUND", &format!("quota not found: {}", id));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(GetQuotaUsageError::Internal(msg)) => {
            let (status, code) = classify_internal_error(&msg);
            let err = ErrorResponse::new(code, &msg);
            (status, Json(err)).into_response()
        }
    }
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct ListQuotasParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub subject_type: Option<String>,
    pub subject_id: Option<String>,
    pub enabled_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateQuotaRequest {
    pub name: String,
    pub subject_type: String,
    pub subject_id: String,
    pub limit: u64,
    pub period: String,
    pub enabled: bool,
    pub alert_threshold_percent: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateQuotaRequest {
    pub name: String,
    pub subject_type: String,
    pub subject_id: String,
    pub limit: u64,
    pub period: String,
    pub enabled: bool,
    pub alert_threshold_percent: Option<u8>,
}

/// DELETE /api/v1/quotas/:id
pub async fn delete_quota(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    use crate::usecase::delete_quota_policy::DeleteQuotaPolicyError;

    match state.delete_policy_uc.execute(&id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(DeleteQuotaPolicyError::NotFound(id)) => {
            let err =
                ErrorResponse::new("SYS_QUOTA_NOT_FOUND", &format!("quota not found: {}", id));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(DeleteQuotaPolicyError::Internal(msg)) => {
            let (status, code) = classify_internal_error(&msg);
            let err = ErrorResponse::new(code, &msg);
            (status, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/quotas/:id/usage
pub async fn get_usage(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    use crate::usecase::get_quota_usage::GetQuotaUsageError;

    match state.get_usage_uc.execute(&id).await {
        Ok(usage) => (StatusCode::OK, Json(serde_json::to_value(usage).expect("クォータ使用量のJSON変換に失敗"))).into_response(),
        Err(GetQuotaUsageError::NotFound(id)) => {
            let err =
                ErrorResponse::new("SYS_QUOTA_NOT_FOUND", &format!("quota not found: {}", id));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(GetQuotaUsageError::Internal(msg)) => {
            let (status, code) = classify_internal_error(&msg);
            let err = ErrorResponse::new(code, &msg);
            (status, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/quotas/:id/usage/increment
pub async fn increment_usage(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<IncrementUsageRequest>,
) -> impl IntoResponse {
    let input = IncrementQuotaUsageInput {
        quota_id: id,
        amount: req.amount,
        request_id: req.request_id,
    };

    match state.increment_usage_uc.execute(&input).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).expect("クォータ増分結果のJSON変換に失敗"))).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_QUOTA_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else if msg.contains("exceeded") {
                let err = ErrorResponse::new("SYS_QUOTA_EXCEEDED", &msg);
                (StatusCode::TOO_MANY_REQUESTS, Json(err)).into_response()
            } else {
                let (status, code) = classify_internal_error(&msg);
                let err = ErrorResponse::new(code, &msg);
                (status, Json(err)).into_response()
            }
        }
    }
}

/// POST /api/v1/quotas/:id/usage/reset
pub async fn reset_usage(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ResetUsageRequest>,
) -> impl IntoResponse {
    use crate::usecase::reset_quota_usage::ResetQuotaUsageError;

    let input = ResetQuotaUsageInput {
        quota_id: id,
        reason: req.reason,
        reset_by: req.reset_by,
    };

    match state.reset_usage_uc.execute(&input).await {
        Ok(output) => (StatusCode::OK, Json(serde_json::to_value(output).expect("クォータリセット結果のJSON変換に失敗"))).into_response(),
        Err(ResetQuotaUsageError::NotFound(id)) => {
            let err =
                ErrorResponse::new("SYS_QUOTA_NOT_FOUND", &format!("quota not found: {}", id));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(ResetQuotaUsageError::Validation(msg)) => {
            let err = validation_error_response(&msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(ResetQuotaUsageError::Internal(msg)) => {
            let (status, code) = classify_internal_error(&msg);
            let err = ErrorResponse::new(code, &msg);
            (status, Json(err)).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct IncrementUsageRequest {
    pub amount: u64,
    pub request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResetUsageRequest {
    pub reason: String,
    pub reset_by: String,
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
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub field: String,
    pub message: String,
}

impl ErrorResponse {
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

    pub fn with_details(code: &str, message: &str, details: Vec<ErrorDetail>) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
                details,
            },
        }
    }
}
