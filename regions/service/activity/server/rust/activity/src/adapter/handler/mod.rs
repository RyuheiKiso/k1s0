pub mod activity_handler;
pub mod health;

use axum::routing::{get, post, put};
use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use crate::usecase;

#[derive(Clone)]
pub struct AppState {
    pub create_activity_uc: Arc<usecase::create_activity::CreateActivityUseCase>,
    pub get_activity_uc: Arc<usecase::get_activity::GetActivityUseCase>,
    pub list_activities_uc: Arc<usecase::list_activities::ListActivitiesUseCase>,
    pub submit_activity_uc: Arc<usecase::submit_activity::SubmitActivityUseCase>,
    pub approve_activity_uc: Arc<usecase::approve_activity::ApproveActivityUseCase>,
    pub reject_activity_uc: Arc<usecase::reject_activity::RejectActivityUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(health::metrics_handler))
        .route("/api/v1/activities", get(activity_handler::list_activities).post(activity_handler::create_activity))
        .route("/api/v1/activities/{id}", get(activity_handler::get_activity))
        .route("/api/v1/activities/{id}/submit", put(activity_handler::submit_activity))
        .route("/api/v1/activities/{id}/approve", put(activity_handler::approve_activity))
        .route("/api/v1/activities/{id}/reject", put(activity_handler::reject_activity))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
