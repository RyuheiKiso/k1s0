// ボードカラム REST ハンドラー。
// Claims 拡張から認証ユーザー情報を取得してユースケースに渡す。
// RLS テナント分離のため Claims の iss（発行者）を tenant_id として使用する。
// iss が空の場合は "system" をデフォルト値として使用する。
use crate::adapter::handler::AppState;
use crate::domain::entity::board_column::{
    BoardColumnFilter, DecrementColumnRequest, IncrementColumnRequest, UpdateWipLimitRequest,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::Claims;
use k1s0_server_common::ServiceError;
use serde::Deserialize;
use uuid::Uuid;

fn map_err(e: anyhow::Error) -> ServiceError {
    ServiceError::Internal {
        code: k1s0_server_common::ErrorCode::new("SVC_BOARD_ERROR"),
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
pub struct ListBoardColumnsQuery {
    pub project_id: Option<Uuid>,
    pub status_code: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_board_columns(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Query(q): Query<ListBoardColumnsQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|ext| &ext.0));
    let filter = BoardColumnFilter {
        project_id: q.project_id,
        status_code: q.status_code,
        limit: q.limit,
        offset: q.offset,
    };
    let (cols, total) = state.list_board_columns_uc.execute(tenant_id, &filter).await.map_err(map_err)?;
    Ok(Json(serde_json::json!({ "columns": cols, "total": total })))
}

pub async fn get_board_column(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|ext| &ext.0));
    let col = state
        .get_board_column_uc
        .execute(tenant_id, id)
        .await
        .map_err(map_err)?
        .ok_or_else(|| ServiceError::NotFound {
            code: k1s0_server_common::ErrorCode::new("SVC_BOARD_COLUMN_NOT_FOUND"),
            message: format!("BoardColumn '{}' not found", id),
        })?;
    Ok(Json(col))
}

pub async fn increment_column(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Json(req): Json<IncrementColumnRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|ext| &ext.0));
    let col = state.increment_column_uc.execute(tenant_id, &req).await.map_err(map_err)?;
    Ok((StatusCode::OK, Json(col)))
}

pub async fn decrement_column(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Json(req): Json<DecrementColumnRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|ext| &ext.0));
    let col = state.decrement_column_uc.execute(tenant_id, &req).await.map_err(map_err)?;
    Ok((StatusCode::OK, Json(col)))
}

pub async fn update_wip_limit(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path(column_id): Path<Uuid>,
    Json(mut req): Json<UpdateWipLimitRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|ext| &ext.0));
    req.column_id = column_id;
    let col = state.update_wip_limit_uc.execute(tenant_id, &req).await.map_err(map_err)?;
    Ok(Json(col))
}
