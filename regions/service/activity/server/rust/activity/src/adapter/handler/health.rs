use crate::adapter::handler::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let filter = crate::domain::entity::activity::ActivityFilter::default();
    // ヘルスチェックではテナント分離不要のため固定の tenant_id を渡す
    let ok = state.list_activities_uc.execute("health", &filter).await.is_ok();
    let status = if ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (status, Json(serde_json::json!({ "status": if ok { "ready" } else { "not_ready" } })))
}

pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (StatusCode::OK, [("content-type", "text/plain; version=0.0.4; charset=utf-8")], body)
}
