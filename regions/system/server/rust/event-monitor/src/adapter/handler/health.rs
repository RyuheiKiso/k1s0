use crate::adapter::handler::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す
    let timestamp = chrono::Utc::now().to_rfc3339();

    // DB 疎通確認: AppState に db_pool が存在する場合のみ実行する。
    // in-memory モード（db_pool = None）ではスキップし "skipped" を返す。
    // ビジネスロジック経由の確認は RLS や UC の副作用の影響を受けるため直接 SELECT 1 を使用する。
    let (db_status, overall_ok) = if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool.as_ref()).await {
            Ok(_) => ("ok", true),
            Err(e) => {
                tracing::error!(error = %e, "readyz: DB ping failed");
                ("error", false)
            }
        }
    } else {
        ("skipped", true)
    };

    // ADR-0068 対応: "ready"/"not_ready" から "healthy"/"unhealthy" に統一する
    if overall_ok {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "healthy",
                "checks": { "postgres": db_status },
                "timestamp": timestamp
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "unhealthy",
                "checks": { "postgres": db_status },
                "timestamp": timestamp
            })),
        )
    }
}

// mod.rs の metrics_handler が使用されるため、この実装は参照されない
#[allow(dead_code)]
// async は不要（.await 呼び出しがないため同期関数に変換する）
#[must_use]
pub fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
