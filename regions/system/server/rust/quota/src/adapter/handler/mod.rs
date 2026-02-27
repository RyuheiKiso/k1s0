pub mod health;
pub mod quota_handler;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::Router;

use crate::adapter::middleware::auth::{auth_middleware, QuotaAuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::usecase::{
    CreateQuotaPolicyUseCase, DeleteQuotaPolicyUseCase, GetQuotaPolicyUseCase,
    GetQuotaUsageUseCase, IncrementQuotaUsageUseCase, ListQuotaPoliciesUseCase,
    ResetQuotaUsageUseCase, UpdateQuotaPolicyUseCase,
};

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub create_policy_uc: Arc<CreateQuotaPolicyUseCase>,
    pub get_policy_uc: Arc<GetQuotaPolicyUseCase>,
    pub list_policies_uc: Arc<ListQuotaPoliciesUseCase>,
    pub update_policy_uc: Arc<UpdateQuotaPolicyUseCase>,
    pub delete_policy_uc: Arc<DeleteQuotaPolicyUseCase>,
    pub get_usage_uc: Arc<GetQuotaUsageUseCase>,
    pub increment_usage_uc: Arc<IncrementQuotaUsageUseCase>,
    pub reset_usage_uc: Arc<ResetQuotaUsageUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<QuotaAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: QuotaAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler));

    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET/check/usage -> quotas/read
        let read_routes = Router::new()
            .route(
                "/api/v1/quotas",
                get(quota_handler::list_quotas),
            )
            .route(
                "/api/v1/quotas/:id",
                get(quota_handler::get_quota),
            )
            .route(
                "/api/v1/quotas/:id/check",
                post(quota_handler::check_quota),
            )
            .route(
                "/api/v1/quotas/:id/usage",
                get(quota_handler::get_usage),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "quotas", "read",
            )));

        // POST/PUT/increment -> quotas/write
        let write_routes = Router::new()
            .route(
                "/api/v1/quotas",
                post(quota_handler::create_quota),
            )
            .route(
                "/api/v1/quotas/:id",
                put(quota_handler::update_quota),
            )
            .route(
                "/api/v1/quotas/:id/usage/increment",
                post(quota_handler::increment_usage),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "quotas", "write",
            )));

        // DELETE/reset -> quotas/admin
        let admin_routes = Router::new()
            .route(
                "/api/v1/quotas/:id",
                axum::routing::delete(quota_handler::delete_quota),
            )
            .route(
                "/api/v1/quotas/:id/usage/reset",
                post(quota_handler::reset_usage),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "quotas", "admin",
            )));

        Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .merge(admin_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
    } else {
        Router::new()
            .route(
                "/api/v1/quotas",
                get(quota_handler::list_quotas).post(quota_handler::create_quota),
            )
            .route(
                "/api/v1/quotas/:id",
                get(quota_handler::get_quota)
                    .put(quota_handler::update_quota)
                    .delete(quota_handler::delete_quota),
            )
            .route(
                "/api/v1/quotas/:id/check",
                post(quota_handler::check_quota),
            )
            .route(
                "/api/v1/quotas/:id/usage",
                get(quota_handler::get_usage),
            )
            .route(
                "/api/v1/quotas/:id/usage/increment",
                post(quota_handler::increment_usage),
            )
            .route(
                "/api/v1/quotas/:id/usage/reset",
                post(quota_handler::reset_usage),
            )
    };

    public_routes
        .merge(api_routes)
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
