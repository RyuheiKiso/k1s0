// ヘルスチェックハンドラー。
// サーバーの稼働状態と準備状態を返す。

use axum::response::IntoResponse;
use axum::Json;

/// ライブネスプローブ。サーバーが稼働中であることを返す。
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// レディネスプローブ。サーバーがリクエストを受け付ける準備ができていることを返す。
pub async fn readyz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ready"}))
}
