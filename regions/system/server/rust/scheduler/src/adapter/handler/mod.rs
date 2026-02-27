pub mod health;
pub mod job_handler;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::Router;

use crate::adapter::middleware::auth::{auth_middleware, SchedulerAuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::usecase::{
    CreateJobUseCase, DeleteJobUseCase, GetJobUseCase, ListExecutionsUseCase, ListJobsUseCase,
    PauseJobUseCase, ResumeJobUseCase, TriggerJobUseCase, UpdateJobUseCase,
};

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub list_jobs_uc: Arc<ListJobsUseCase>,
    pub create_job_uc: Arc<CreateJobUseCase>,
    pub get_job_uc: Arc<GetJobUseCase>,
    pub delete_job_uc: Arc<DeleteJobUseCase>,
    pub pause_job_uc: Arc<PauseJobUseCase>,
    pub resume_job_uc: Arc<ResumeJobUseCase>,
    pub update_job_uc: Arc<UpdateJobUseCase>,
    pub trigger_job_uc: Arc<TriggerJobUseCase>,
    pub list_executions_uc: Arc<ListExecutionsUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<SchedulerAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: SchedulerAuthState) -> Self {
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
        // GET -> jobs/read
        let read_routes = Router::new()
            .route(
                "/api/v1/jobs",
                get(job_handler::list_jobs),
            )
            .route(
                "/api/v1/jobs/:id",
                get(job_handler::get_job),
            )
            .route(
                "/api/v1/jobs/:id/executions",
                get(job_handler::list_executions),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "jobs", "read",
            )));

        // POST/PUT/trigger/pause/resume -> jobs/write
        let write_routes = Router::new()
            .route(
                "/api/v1/jobs",
                post(job_handler::create_job),
            )
            .route(
                "/api/v1/jobs/:id",
                put(job_handler::update_job),
            )
            .route("/api/v1/jobs/:id/trigger", post(job_handler::trigger_job))
            .route("/api/v1/jobs/:id/pause", put(job_handler::pause_job))
            .route("/api/v1/jobs/:id/resume", put(job_handler::resume_job))
            .route_layer(axum::middleware::from_fn(require_permission(
                "jobs", "write",
            )));

        // DELETE -> jobs/admin
        let admin_routes = Router::new()
            .route(
                "/api/v1/jobs/:id",
                axum::routing::delete(job_handler::delete_job),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "jobs", "admin",
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
                "/api/v1/jobs",
                get(job_handler::list_jobs).post(job_handler::create_job),
            )
            .route(
                "/api/v1/jobs/:id",
                get(job_handler::get_job)
                    .put(job_handler::update_job)
                    .delete(job_handler::delete_job),
            )
            .route("/api/v1/jobs/:id/pause", put(job_handler::pause_job))
            .route("/api/v1/jobs/:id/resume", put(job_handler::resume_job))
            .route("/api/v1/jobs/:id/trigger", post(job_handler::trigger_job))
            .route(
                "/api/v1/jobs/:id/executions",
                get(job_handler::list_executions),
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
