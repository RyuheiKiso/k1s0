use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use super::{AppState, ErrorResponse};
use crate::domain::entity::app::App;

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Health check OK"),
    )
)]
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
        (status = 200, description = "App list", body = Vec<App>),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_apps(
    State(state): State<AppState>,
    Query(params): Query<ListAppsQuery>,
) -> impl IntoResponse {
    match state
        .list_apps_uc
        .execute(params.category.as_deref(), params.search.as_deref())
        .await
    {
        Ok(apps) => (StatusCode::OK, Json(serde_json::to_value(apps).unwrap())).into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", &e.to_string());
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
pub async fn get_app(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.get_app_uc.execute(&id).await {
        Ok(app) => (StatusCode::OK, Json(serde_json::to_value(app).unwrap())).into_response(),
        Err(crate::usecase::get_app::GetAppError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_APPS_APP_NOT_FOUND",
                "The specified app was not found",
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
