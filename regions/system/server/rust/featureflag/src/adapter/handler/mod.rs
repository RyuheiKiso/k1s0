pub mod flag_handler;
pub mod health;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
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
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
}

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler))
        .route("/api/v1/flags", get(flag_handler::list_flags))
        .route("/api/v1/flags", post(flag_handler::create_flag))
        .route("/api/v1/flags/:key", get(flag_handler::get_flag))
        .route("/api/v1/flags/:key", put(flag_handler::update_flag))
        .route("/api/v1/flags/:id", delete(flag_handler::delete_flag))
        .route(
            "/api/v1/flags/:key/evaluate",
            post(flag_handler::evaluate_flag),
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
