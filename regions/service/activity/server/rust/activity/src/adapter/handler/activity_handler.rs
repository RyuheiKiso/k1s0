// アクティビティ REST ハンドラー。
// Claims 拡張から認証ユーザー ID および tenant_id を取得してユースケースに渡す。
// RLS テナント分離のため Claims の iss（発行者）を tenant_id として使用する。
// iss が空の場合は "system" をデフォルト値として使用する。
use crate::adapter::handler::AppState;
use crate::domain::entity::activity::{ActivityFilter, CreateActivity};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::claims::actor_from_claims;
use k1s0_auth::Claims;
use k1s0_server_common::ServiceError;
use serde::Deserialize;
use uuid::Uuid;

fn map_err(e: anyhow::Error) -> ServiceError {
    ServiceError::Internal {
        code: k1s0_server_common::ErrorCode::new("SVC_ACTIVITY_ERROR"),
        message: e.to_string(),
    }
}

/// Claims の iss（トークン発行者）からテナント ID を取得する。
/// iss が空または Claims が存在しない場合は "system" を返す。
/// TODO: テナント専用 JWT Claim または X-Tenant-ID ヘッダーが導入された場合は更新すること
fn tenant_id_from_claims(claims: Option<&Claims>) -> &str {
    claims
        .map(|c| c.iss.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("system")
}

#[derive(Debug, Deserialize)]
pub struct ListActivitiesQuery {
    pub task_id: Option<Uuid>,
    pub actor_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_activities(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Query(q): Query<ListActivitiesQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|ext| &ext.0));
    let filter = ActivityFilter {
        task_id: q.task_id,
        actor_id: q.actor_id,
        status: q.status.as_deref().and_then(|s| s.parse().ok()),
        limit: q.limit,
        offset: q.offset,
    };
    let (activities, total) = state.list_activities_uc.execute(tenant_id, &filter).await.map_err(map_err)?;
    Ok(Json(serde_json::json!({ "activities": activities, "total": total })))
}

pub async fn get_activity(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|ext| &ext.0));
    let activity = state
        .get_activity_uc
        .execute(tenant_id, id)
        .await
        .map_err(map_err)?
        .ok_or_else(|| ServiceError::NotFound {
            code: k1s0_server_common::ErrorCode::new("SVC_ACTIVITY_NOT_FOUND"),
            message: format!("Activity '{}' not found", id),
        })?;
    Ok(Json(activity))
}

// アクティビティ作成: リクエスト拡張から Claims を取得し、actor_id および tenant_id として使用する
pub async fn create_activity(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Json(input): Json<CreateActivity>,
) -> Result<impl IntoResponse, ServiceError> {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|ext| &ext.0));
    // 認証済みの場合は JWT sub/username を使用し、未認証の場合は "anonymous" を使用する
    let actor = actor_from_claims(claims.as_ref().map(|ext| &ext.0));
    let activity = state
        .create_activity_uc
        .execute(tenant_id, &input, &actor)
        .await
        .map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(activity)))
}

// アクティビティ提出: リクエスト拡張から Claims を取得し、actor_id および tenant_id として使用する（Active → Submitted）
pub async fn submit_activity(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|ext| &ext.0));
    // 認証済みの場合は JWT sub/username を使用し、未認証の場合は "anonymous" を使用する
    let actor = actor_from_claims(claims.as_ref().map(|ext| &ext.0));
    let activity = state
        .submit_activity_uc
        .execute(tenant_id, id, &actor)
        .await
        .map_err(map_err)?;
    Ok(Json(activity))
}

// アクティビティ承認: リクエスト拡張から Claims を取得し、approver_id および tenant_id として使用する（Submitted → Approved）
pub async fn approve_activity(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|ext| &ext.0));
    // 認証済みの場合は JWT sub/username を使用し、未認証の場合は "anonymous" を使用する
    let actor = actor_from_claims(claims.as_ref().map(|ext| &ext.0));
    let activity = state
        .approve_activity_uc
        .execute(tenant_id, id, &actor)
        .await
        .map_err(map_err)?;
    Ok(Json(activity))
}

// アクティビティ却下: リクエスト拡張から Claims を取得し、rejector_id および tenant_id として使用する（Submitted → Rejected）
pub async fn reject_activity(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|ext| &ext.0));
    // 認証済みの場合は JWT sub/username を使用し、未認証の場合は "anonymous" を使用する
    let actor = actor_from_claims(claims.as_ref().map(|ext| &ext.0));
    let activity = state
        .reject_activity_uc
        .execute(tenant_id, id, &actor, "no reason provided")
        .await
        .map_err(map_err)?;
    Ok(Json(activity))
}
