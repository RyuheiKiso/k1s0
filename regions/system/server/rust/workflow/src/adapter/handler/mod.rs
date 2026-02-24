pub mod health;
pub mod workflow_handler;

pub use workflow_handler::AppState;

use axum::routing::{get, post};
use axum::Router;

/// REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route(
            "/api/v1/workflows",
            post(workflow_handler::create_workflow).get(workflow_handler::list_workflows),
        )
        .route(
            "/api/v1/workflows/:id",
            get(workflow_handler::get_workflow),
        )
        .route(
            "/api/v1/workflows/:id/execute",
            post(workflow_handler::execute_workflow),
        )
        .route(
            "/api/v1/workflows/:id/status",
            get(workflow_handler::get_workflow_status),
        )
        .with_state(state)
}
