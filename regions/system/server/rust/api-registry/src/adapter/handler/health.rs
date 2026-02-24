use axum::response::IntoResponse;
use axum::Json;

pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn readyz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ready"}))
}

pub async fn metrics() -> impl IntoResponse {
    // Prometheus メトリクス（テキスト形式）を返す
    // k1s0-telemetry 統合時に Metrics::encode() で置き換え
    axum::response::Response::builder()
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(axum::body::Body::from("# k1s0-api-registry-server metrics\n"))
        .unwrap()
}
