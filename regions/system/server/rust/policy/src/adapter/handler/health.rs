use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

/// ヘルスチェックエンドポイント: サーバーが起動しているかを返す
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// レディネスチェックエンドポイント: ADR-0068 準拠形式で返す（MED-008 監査対応）。
/// in-memory バックエンド使用時は degraded ステータスを返して
/// 永続化バックエンド未構成であることを運用チームに明示する。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // バックエンド種別に応じてステータスを決定（ADR-0068 標準: healthy/degraded/unhealthy）
    let status = if state.backend_kind == "in-memory" {
        "degraded"
    } else {
        "healthy"
    };

    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す
    let timestamp = chrono::Utc::now().to_rfc3339();
    Json(serde_json::json!({
        "status": status,
        "checks": {
            "policy_backend": state.backend_kind
        },
        "timestamp": timestamp
    }))
}
