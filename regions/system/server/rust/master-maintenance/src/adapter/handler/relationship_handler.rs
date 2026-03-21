use crate::adapter::handler::error::AppError;
use crate::adapter::handler::{actor_from_claims, publish_change_event, AppState};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::Claims;

/// リレーション一覧ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn list_relationships(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let relationships = state.manage_relationships_uc.list_relationships().await?;
    Ok(Json(relationships))
}

/// リレーション作成ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn create_relationship(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let claims_ext = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims_ext.0));
    let relationship = state
        .manage_relationships_uc
        .create_relationship(&input, &actor, None)
        .await?;
    publish_change_event(
        &state,
        serde_json::json!({
            "event_type": "MASTER_MAINTENANCE_DATA_CHANGED",
            "resource_type": "relationship",
            "resource_id": relationship.id,
            "action": "created",
            "actor": actor,
            "after": relationship.clone(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    )
    .await;
    Ok((StatusCode::CREATED, Json(relationship)))
}

/// リレーション更新ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn update_relationship(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<uuid::Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let relationship = state
        .manage_relationships_uc
        .update_relationship(id, &input)
        .await?;
    Ok(Json(relationship))
}

/// リレーション削除ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn delete_relationship(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    state
        .manage_relationships_uc
        .delete_relationship(id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 関連レコード取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn get_related_records(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((name, id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let related = state
        .manage_relationships_uc
        .get_related_records(&name, &id, None)
        .await?;
    Ok(Json(related))
}
