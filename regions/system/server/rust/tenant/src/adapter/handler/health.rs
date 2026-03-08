use axum::response::IntoResponse;
use axum::Json;

#[allow(dead_code)]
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "tenant"}))
}
