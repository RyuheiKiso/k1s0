use axum::response::IntoResponse;
use axum::Json;

#[allow(dead_code)]
// async は不要（.await 呼び出しがないため同期関数に変換する）
#[must_use]
pub fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "tenant"}))
}
