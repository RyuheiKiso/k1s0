// ヘルスチェックエンドポイント。
// MEDIUM-001 監査対応: readyz で DB 疎通確認（SELECT 1）を行い、DB 障害を K8s readiness probe で検知できるようにする。
// task-rust と同一パターンで実装する。
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use super::AppState;

/// Liveness probe
pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// Readiness probe（DB 接続確認）
/// ADR-0068 対応: "ready" から "healthy" に統一し、timestamp フィールドを追加する
/// MEDIUM-001 監査対応: SELECT 1 で DB 疎通を確認し、失敗時は HTTP 503 を返す
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // DB 疎通確認には単純な SELECT 1 を使用する。
    // ビジネスロジック経由の確認は RLS や UC の副作用の影響を受けるため禁止する。
    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す
    let timestamp = chrono::Utc::now().to_rfc3339();
    match sqlx::query("SELECT 1").execute(&state.db_pool).await {
        Ok(_) => (
            StatusCode::OK,
            // ADR-0068 対応: "ready"/"not_ready" から "healthy"/"unhealthy" に統一する
            Json(json!({
                "status": "healthy",
                "checks": { "postgres": "ok" },
                "timestamp": timestamp
            })),
        ),
        Err(e) => {
            tracing::error!(error = %e, "readyz: DB ping failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "unhealthy",
                    "checks": { "postgres": "error" },
                    "timestamp": timestamp
                })),
            )
        }
    }
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
