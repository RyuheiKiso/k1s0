use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

/// システムテナントUUID: readyz の DB 疎通確認に使用するフォールバックテナントID。
/// STATIC-CRITICAL-001 監査対応: find_all の第1引数として使用する。
/// HIGH-005 対応: &str 型で直接使用する（migration 006 で DB の TEXT 型に変更済み）。
const SYSTEM_TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";

/// MED-001 監査対応: StatusCode を明示的に返すタプルパターンを使用し、
/// Content-Type: application/json と HTTP 200 が確実に返されることを保証する
pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // DB 疎通確認はシステムテナント UUID でフォールバックする（ADR-0028 Phase 1）
    // HIGH-005 対応: &str 型で直接渡す（Uuid::parse_str 不要）
    // MED-001 対応: .is_ok() でエラーを握り潰さず tracing::error! で詳細を記録する
    let db_ok = match state.flag_repo.find_all(SYSTEM_TENANT_ID).await {
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

    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す
    let timestamp = chrono::Utc::now().to_rfc3339();
    (
        if ready {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        // ADR-0068 対応: "ready"/"not_ready" から "healthy"/"unhealthy" に統一する
        Json(serde_json::json!({
            "status": if ready { "healthy" } else { "unhealthy" },
            "checks": {
                "database": if db_ok { "ok" } else { "error" },
                "kafka": if kafka_ok { "ok" } else { "error" }
            },
            "timestamp": timestamp
        })),
    )
}
