use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use super::AppState;
use crate::domain::entity::feature_flag::FlagVariant;
use crate::usecase::create_flag::CreateFlagInput;
use crate::usecase::update_flag::UpdateFlagInput;

/// GET /api/v1/flags
pub async fn list_flags(State(state): State<AppState>) -> impl IntoResponse {
    match state.flag_repo.find_all().await {
        Ok(flags) => {
            let items: Vec<FlagResponse> = flags.into_iter().map(FlagResponse::from).collect();
            (StatusCode::OK, Json(serde_json::json!({ "flags": items }))).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_FF_LIST_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/flags/:key
pub async fn get_flag(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    match state.get_flag_uc.execute(&key).await {
        Ok(flag) => {
            let resp = FlagResponse::from(flag);
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_FF_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_FF_GET_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// POST /api/v1/flags
pub async fn create_flag(
    State(state): State<AppState>,
    Json(req): Json<CreateFlagRequest>,
) -> impl IntoResponse {
    let input = CreateFlagInput {
        flag_key: req.flag_key,
        description: req.description,
        enabled: req.enabled,
        variants: req.variants.unwrap_or_default(),
    };

    match state.create_flag_uc.execute(&input).await {
        Ok(flag) => {
            let resp = FlagResponse::from(flag);
            (StatusCode::CREATED, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already exists") {
                let err = ErrorResponse::new("SYS_FF_ALREADY_EXISTS", &msg);
                (StatusCode::CONFLICT, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_FF_CREATE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// PUT /api/v1/flags/:key
pub async fn update_flag(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<UpdateFlagRequest>,
) -> impl IntoResponse {
    let input = UpdateFlagInput {
        flag_key: key,
        enabled: req.enabled,
        description: req.description,
    };

    match state.update_flag_uc.execute(&input).await {
        Ok(flag) => {
            let resp = FlagResponse::from(flag);
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_FF_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_FF_UPDATE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// DELETE /api/v1/flags/:key
pub async fn delete_flag(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    use crate::usecase::delete_flag::DeleteFlagError;

    let flag = match state.get_flag_uc.execute(&key).await {
        Ok(f) => f,
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_FF_NOT_FOUND", &msg);
                return (StatusCode::NOT_FOUND, Json(err)).into_response();
            } else {
                let err = ErrorResponse::new("SYS_FF_DELETE_FAILED", &msg);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response();
            }
        }
    };

    match state.delete_flag_uc.execute(&flag.id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"success": true, "message": format!("flag {} deleted", key)})),
        )
            .into_response(),
        Err(DeleteFlagError::NotFound(_)) => {
            let err = ErrorResponse::new("SYS_FF_NOT_FOUND", &format!("flag not found: {}", key));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(DeleteFlagError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_FF_DELETE_FAILED", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/flags/:key/evaluate
pub async fn evaluate_flag(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<EvaluateFlagRequest>,
) -> impl IntoResponse {
    use crate::usecase::evaluate_flag::EvaluateFlagInput;
    use crate::domain::entity::evaluation::EvaluationContext;

    let input = EvaluateFlagInput {
        flag_key: key,
        context: EvaluationContext {
            user_id: req.user_id,
            tenant_id: req.tenant_id,
            attributes: req.attributes.unwrap_or_default(),
        },
    };

    match state.evaluate_flag_uc.execute(&input).await {
        Ok(result) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "flag_key": result.flag_key,
                "enabled": result.enabled,
                "variant": result.variant,
                "reason": result.reason
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_FF_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_FF_EVALUATE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct CreateFlagRequest {
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Option<Vec<FlagVariant>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFlagRequest {
    pub enabled: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EvaluateFlagRequest {
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub attributes: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct FlagResponse {
    pub id: String,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<FlagVariant>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::domain::entity::feature_flag::FeatureFlag> for FlagResponse {
    fn from(f: crate::domain::entity::feature_flag::FeatureFlag) -> Self {
        Self {
            id: f.id.to_string(),
            flag_key: f.flag_key,
            description: f.description,
            enabled: f.enabled,
            variants: f.variants,
            created_at: f.created_at.to_rfc3339(),
            updated_at: f.updated_at.to_rfc3339(),
        }
    }
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
