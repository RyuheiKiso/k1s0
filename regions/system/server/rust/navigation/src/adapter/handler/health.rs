use axum::response::IntoResponse;
use axum::Json;

pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

// ADR-0068 対応: "ready"/"not_ready" から "healthy"/"unhealthy" に統一する
// timestamp フィールドを追加して監視ダッシュボードでの時系列確認を可能にする
pub async fn readyz() -> impl IntoResponse {
    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す
    let timestamp = chrono::Utc::now().to_rfc3339();
    Json(serde_json::json!({
        "status": "healthy",
        "checks": {},
        "timestamp": timestamp
    }))
}
