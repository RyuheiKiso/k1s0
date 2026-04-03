use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use super::AppState;

/// システムテナントUUID: readyz の DB 疎通確認に使用するフォールバックテナントID。
/// STATIC-CRITICAL-001 監査対応: find_all の第1引数として使用する。
const SYSTEM_TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";

pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // DB 疎通確認はシステムテナント UUID でフォールバックする（ADR-0028 Phase 1）
    let system_tenant =
        Uuid::parse_str(SYSTEM_TENANT_ID).expect("SYSTEM_TENANT_ID は有効な UUID である");
    let db_ok = state.flag_repo.find_all(system_tenant).await.is_ok();
    let kafka_ok = state.event_publisher.health_check().await.is_ok();
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
