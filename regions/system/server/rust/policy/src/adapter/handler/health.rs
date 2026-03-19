use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

/// ヘルスチェックエンドポイント: サーバーが起動しているかを返す
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// レディネスチェックエンドポイント: バックエンド種別を含む応答を返す。
/// in-memory バックエンド使用時は degraded ステータスを返して
/// 永続化バックエンド未構成であることを運用チームに明示する。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // バックエンド種別に応じてステータスを決定
    let status = if state.backend_kind == "in-memory" {
        "degraded"
    } else {
        "ok"
    };

    Json(serde_json::json!({
        "status": status,
        "service": "policy",
        "backend": state.backend_kind
    }))
}
