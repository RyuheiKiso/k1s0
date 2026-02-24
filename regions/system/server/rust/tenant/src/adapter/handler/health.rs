use axum::response::IntoResponse;
use axum::Json;

pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "tenant"}))
}
