pub mod health;
pub mod workflow_handler;

pub use workflow_handler::AppState;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;

/// REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler))
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
        .route(
            "/api/v1/workflows/:id",
            put(workflow_handler::update_workflow),
        )
        .route(
            "/api/v1/workflows/:id",
            delete(workflow_handler::delete_workflow),
        )
        .route(
            "/api/v1/instances",
            get(workflow_handler::list_instances),
        )
        .route(
            "/api/v1/instances/:id",
            get(workflow_handler::get_instance),
        )
        .route(
            "/api/v1/instances/:id/cancel",
            post(workflow_handler::cancel_instance),
        )
        .route(
            "/api/v1/tasks",
            get(workflow_handler::list_tasks),
        )
        .route(
            "/api/v1/tasks/:id/approve",
            post(workflow_handler::approve_task),
        )
        .route(
            "/api/v1/tasks/:id/reject",
            post(workflow_handler::reject_task),
        )
        .route(
            "/api/v1/tasks/:id/reassign",
            post(workflow_handler::reassign_task),
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
