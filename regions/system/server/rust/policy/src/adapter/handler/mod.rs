pub mod health;
pub mod policy_handler;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::adapter::middleware::auth::{auth_middleware, PolicyAuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::domain::repository::PolicyRepository;
use crate::usecase::{
    CreateBundleUseCase, CreatePolicyUseCase, DeletePolicyUseCase, EvaluatePolicyUseCase,
    GetPolicyUseCase, ListBundlesUseCase, UpdatePolicyUseCase,
};

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub policy_repo: Arc<dyn PolicyRepository>,
    pub create_policy_uc: Arc<CreatePolicyUseCase>,
    pub get_policy_uc: Arc<GetPolicyUseCase>,
    pub update_policy_uc: Arc<UpdatePolicyUseCase>,
    pub delete_policy_uc: Arc<DeletePolicyUseCase>,
    pub evaluate_policy_uc: Arc<EvaluatePolicyUseCase>,
    pub create_bundle_uc: Arc<CreateBundleUseCase>,
    pub list_bundles_uc: Arc<ListBundlesUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<PolicyAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: PolicyAuthState) -> Self {
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
        // GET -> policies/read
        let read_routes = Router::new()
            .route("/api/v1/policies", get(policy_handler::list_policies))
            .route("/api/v1/policies/:id", get(policy_handler::get_policy))
            .route("/api/v1/bundles", get(policy_handler::list_bundles))
            .route_layer(axum::middleware::from_fn(require_permission(
                "policies", "read",
            )));

        // POST/PUT/evaluate -> policies/write
        let write_routes = Router::new()
            .route("/api/v1/policies", post(policy_handler::create_policy))
            .route("/api/v1/policies/:id", put(policy_handler::update_policy))
            .route(
                "/api/v1/policies/:id/evaluate",
                post(policy_handler::evaluate_policy),
            )
            .route("/api/v1/bundles", post(policy_handler::create_bundle))
            .route_layer(axum::middleware::from_fn(require_permission(
                "policies", "write",
            )));

        // DELETE -> policies/admin
        let admin_routes = Router::new()
            .route(
                "/api/v1/policies/:id",
                delete(policy_handler::delete_policy),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "policies", "admin",
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
            .route("/api/v1/policies", get(policy_handler::list_policies))
            .route("/api/v1/policies", post(policy_handler::create_policy))
            .route("/api/v1/policies/:id", get(policy_handler::get_policy))
            .route("/api/v1/policies/:id", put(policy_handler::update_policy))
            .route(
                "/api/v1/policies/:id",
                delete(policy_handler::delete_policy),
            )
            .route(
                "/api/v1/policies/:id/evaluate",
                post(policy_handler::evaluate_policy),
            )
            .route(
                "/api/v1/bundles",
                get(policy_handler::list_bundles),
            )
            .route(
                "/api/v1/bundles",
                post(policy_handler::create_bundle),
            )
    };

    Router::new()
        .merge(public_routes)
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
