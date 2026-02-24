use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use super::AppState;
use crate::usecase::create_quota_policy::CreateQuotaPolicyInput;
use crate::usecase::increment_quota_usage::IncrementQuotaUsageInput;
use crate::usecase::list_quota_policies::ListQuotaPoliciesInput;
use crate::usecase::update_quota_policy::UpdateQuotaPolicyInput;

/// GET /api/v1/quotas
pub async fn list_quotas(
    State(state): State<AppState>,
    Query(params): Query<ListQuotasParams>,
) -> impl IntoResponse {
    let input = ListQuotaPoliciesInput {
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
    };

    match state.list_policies_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "quotas": output.quotas,
                "total_count": output.total_count,
                "page": output.page,
                "page_size": output.page_size,
                "has_next": output.has_next
            })),
        )
            .into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_QUOTA_LIST_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/quotas/:id
pub async fn get_quota(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.get_policy_uc.execute(&id).await {
        Ok(policy) => (StatusCode::OK, Json(serde_json::to_value(policy).unwrap())).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_QUOTA_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_QUOTA_GET_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
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
        Ok(policy) => {
            (StatusCode::CREATED, Json(serde_json::to_value(policy).unwrap())).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("validation") {
                let err = ErrorResponse::new("SYS_QUOTA_VALIDATION", &msg);
                (StatusCode::BAD_REQUEST, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_QUOTA_CREATE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
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
        Ok(policy) => (StatusCode::OK, Json(serde_json::to_value(policy).unwrap())).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_QUOTA_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else if msg.contains("validation") {
                let err = ErrorResponse::new("SYS_QUOTA_VALIDATION", &msg);
                (StatusCode::BAD_REQUEST, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_QUOTA_UPDATE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// POST /api/v1/quotas/:id/check - Check quota usage and optionally increment
pub async fn check_quota(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CheckQuotaRequest>,
) -> impl IntoResponse {
    if let Some(amount) = req.increment {
        // Increment and check
        let input = IncrementQuotaUsageInput {
            quota_id: id,
            amount,
        };
        match state.increment_usage_uc.execute(&input).await {
            Ok(result) => {
                (StatusCode::OK, Json(serde_json::to_value(result).unwrap())).into_response()
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("not found") {
                    let err = ErrorResponse::new("SYS_QUOTA_NOT_FOUND", &msg);
                    (StatusCode::NOT_FOUND, Json(err)).into_response()
                } else if msg.contains("exceeded") {
                    let err = ErrorResponse::new("SYS_QUOTA_EXCEEDED", &msg);
                    (StatusCode::TOO_MANY_REQUESTS, Json(err)).into_response()
                } else {
                    let err = ErrorResponse::new("SYS_QUOTA_CHECK_FAILED", &msg);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
                }
            }
        }
    } else {
        // Just get usage
        match state.get_usage_uc.execute(&id).await {
            Ok(usage) => {
                (StatusCode::OK, Json(serde_json::to_value(usage).unwrap())).into_response()
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("not found") {
                    let err = ErrorResponse::new("SYS_QUOTA_NOT_FOUND", &msg);
                    (StatusCode::NOT_FOUND, Json(err)).into_response()
                } else {
                    let err = ErrorResponse::new("SYS_QUOTA_CHECK_FAILED", &msg);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
                }
            }
        }
    }
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct ListQuotasParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
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

#[derive(Debug, Deserialize)]
pub struct CheckQuotaRequest {
    pub increment: Option<u64>,
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
