use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use super::AppState;
use crate::domain::entity::feature_flag::{FlagRule, FlagVariant};
use crate::usecase::create_flag::CreateFlagInput;
use crate::usecase::update_flag::UpdateFlagInput;
use k1s0_server_common::error as codes;
use k1s0_server_common::ErrorResponse;

/// GET /api/v1/flags
pub async fn list_flags(State(state): State<AppState>) -> impl IntoResponse {
    match state.list_flags_uc.execute().await {
        Ok(flags) => {
            let items: Vec<FlagResponse> = flags.into_iter().map(FlagResponse::from).collect();
            (StatusCode::OK, Json(serde_json::json!({ "flags": items }))).into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            codes::featureflag::list_failed(),
            e.to_string(),
        ),
    }
}

/// GET /api/v1/flags/:key
pub async fn get_flag(State(state): State<AppState>, Path(key): Path<String>) -> impl IntoResponse {
    match state.get_flag_uc.execute(&key).await {
        Ok(flag) => {
            let resp = FlagResponse::from(flag);
            (StatusCode::OK, Json(serde_json::to_value(resp).expect("フラグ取得レスポンスのJSON変換に失敗"))).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                error_response(StatusCode::NOT_FOUND, codes::featureflag::not_found(), &msg)
            } else {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    codes::featureflag::get_failed(),
                    &msg,
                )
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
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(resp).expect("フラグ作成レスポンスのJSON変換に失敗")),
            )
                .into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already exists") {
                error_response(
                    StatusCode::CONFLICT,
                    codes::featureflag::already_exists(),
                    &msg,
                )
            } else {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    codes::featureflag::create_failed(),
                    &msg,
                )
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
        variants: req.variants,
        rules: req.rules,
    };

    match state.update_flag_uc.execute(&input).await {
        Ok(flag) => {
            let resp = FlagResponse::from(flag);
            (StatusCode::OK, Json(serde_json::to_value(resp).expect("フラグ更新レスポンスのJSON変換に失敗"))).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                error_response(StatusCode::NOT_FOUND, codes::featureflag::not_found(), &msg)
            } else {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    codes::featureflag::update_failed(),
                    &msg,
                )
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
                return error_response(
                    StatusCode::NOT_FOUND,
                    codes::featureflag::not_found(),
                    &msg,
                );
            } else {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    codes::featureflag::get_failed(),
                    &msg,
                );
            }
        }
    };

    match state.delete_flag_uc.execute(&flag.id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"success": true, "message": format!("flag {} deleted", key)})),
        )
            .into_response(),
        Err(DeleteFlagError::NotFound(_)) => error_response(
            StatusCode::NOT_FOUND,
            codes::featureflag::not_found(),
            format!("flag not found: {}", key),
        ),
        Err(DeleteFlagError::Internal(msg)) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            codes::featureflag::delete_failed(),
            &msg,
        ),
    }
}

/// POST /api/v1/flags/:key/evaluate
pub async fn evaluate_flag(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<EvaluateFlagRequest>,
) -> impl IntoResponse {
    use crate::domain::entity::evaluation::EvaluationContext;
    use crate::usecase::evaluate_flag::EvaluateFlagInput;

    let input = EvaluateFlagInput {
        flag_key: key,
        context: EvaluationContext {
            user_id: req.context.user_id,
            tenant_id: req.context.tenant_id,
            attributes: req.context.attributes.unwrap_or_default(),
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
                error_response(StatusCode::NOT_FOUND, codes::featureflag::not_found(), &msg)
            } else {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    codes::featureflag::evaluate_failed(),
                    &msg,
                )
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
    pub variants: Option<Vec<FlagVariant>>,
    pub rules: Option<Vec<FlagRule>>,
}

#[derive(Debug, Deserialize)]
pub struct EvaluateFlagRequest {
    #[serde(default)]
    pub context: EvaluateFlagContextRequest,
}

#[derive(Debug, Deserialize, Default)]
pub struct EvaluateFlagContextRequest {
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
    pub rules: Vec<FlagRule>,
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
            rules: f.rules,
            created_at: f.created_at.to_rfc3339(),
            updated_at: f.updated_at.to_rfc3339(),
        }
    }
}

fn error_response(
    status: StatusCode,
    code: impl Into<k1s0_server_common::ErrorCode>,
    message: impl Into<String>,
) -> Response {
    let err = ErrorResponse::new(code, message);
    // エラーレスポンスをJSON値に変換（ErrorResponseは常にシリアライズ可能）
    (status, Json(serde_json::to_value(err).expect("ErrorResponseのJSON変換に失敗"))).into_response()
}
