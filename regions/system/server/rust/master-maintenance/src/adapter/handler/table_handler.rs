use crate::adapter::handler::error::AppError;
use crate::adapter::handler::{actor_from_claims, publish_change_event, AppState};
use crate::domain::entity::table_definition::{CreateTableDefinition, UpdateTableDefinition};
use crate::domain::value_object::domain_filter::DomainFilter;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use k1s0_auth::Claims;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListTablesQuery {
    pub category: Option<String>,
    pub active_only: Option<bool>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub domain_scope: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DomainScopeQuery {
    pub domain_scope: Option<String>,
}

/// ヘルスチェックハンドラー。認証不要。
pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

/// 起動確認ハンドラー。認証不要。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let postgres_ok = state
        .manage_tables_uc
        .list_tables(None, false, &DomainFilter::All)
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
            // ADR-0068 準拠: "healthy"/"unhealthy" + timestamp
            "status": if postgres_ok { "healthy" } else { "unhealthy" },
            "checks": {
                "postgres": if postgres_ok { "ok" } else { "error" }
            },
            "timestamp": Utc::now().to_rfc3339()
        })),
    )
}

/// メトリクスハンドラー。認証不要。
pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

/// テーブル一覧ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn list_tables(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Query(query): Query<ListTablesQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let domain_filter = match &query.domain_scope {
        Some(ds) => DomainFilter::Domain(ds.clone()),
        None => DomainFilter::All,
    };
    let tables = state
        .manage_tables_uc
        .list_tables(
            query.category.as_deref(),
            query.active_only.unwrap_or(false),
            &domain_filter,
        )
        .await?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let total_count = tables.len() as u64;
    let start = ((page - 1) * page_size) as usize;
    let paged: Vec<_> = tables
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .collect();
    let has_next = (start + paged.len()) < total_count as usize;

    Ok(Json(serde_json::json!({
        "tables": paged,
        "pagination": {
            "total_count": total_count,
            "page": page,
            "page_size": page_size,
            "has_next": has_next
        }
    })))
}

/// テーブル取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn get_table(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Query(ds_query): Query<DomainScopeQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let table = state
        .manage_tables_uc
        .get_table(&name, ds_query.domain_scope.as_deref())
        .await?
        .ok_or_else(|| {
            AppError::not_found(
                "SYS_MM_TABLE_NOT_FOUND",
                &format!("Table '{name}' not found"),
            )
        })?;
    let columns = state
        .manage_columns_uc
        .list_columns(&name, ds_query.domain_scope.as_deref())
        .await?;

    // テーブル情報とカラム情報をマージしたJSONオブジェクトを構築する
    let mut payload = serde_json::json!(table);
    if let Some(object) = payload.as_object_mut() {
        object.insert("columns".to_string(), serde_json::json!(columns));
    }

    Ok(Json(payload))
}

/// テーブル作成ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn create_table(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<CreateTableDefinition>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let claims_ext = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims_ext.0));
    let table = state.manage_tables_uc.create_table(&input, &actor).await?;
    publish_change_event(
        &state,
        serde_json::json!({
            "event_type": "MASTER_MAINTENANCE_DATA_CHANGED",
            "resource_type": "table_definition",
            "resource_id": table.id,
            "resource_name": table.name,
            "action": "created",
            "actor": actor,
            "after": table.clone(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    )
    .await;
    Ok((StatusCode::CREATED, Json(table)))
}

/// テーブル更新ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn update_table(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Query(ds_query): Query<DomainScopeQuery>,
    Json(input): Json<UpdateTableDefinition>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let table = state
        .manage_tables_uc
        .update_table(&name, &input, ds_query.domain_scope.as_deref())
        .await?;
    Ok(Json(table))
}

/// テーブル削除ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn delete_table(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Query(ds_query): Query<DomainScopeQuery>,
) -> Result<StatusCode, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    state
        .manage_tables_uc
        .delete_table(&name, ds_query.domain_scope.as_deref())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// テーブルスキーマ取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn get_table_schema(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Query(ds_query): Query<DomainScopeQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let schema = state
        .manage_tables_uc
        .get_table_schema(&name, ds_query.domain_scope.as_deref())
        .await?;
    Ok(Json(schema))
}

/// カラム一覧ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn list_columns(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Query(ds_query): Query<DomainScopeQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let columns = state
        .manage_columns_uc
        .list_columns(&name, ds_query.domain_scope.as_deref())
        .await?;
    Ok(Json(columns))
}

/// カラム作成ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn create_columns(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Query(ds_query): Query<DomainScopeQuery>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let columns = state
        .manage_columns_uc
        .create_columns(&name, &input, ds_query.domain_scope.as_deref())
        .await?;
    Ok((StatusCode::CREATED, Json(columns)))
}

/// カラム更新ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn update_column(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((name, column)): Path<(String, String)>,
    Query(ds_query): Query<DomainScopeQuery>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let col = state
        .manage_columns_uc
        .update_column(&name, &column, &input, ds_query.domain_scope.as_deref())
        .await?;
    Ok(Json(col))
}

/// カラム削除ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn delete_column(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((name, column)): Path<(String, String)>,
    Query(ds_query): Query<DomainScopeQuery>,
) -> Result<StatusCode, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    state
        .manage_columns_uc
        .delete_column(&name, &column, ds_query.domain_scope.as_deref())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// ドメイン一覧ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn list_domains(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let domains = state.manage_tables_uc.list_domains().await?;
    let domain_list: Vec<serde_json::Value> = domains
        .into_iter()
        .map(|(domain_scope, table_count)| {
            serde_json::json!({
                "domain_scope": domain_scope,
                "table_count": table_count
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "domains": domain_list })))
}
