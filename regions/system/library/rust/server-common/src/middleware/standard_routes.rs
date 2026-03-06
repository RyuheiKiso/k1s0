use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

use k1s0_health::CompositeHealthChecker;
use k1s0_telemetry::metrics::Metrics;

/// merge_standard_routes は標準エンドポイントをRouterにマージする。
pub fn merge_standard_routes(
    router: Router,
    metrics: Option<Arc<Metrics>>,
    health_checker: Option<Arc<CompositeHealthChecker>>,
) -> Router {
    // /healthz は常に追加
    let router = router.route("/healthz", get(healthz));

    // /readyz は health_checker 設定時のみ
    let router = if let Some(checker) = health_checker {
        router.route(
            "/readyz",
            get(move || {
                let checker = checker.clone();
                async move { readyz(checker).await }
            }),
        )
    } else {
        router
    };

    // /metrics は metrics 設定時のみ
    if let Some(metrics) = metrics {
        router.route(
            "/metrics",
            get(move || {
                let metrics = metrics.clone();
                async move { metrics.gather_metrics() }
            }),
        )
    } else {
        router
    }
}

async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

async fn readyz(checker: Arc<CompositeHealthChecker>) -> impl IntoResponse {
    let response = checker.readyz().await;
    let status = match response.status {
        k1s0_health::HealthStatus::Healthy => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status, Json(response))
}
