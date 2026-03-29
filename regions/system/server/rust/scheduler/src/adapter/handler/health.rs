use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

/// ヘルスチェックエンドポイント: DB 接続確認込みで稼働状態を応答する（C-02 対応）
/// DB が設定されている場合は SELECT 1 で疎通確認し、失敗時は 503 を返す
pub async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => Json(serde_json::json!({"status": "ok", "database": "connected"}))
                .into_response(),
            Err(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "status": "unhealthy",
                    "database": e.to_string()
                })),
            )
                .into_response(),
        }
    } else {
        Json(serde_json::json!({"status": "ok", "database": "not_configured"})).into_response()
    }
}

/// レディネスチェックエンドポイント: サービスが起動してリクエスト受付可能かを応答する
pub async fn readyz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ready"}))
}
