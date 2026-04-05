use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // MED-001 対応: .is_ok() でエラーを握り潰さず tracing::error! で詳細を記録する
    let db_ok = match state.stream_repo.list_all(1, 1).await {
        Ok(_) => true,
        Err(e) => {
            tracing::error!(error = %e, "readyz: DB health check failed");
            false
        }
    };
    let kafka_ok = match state.event_publisher.health_check().await {
        Ok(_) => true,
        Err(e) => {
            tracing::error!(error = %e, "readyz: Kafka health check failed");
            false
        }
    };
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
