pub mod health;
pub mod job_handler;

use std::sync::Arc;

use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::domain::repository::SchedulerJobRepository;
use crate::usecase::{CreateJobUseCase, GetJobUseCase, PauseJobUseCase, ResumeJobUseCase};

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub job_repo: Arc<dyn SchedulerJobRepository>,
    pub create_job_uc: Arc<CreateJobUseCase>,
    pub get_job_uc: Arc<GetJobUseCase>,
    pub pause_job_uc: Arc<PauseJobUseCase>,
    pub resume_job_uc: Arc<ResumeJobUseCase>,
}

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/api/v1/jobs", get(job_handler::list_jobs))
        .route("/api/v1/jobs", post(job_handler::create_job))
        .route("/api/v1/jobs/:id", get(job_handler::get_job))
        .route("/api/v1/jobs/:id", delete(job_handler::delete_job))
        .route("/api/v1/jobs/:id/pause", put(job_handler::pause_job))
        .route("/api/v1/jobs/:id/resume", put(job_handler::resume_job))
        .with_state(state)
}
