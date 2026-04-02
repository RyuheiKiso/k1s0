use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

/// ヘルスチェックエンドポイント: サーバーが起動しているかを返す
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// レディネスチェックエンドポイント: ADR-0068 準拠形式で返す（MED-007/MED-008 監査対応）。
/// in-memory バックエンド使用時は degraded ステータスを返して
/// 永続化バックエンド未構成であることを運用チームに明示する。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // ADR-0068 標準: healthy / degraded / unhealthy の3値を使用する
    let status = if state.backend_kind == "in-memory" {
        "degraded"
    } else {
        "healthy"
    };

    Json(serde_json::json!({
        "status": status,
        "checks": {
            "rule_engine_backend": state.backend_kind
        }
    }))
}
