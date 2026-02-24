pub mod health;
pub mod policy_handler;

use std::sync::Arc;

use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::domain::repository::PolicyRepository;
use crate::usecase::{
    CreatePolicyUseCase, EvaluatePolicyUseCase, GetPolicyUseCase, UpdatePolicyUseCase,
};

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub policy_repo: Arc<dyn PolicyRepository>,
    pub create_policy_uc: Arc<CreatePolicyUseCase>,
    pub get_policy_uc: Arc<GetPolicyUseCase>,
    pub update_policy_uc: Arc<UpdatePolicyUseCase>,
    pub evaluate_policy_uc: Arc<EvaluatePolicyUseCase>,
}

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
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
        .with_state(state)
}
