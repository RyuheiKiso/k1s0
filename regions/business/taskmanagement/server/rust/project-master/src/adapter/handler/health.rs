// ヘルスチェックエンドポイント。
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use super::AppState;

/// Liveness probe
pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// Readiness probe（DB 接続確認）
/// ADR-0068 対応: "ready" から "healthy" に統一し、timestamp フィールドを追加する
pub async fn readyz(State(_state): State<AppState>) -> impl IntoResponse {
    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す
    let timestamp = chrono::Utc::now().to_rfc3339();
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "checks": {},
            "timestamp": timestamp
        })),
    )
}

/// Prometheus メトリクスエンドポイント
pub async fn metrics_handler() -> impl IntoResponse {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    // MED-015 監査対応: エンコードエラーをサイレント無視せずログ出力する
    // unwrap_or_default() はエラーを完全に無視するため、検知不能な障害を引き起こす可能性がある
    encoder
        .encode(&prometheus::gather(), &mut buffer)
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "prometheus metrics encode failed");
        });
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        buffer,
    )
}
