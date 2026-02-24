pub mod flag_handler;
pub mod health;

use std::sync::Arc;

use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::domain::repository::FeatureFlagRepository;
use crate::usecase::{CreateFlagUseCase, EvaluateFlagUseCase, GetFlagUseCase, UpdateFlagUseCase};

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub flag_repo: Arc<dyn FeatureFlagRepository>,
    pub evaluate_flag_uc: Arc<EvaluateFlagUseCase>,
    pub get_flag_uc: Arc<GetFlagUseCase>,
    pub create_flag_uc: Arc<CreateFlagUseCase>,
    pub update_flag_uc: Arc<UpdateFlagUseCase>,
}

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/api/v1/flags", get(flag_handler::list_flags))
        .route("/api/v1/flags", post(flag_handler::create_flag))
        .route("/api/v1/flags/:key", get(flag_handler::get_flag))
        .route("/api/v1/flags/:key", put(flag_handler::update_flag))
        .route("/api/v1/flags/:id", delete(flag_handler::delete_flag))
        .with_state(state)
}
