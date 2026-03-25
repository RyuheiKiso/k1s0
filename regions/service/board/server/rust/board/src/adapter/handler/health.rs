// ヘルスチェックハンドラー。
use crate::adapter::handler::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // DB 疎通確認には単純な SELECT 1 を使用する。
    // ビジネスロジック経由の確認は RLS や UC の副作用の影響を受けるため禁止する。
    match sqlx::query("SELECT 1").execute(&state.db_pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ready",
                "checks": { "postgres": "ok" }
            })),
        ),
        Err(e) => {
            tracing::error!(error = %e, "readyz: DB ping failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "status": "not_ready",
                    "checks": { "postgres": "error" }
                })),
            )
        }
    }
}

pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (StatusCode::OK, [("content-type", "text/plain; version=0.0.4; charset=utf-8")], body)
}
