use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

/// liveness probe: プロセスが起動していれば常に ok を返す。
/// CRITICAL-003 監査対応: DB 確認は readyz に移動し、healthz は liveness のみとする。
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// readiness probe: DB 接続確認を行い、サービスがリクエスト受付可能かを返す。
/// CRITICAL-003 監査対応: Docker Compose の healthcheck および Kubernetes readinessProbe として使用する。
/// ADR-0068 対応: "ready"/"not_ready" から "healthy"/"unhealthy" に統一し、timestamp フィールドを追加する。
/// DB が設定されている場合は SELECT 1 で疎通確認し、失敗時は 503 を返す。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す
    let timestamp = chrono::Utc::now().to_rfc3339();
    if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => Json(serde_json::json!({
                // ADR-0068 対応: "ready"/"not_ready" から "healthy"/"unhealthy" に統一する
                "status": "healthy",
                "checks": { "database": "ok" },
                "timestamp": timestamp
            }))
            .into_response(),
            Err(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "status": "unhealthy",
                    "checks": { "database": format!("error: {}", e) },
                    "timestamp": timestamp
                })),
            )
                .into_response(),
        }
    } else {
        Json(serde_json::json!({
            "status": "healthy",
            "checks": { "database": "not_configured" },
            "timestamp": timestamp
        }))
        .into_response()
    }
}
