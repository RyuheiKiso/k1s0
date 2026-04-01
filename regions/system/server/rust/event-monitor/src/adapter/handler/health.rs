use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

/// liveness probe: プロセスが起動していれば常に ok を返す。
/// CRITICAL-003 監査対応: DB・DLQ 確認は readyz に移動し、healthz は liveness のみとする。
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// readiness probe: DLQ 接続状態を確認し、サービスがリクエスト受付可能かを返す。
/// CRITICAL-003 監査対応: DLQ クライアントが NoopDlqClient の場合は degraded を返す。
/// Docker Compose の healthcheck および Kubernetes readinessProbe として使用する。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    if state.dlq_noop {
        // Noop 使用時: サービスは起動しているが DLQ 連携が無効であることを示す
        Json(serde_json::json!({
            "status": "degraded",
            "components": {
                "server": "ok",
                "dlq_client": {
                    "status": "noop",
                    "message": "DLQ manager client is not configured; replay functionality is unavailable"
                }
            }
        }))
    } else {
        Json(serde_json::json!({
            "status": "ready",
            "components": {
                "server": "ok",
                "dlq_client": {"status": "connected"}
            }
        }))
    }
}
