use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use crate::domain::entity::feature_flag::{FlagRule, FlagVariant};
use crate::usecase::create_flag::CreateFlagInput;
use crate::usecase::update_flag::UpdateFlagInput;
use k1s0_server_common::error as codes;
use k1s0_server_common::ErrorResponse;

/// システムテナントUUID: JWT クレームが存在しない場合のフォールバック
const SYSTEM_TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";

/// JWT クレームからテナントIDを抽出するヘルパー。
/// クレームがない場合はシステムテナントUUIDをフォールバックとして使用する。
fn extract_tenant_id(claims: &Option<Extension<k1s0_auth::Claims>>) -> Uuid {
    claims
        .as_ref()
        .and_then(|ext| Uuid::parse_str(&ext.0.tenant_id).ok())
        .unwrap_or_else(|| {
            Uuid::parse_str(SYSTEM_TENANT_ID).expect("system tenant UUID is valid")
        })
}

/// GET /api/v1/flags
pub async fn list_flags(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
) -> impl IntoResponse {
    // STATIC-CRITICAL-001: テナントスコープでフラグ一覧を取得する
    let tenant_id = extract_tenant_id(&claims);
    match state.list_flags_uc.execute(tenant_id).await {
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
pub async fn get_flag(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    // STATIC-CRITICAL-001: テナントスコープでフラグを取得する
    let tenant_id = extract_tenant_id(&claims);
    match state.get_flag_uc.execute(tenant_id, &key).await {
        Ok(flag) => {
            // フラグレスポンスを直接 Json<FlagResponse> として返す（.expect() 排除）
            let resp = FlagResponse::from(flag);
            (StatusCode::OK, Json(resp)).into_response()
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
    claims: Option<Extension<k1s0_auth::Claims>>,
    Json(req): Json<CreateFlagRequest>,
) -> impl IntoResponse {
    // STATIC-CRITICAL-001: テナントスコープでフラグを作成する
    let tenant_id = extract_tenant_id(&claims);
    let input = CreateFlagInput {
        tenant_id,
        flag_key: req.flag_key,
        description: req.description,
        enabled: req.enabled,
        variants: req.variants.unwrap_or_default(),
    };

    match state.create_flag_uc.execute(&input).await {
        Ok(flag) => {
            // フラグレスポンスを直接 Json<FlagResponse> として返す（.expect() 排除）
            let resp = FlagResponse::from(flag);
            (StatusCode::CREATED, Json(resp)).into_response()
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
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(key): Path<String>,
    Json(req): Json<UpdateFlagRequest>,
) -> impl IntoResponse {
    // STATIC-CRITICAL-001: テナントスコープでフラグを更新する
    let tenant_id = extract_tenant_id(&claims);
    let input = UpdateFlagInput {
        tenant_id,
        flag_key: key,
        enabled: req.enabled,
        description: req.description,
        variants: req.variants,
        rules: req.rules,
    };

    match state.update_flag_uc.execute(&input).await {
        Ok(flag) => {
            // フラグレスポンスを直接 Json<FlagResponse> として返す（.expect() 排除）
            let resp = FlagResponse::from(flag);
            (StatusCode::OK, Json(resp)).into_response()
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
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    use crate::usecase::delete_flag::DeleteFlagError;

    // STATIC-CRITICAL-001: テナントスコープでフラグを削除する
    let tenant_id = extract_tenant_id(&claims);

    let flag = match state.get_flag_uc.execute(tenant_id, &key).await {
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

    match state.delete_flag_uc.execute(tenant_id, &flag.id).await {
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
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(key): Path<String>,
    Json(req): Json<EvaluateFlagRequest>,
) -> impl IntoResponse {
    use crate::domain::entity::evaluation::EvaluationContext;
    use crate::usecase::evaluate_flag::EvaluateFlagInput;

    // STATIC-CRITICAL-001: テナントスコープでフラグを評価する
    let tenant_id = extract_tenant_id(&claims);
    let input = EvaluateFlagInput {
        tenant_id,
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

/// エラーレスポンスを生成するヘルパー関数。
/// Json<ErrorResponse> を直接返すことで .expect() を排除する。
fn error_response(
    status: StatusCode,
    code: impl Into<k1s0_server_common::ErrorCode>,
    message: impl Into<String>,
) -> Response {
    let err = ErrorResponse::new(code, message);
    (status, Json(err)).into_response()
}
