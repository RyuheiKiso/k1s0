use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use super::{AppState, ErrorResponse};
use crate::domain::entity::app::App;
use crate::usecase::create_app::CreateAppInput;
use crate::usecase::get_download_stats::DownloadStatsSummary;
use crate::usecase::update_app::UpdateAppInput;

/// アプリ一覧レスポンス
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AppListResponse {
    pub apps: Vec<App>,
}

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Health check OK"),
    )
)]
/// ヘルスチェックエンドポイント
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

#[utoipa::path(
    get,
    path = "/readyz",
    responses(
        (status = 200, description = "Ready"),
        (status = 503, description = "Not ready"),
    )
)]
/// レディネスチェックエンドポイント（DB 疎通確認を含む）
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let mut db_status = "skipped";
    let mut overall_ok = true;

    // DB check
    if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => db_status = "ok",
            Err(_) => {
                db_status = "error";
                overall_ok = false;
            }
        }
    }

    let status_code = if overall_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status_code,
        Json(serde_json::json!({
            "status": if overall_ok { "ready" } else { "not ready" },
            "checks": {
                "database": db_status
            }
        })),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics"),
    )
)]
/// Prometheus メトリクスエンドポイント
pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

/// アプリ一覧取得のクエリパラメータ。
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListAppsQuery {
    pub category: Option<String>,
    pub search: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/apps",
    params(
        ("category" = Option<String>, Query, description = "Filter by category"),
        ("search" = Option<String>, Query, description = "Search by name or description"),
    ),
    responses(
        (status = 200, description = "App list", body = AppListResponse),
    ),
    security(("bearer_auth" = []))
)]
/// アプリ一覧取得ハンドラー（カテゴリ・検索文字列でフィルタリング可能）
pub async fn list_apps(
    State(state): State<AppState>,
    Query(params): Query<ListAppsQuery>,
) -> impl IntoResponse {
    match state
        .list_apps_uc
        .execute(params.category, params.search)
        .await
    {
        Ok(apps) => (StatusCode::OK, Json(AppListResponse { apps })).into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/apps/{id}/stats",
    params(("id" = String, Path, description = "App ID")),
    responses(
        (status = 200, description = "Download stats", body = DownloadStatsSummary),
        (status = 404, description = "App not found"),
    ),
    security(("bearer_auth" = []))
)]
/// アプリのダウンロード統計取得ハンドラー
pub async fn get_download_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.get_download_stats_uc.execute(&id).await {
        Ok(stats) => (StatusCode::OK, Json(stats)).into_response(),
        Err(crate::usecase::get_download_stats::GetDownloadStatsError::AppNotFound(_)) => {
            let err =
                ErrorResponse::new("SYS_APPS_APP_NOT_FOUND", "The specified app was not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/apps/{id}",
    params(("id" = String, Path, description = "App ID")),
    responses(
        (status = 200, description = "App found", body = App),
        (status = 404, description = "App not found"),
    ),
    security(("bearer_auth" = []))
)]
/// アプリ詳細取得ハンドラー
pub async fn get_app(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.get_app_uc.execute(&id).await {
        Ok(app) => (StatusCode::OK, Json(app)).into_response(),
        Err(crate::usecase::get_app::GetAppError::NotFound(_)) => {
            let err =
                ErrorResponse::new("SYS_APPS_APP_NOT_FOUND", "The specified app was not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// アプリ新規登録のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateAppRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub icon_url: Option<String>,
}

/// アプリ更新のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateAppRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub icon_url: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/apps",
    request_body = CreateAppRequest,
    responses(
        (status = 201, description = "App created", body = App),
        (status = 400, description = "Validation error"),
    ),
    security(("bearer_auth" = []))
)]
/// アプリ新規作成ハンドラー
pub async fn create_app(
    State(state): State<AppState>,
    Json(req): Json<CreateAppRequest>,
) -> impl IntoResponse {
    let input = CreateAppInput {
        name: req.name,
        description: req.description,
        category: req.category,
        icon_url: req.icon_url,
    };

    match state.create_app_uc.execute(input).await {
        Ok(app) => (
            StatusCode::CREATED,
            Json(app),
        )
            .into_response(),
        Err(crate::usecase::CreateAppError::ValidationError(msg)) => {
            let err = ErrorResponse::new("SYS_APPS_VALIDATION_ERROR", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/apps/{id}",
    params(("id" = String, Path, description = "App ID")),
    request_body = UpdateAppRequest,
    responses(
        (status = 200, description = "App updated", body = App),
        (status = 404, description = "App not found"),
        (status = 400, description = "Validation error"),
    ),
    security(("bearer_auth" = []))
)]
/// アプリ更新ハンドラー
pub async fn update_app(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAppRequest>,
) -> impl IntoResponse {
    let input = UpdateAppInput {
        id,
        name: req.name,
        description: req.description,
        category: req.category,
        icon_url: req.icon_url,
    };

    match state.update_app_uc.execute(input).await {
        Ok(app) => (StatusCode::OK, Json(app)).into_response(),
        Err(crate::usecase::UpdateAppError::NotFound(_)) => {
            let err =
                ErrorResponse::new("SYS_APPS_APP_NOT_FOUND", "The specified app was not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::UpdateAppError::ValidationError(msg)) => {
            let err = ErrorResponse::new("SYS_APPS_VALIDATION_ERROR", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/apps/{id}",
    params(("id" = String, Path, description = "App ID")),
    responses(
        (status = 204, description = "App deleted"),
        (status = 404, description = "App not found"),
    ),
    security(("bearer_auth" = []))
)]
/// アプリ削除ハンドラー
pub async fn delete_app(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.delete_app_uc.execute(&id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(crate::usecase::DeleteAppError::NotFound(_)) => {
            let err =
                ErrorResponse::new("SYS_APPS_APP_NOT_FOUND", "The specified app was not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
