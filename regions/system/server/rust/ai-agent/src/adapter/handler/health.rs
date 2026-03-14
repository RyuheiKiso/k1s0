// ヘルスチェックハンドラ
// サーバーの生存確認と準備状態を返すエンドポイント

use axum::response::IntoResponse;
use axum::Json;

/// ヘルスチェックエンドポイント: サーバーが生存しているかを確認する
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// レディネスチェックエンドポイント: サーバーがリクエストを受け付け可能かを確認する
pub async fn readyz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ready"}))
}
