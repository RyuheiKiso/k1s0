// ヘルスチェックハンドラー。
use crate::adapter::handler::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let filter = crate::domain::entity::task::TaskFilter::default();
    let ok = state.list_tasks_uc.execute(&filter).await.is_ok();
    let status = if ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (status, Json(serde_json::json!({
        "status": if ok { "ready" } else { "not_ready" },
        "checks": { "postgres": if ok { "ok" } else { "error" } }
    })))
}

pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (StatusCode::OK, [("content-type", "text/plain; version=0.0.4; charset=utf-8")], body)
}
