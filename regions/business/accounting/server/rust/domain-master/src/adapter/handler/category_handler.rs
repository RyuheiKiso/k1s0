use crate::adapter::handler::error::from_anyhow;
use crate::adapter::handler::AppState;
use crate::domain::entity::master_category::{CreateMasterCategory, UpdateMasterCategory};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::actor_from_claims;
use k1s0_auth::Claims;
use k1s0_server_common::ServiceError;
use serde::Deserialize;

/// 一覧取得用クエリパラメータ。
#[derive(Debug, Deserialize)]
pub struct ListCategoriesQuery {
    pub active_only: Option<bool>,
}

/// ヘルスチェックハンドラー。認証不要。
pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

/// 起動準備チェックハンドラー。PostgreSQL への疎通確認を行う。認証不要。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let postgres_ok = state
        .manage_categories_uc
        .list_categories(false)
        .await
        .is_ok();
    let status = if postgres_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(serde_json::json!({
            "status": if postgres_ok { "ready" } else { "not_ready" },
            "checks": {
                "postgres": if postgres_ok { "ok" } else { "error" }
            }
        })),
    )
}

/// Prometheus メトリクスハンドラー。認証不要。
pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

/// カテゴリ一覧取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn list_categories(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    // read 操作も認証が必要（gRPC ハンドラーと同等の認証強度を維持する）
    let _ = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let categories = state
        .manage_categories_uc
        .list_categories(query.active_only.unwrap_or(false))
        .await
        .map_err(from_anyhow)?;
    Ok(Json(serde_json::json!({ "categories": categories })))
}

/// カテゴリ単件取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn get_category(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(code): Path<String>,
) -> Result<impl IntoResponse, ServiceError> {
    // read 操作も認証が必要（gRPC ハンドラーと同等の認証強度を維持する）
    let _ = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let category = state
        .manage_categories_uc
        .get_category(&code)
        .await
        .map_err(from_anyhow)?
        .ok_or_else(|| ServiceError::NotFound {
            code: k1s0_server_common::ErrorCode::new("BIZ_DOMAINMASTER_CATEGORY_NOT_FOUND"),
            message: format!("Category '{}' not found", code),
        })?;
    Ok(Json(category))
}

/// カテゴリ作成ハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn create_category(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<CreateMasterCategory>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す。actor_from_claims は None 時に
    // "anonymous" を返すため、明示的な認証チェックが必要（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    let category = state
        .manage_categories_uc
        .create_category(&input, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok((StatusCode::CREATED, Json(category)))
}

/// カテゴリ更新ハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn update_category(
    State(state): State<AppState>,
    Path(code): Path<String>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<UpdateMasterCategory>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す。actor_from_claims は None 時に
    // "anonymous" を返すため、明示的な認証チェックが必要（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    let category = state
        .manage_categories_uc
        .update_category(&code, &actor, &input)
        .await
        .map_err(from_anyhow)?;
    Ok(Json(category))
}

/// カテゴリ削除ハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn delete_category(
    State(state): State<AppState>,
    Path(code): Path<String>,
    claims: Option<Extension<Claims>>,
) -> Result<StatusCode, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す。actor_from_claims は None 時に
    // "anonymous" を返すため、明示的な認証チェックが必要（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    state
        .manage_categories_uc
        .delete_category(&code, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok(StatusCode::NO_CONTENT)
}
