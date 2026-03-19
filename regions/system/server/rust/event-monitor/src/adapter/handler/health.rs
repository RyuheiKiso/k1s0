use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

/// ヘルスチェックエンドポイント。
/// DLQ クライアントが NoopDlqClient の場合は degraded ステータスを返し、
/// replay 機能が利用不可であることを明示する。
pub async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    if state.dlq_noop {
        // Noop 使用時: サーバーは動作しているが DLQ 連携が無効であることを示す
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
            "status": "ok",
            "components": {
                "server": "ok",
                "dlq_client": {"status": "connected"}
            }
        }))
    }
}

/// レディネスチェックエンドポイント。サーバーがリクエストを受付可能かを返す。
pub async fn readyz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ready"}))
}
