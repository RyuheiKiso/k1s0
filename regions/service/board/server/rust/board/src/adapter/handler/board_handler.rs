// ボードカラム REST ハンドラー。
// BSL-CRIT-002 監査対応: Claims が存在しない場合は 401 Unauthorized を返す。
// 旧実装の tenant_id_from_claims は Claims==None 時に "system" を返しており、
// RLS テナント分離をバイパスするセキュリティ問題があったため削除した。
// Claims 拡張から認証ユーザー情報を取得してユースケースに渡す。
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

// MED-07 監査対応: リスト操作のデフォルト上限と最大上限を定数として定義する。
// リクエストが上限を指定しない場合はデフォルト値を使用し、
// 上限を超えた値が指定された場合は最大値でクランプする。
const LIST_DEFAULT_LIMIT: i64 = 50;
const LIST_MAX_LIMIT: i64 = 100;

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
    // MED-07 監査対応: limit が未指定の場合はデフォルト値を使用し、1〜最大値の範囲にクランプする
    let limit = q
        .limit
        .unwrap_or(LIST_DEFAULT_LIMIT)
        .clamp(1, LIST_MAX_LIMIT);
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("BOARD", "Authentication required"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    let filter = BoardColumnFilter {
        project_id: q.project_id,
        status_code: q.status_code,
        limit: Some(limit),
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
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("BOARD", "Authentication required"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
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
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("BOARD", "Authentication required"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    let col = state.increment_column_uc.execute(tenant_id, &req).await.map_err(map_err)?;
    Ok((StatusCode::OK, Json(col)))
}

pub async fn decrement_column(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Json(req): Json<DecrementColumnRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("BOARD", "Authentication required"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    let col = state.decrement_column_uc.execute(tenant_id, &req).await.map_err(map_err)?;
    Ok((StatusCode::OK, Json(col)))
}

pub async fn update_wip_limit(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path(column_id): Path<Uuid>,
    Json(mut req): Json<UpdateWipLimitRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("BOARD", "Authentication required"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    req.column_id = column_id;
    let col = state.update_wip_limit_uc.execute(tenant_id, &req).await.map_err(map_err)?;
    Ok(Json(col))
}
