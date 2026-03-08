use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

pub async fn readyz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ready"})))
}

pub async fn metrics_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let output = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; charset=utf-8",
        )],
        output,
    )
}
