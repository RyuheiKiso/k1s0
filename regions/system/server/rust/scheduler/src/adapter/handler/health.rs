use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

/// liveness probe: プロセスが起動していれば常に ok を返す。
/// Kubernetes の livenessProbe として使用する。
/// CRITICAL-003 監査対応: DB 確認は readyz に移動し、healthz は liveness のみとする。
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// readiness probe: DB 接続確認を行い、サービスがリクエスト受付可能かを返す。
/// CRITICAL-003 監査対応: Docker Compose の healthcheck および Kubernetes の readinessProbe として使用する。
/// DB が設定されている場合は SELECT 1 で疎通確認し、失敗時は 503 を返す。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => {
                Json(serde_json::json!({"status": "ready", "database": "connected"})).into_response()
            }
            Err(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "status": "not_ready",
                    "database": e.to_string()
                })),
            )
                .into_response(),
        }
    } else {
        Json(serde_json::json!({"status": "ready", "database": "not_configured"})).into_response()
    }
}
