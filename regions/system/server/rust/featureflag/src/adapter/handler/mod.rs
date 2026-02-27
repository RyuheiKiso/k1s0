pub mod flag_handler;
pub mod health;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::adapter::middleware::auth::{auth_middleware, FeatureflagAuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::domain::repository::FeatureFlagRepository;
use crate::usecase::{CreateFlagUseCase, DeleteFlagUseCase, EvaluateFlagUseCase, GetFlagUseCase, UpdateFlagUseCase};

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub flag_repo: Arc<dyn FeatureFlagRepository>,
    pub evaluate_flag_uc: Arc<EvaluateFlagUseCase>,
    pub get_flag_uc: Arc<GetFlagUseCase>,
    pub create_flag_uc: Arc<CreateFlagUseCase>,
    pub update_flag_uc: Arc<UpdateFlagUseCase>,
    pub delete_flag_uc: Arc<DeleteFlagUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<FeatureflagAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: FeatureflagAuthState) -> Self {
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
        // GET -> flags/read
        let read_routes = Router::new()
            .route("/api/v1/flags", get(flag_handler::list_flags))
            .route("/api/v1/flags/:key", get(flag_handler::get_flag))
            .route_layer(axum::middleware::from_fn(require_permission(
                "flags", "read",
            )));

        // POST/PUT/evaluate -> flags/write
        let write_routes = Router::new()
            .route("/api/v1/flags", post(flag_handler::create_flag))
            .route("/api/v1/flags/:key", put(flag_handler::update_flag))
            .route(
                "/api/v1/flags/:key/evaluate",
                post(flag_handler::evaluate_flag),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "flags", "write",
            )));

        // DELETE -> flags/admin
        let admin_routes = Router::new()
            .route("/api/v1/flags/:key", delete(flag_handler::delete_flag))
            .route_layer(axum::middleware::from_fn(require_permission(
                "flags", "admin",
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
            .route("/api/v1/flags", get(flag_handler::list_flags))
            .route("/api/v1/flags", post(flag_handler::create_flag))
            .route("/api/v1/flags/:key", get(flag_handler::get_flag))
            .route("/api/v1/flags/:key", put(flag_handler::update_flag))
            .route("/api/v1/flags/:key", delete(flag_handler::delete_flag))
            .route(
                "/api/v1/flags/:key/evaluate",
                post(flag_handler::evaluate_flag),
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
