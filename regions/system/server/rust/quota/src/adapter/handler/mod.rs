pub mod health;
pub mod quota_handler;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::Router;

use crate::usecase::{
    CreateQuotaPolicyUseCase, GetQuotaPolicyUseCase, GetQuotaUsageUseCase,
    IncrementQuotaUsageUseCase, ListQuotaPoliciesUseCase, UpdateQuotaPolicyUseCase,
};

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub create_policy_uc: Arc<CreateQuotaPolicyUseCase>,
    pub get_policy_uc: Arc<GetQuotaPolicyUseCase>,
    pub list_policies_uc: Arc<ListQuotaPoliciesUseCase>,
    pub update_policy_uc: Arc<UpdateQuotaPolicyUseCase>,
    pub get_usage_uc: Arc<GetQuotaUsageUseCase>,
    pub increment_usage_uc: Arc<IncrementQuotaUsageUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
}

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler))
        .route("/api/v1/quotas", get(quota_handler::list_quotas))
        .route("/api/v1/quotas", post(quota_handler::create_quota))
        .route("/api/v1/quotas/:id", get(quota_handler::get_quota))
        .route("/api/v1/quotas/:id", put(quota_handler::update_quota))
        .route(
            "/api/v1/quotas/:id/check",
            post(quota_handler::check_quota),
        )
        .with_state(state)
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
