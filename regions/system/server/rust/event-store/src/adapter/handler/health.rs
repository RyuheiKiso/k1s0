use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = state.stream_repo.list_all(1, 1).await.is_ok();
    let kafka_ok = state.event_publisher.health_check().await.is_ok();
    let ready = db_ok && kafka_ok;
    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す（タプル式内での let 宣言は Rust 文法違反のため外に出す）
    let timestamp = chrono::Utc::now().to_rfc3339();

    (
        if ready {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        Json(serde_json::json!({
            // ADR-0068 対応: "ready"/"not_ready" から "healthy"/"unhealthy" に統一する
            "status": if ready { "healthy" } else { "unhealthy" },
            "checks": {
                "database": if db_ok { "ok" } else { "error" },
                "kafka": if kafka_ok { "ok" } else { "error" }
            },
            "timestamp": timestamp
        })),
    )
}
