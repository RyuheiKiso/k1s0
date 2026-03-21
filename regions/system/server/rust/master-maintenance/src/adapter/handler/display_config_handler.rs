use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::Claims;

use crate::adapter::handler::error::AppError;
use crate::adapter::handler::{actor_from_claims, publish_change_event, AppState};

/// 表示設定一覧ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn list_display_configs(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _ = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let configs = state
        .manage_display_configs_uc
        .list_display_configs(&name, None)
        .await?;
    Ok(Json(configs))
}

/// 表示設定取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn get_display_config(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((_name, id)): Path<(String, uuid::Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _ = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let config = state
        .manage_display_configs_uc
        .get_display_config(id)
        .await?
        .ok_or_else(|| {
            AppError::not_found(
                "SYS_MM_DISPLAY_CONFIG_NOT_FOUND",
                "Display config not found",
            )
        })?;
    Ok(Json(config))
}

/// 表示設定作成ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn create_display_config(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let claims_ext = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims_ext.0));
    let config = state
        .manage_display_configs_uc
        .create_display_config(&name, &input, &actor, None)
        .await?;
    publish_change_event(
        &state,
        serde_json::json!({
            "event_type": "MASTER_MAINTENANCE_DATA_CHANGED",
            "resource_type": "display_config",
            "resource_id": config.id,
            "resource_name": name,
            "action": "created",
            "actor": actor,
            "after": config.clone(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    )
    .await;
    Ok((StatusCode::CREATED, Json(config)))
}

/// 表示設定更新ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn update_display_config(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((_name, id)): Path<(String, uuid::Uuid)>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _ = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let config = state
        .manage_display_configs_uc
        .update_display_config(id, &input)
        .await?;
    Ok(Json(config))
}

/// 表示設定削除ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn delete_display_config(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((_name, id)): Path<(String, uuid::Uuid)>,
) -> Result<StatusCode, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _ = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    state
        .manage_display_configs_uc
        .delete_display_config(id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
