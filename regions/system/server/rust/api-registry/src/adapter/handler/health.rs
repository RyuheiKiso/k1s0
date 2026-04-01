use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// readiness probe: DB 接続確認を行い、サービスがリクエスト受付可能かを返す。
/// CRITICAL-003 監査対応: Docker Compose の healthcheck および Kubernetes readinessProbe として使用する。
/// DB が設定されている場合は SELECT 1 で疎通確認し、失敗時は 503 を返す。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => {
                Json(serde_json::json!({"status": "ready", "database": "connected"})).into_response()
            }
            Err(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "status": "not_ready",
                    "database": e.to_string()
                })),
            )
                .into_response(),
        }
    } else {
        Json(serde_json::json!({"status": "ready", "database": "not_configured"})).into_response()
    }
}

pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
