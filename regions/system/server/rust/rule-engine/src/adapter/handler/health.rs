use axum::response::IntoResponse;
use axum::Json;

pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn readyz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ready"}))
}
