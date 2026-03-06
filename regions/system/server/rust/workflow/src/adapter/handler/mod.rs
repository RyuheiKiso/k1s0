pub mod health;
pub mod workflow_handler;

pub use workflow_handler::AppState;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::adapter::middleware::auth::auth_middleware;
use crate::adapter::middleware::rbac::require_permission;

/// REST API router.
pub fn router(state: AppState, metrics_enabled: bool, metrics_path: &str) -> Router {
    // 認証不要のエンドポイント
    let mut public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz));
    if metrics_enabled {
        public_routes = public_routes.route(metrics_path, get(metrics_handler));
    }

    let internal_routes = Router::new().route(
        "/internal/tasks/check-overdue",
        post(workflow_handler::check_overdue_tasks),
    );

    // 認証が設定されている場合は RBAC 付きルーティング
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> workflows/read
        let read_routes = Router::new()
            .route("/api/v1/workflows", get(workflow_handler::list_workflows))
            .route("/api/v1/workflows/:id", get(workflow_handler::get_workflow))
            .route("/api/v1/instances", get(workflow_handler::list_instances))
            .route("/api/v1/instances/:id", get(workflow_handler::get_instance))
            .route(
                "/api/v1/instances/:id/status",
                get(workflow_handler::get_instance_status),
            )
            .route("/api/v1/tasks", get(workflow_handler::list_tasks))
            .route_layer(axum::middleware::from_fn(require_permission(
                "workflows",
                "read",
            )));

        // POST/PUT/execute/approve/reject/reassign -> workflows/write
        let write_routes = Router::new()
            .route("/api/v1/workflows", post(workflow_handler::create_workflow))
            .route(
                "/api/v1/workflows/:id",
                put(workflow_handler::update_workflow),
            )
            .route(
                "/api/v1/workflows/:id/execute",
                post(workflow_handler::execute_workflow),
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
            .route_layer(axum::middleware::from_fn(require_permission(
                "workflows",
                "write",
            )));

        // DELETE/cancel -> workflows/admin
        let admin_routes = Router::new()
            .route(
                "/api/v1/workflows/:id",
                delete(workflow_handler::delete_workflow),
            )
            .route(
                "/api/v1/instances/:id/cancel",
                post(workflow_handler::cancel_instance),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "workflows",
                "admin",
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
        // 認証なし（dev モード / テスト）
        Router::new()
            .route(
                "/api/v1/workflows",
                post(workflow_handler::create_workflow).get(workflow_handler::list_workflows),
            )
            .route(
                "/api/v1/workflows/:id",
                get(workflow_handler::get_workflow)
                    .put(workflow_handler::update_workflow)
                    .delete(workflow_handler::delete_workflow),
            )
            .route(
                "/api/v1/workflows/:id/execute",
                post(workflow_handler::execute_workflow),
            )
            .route("/api/v1/instances", get(workflow_handler::list_instances))
            .route("/api/v1/instances/:id", get(workflow_handler::get_instance))
            .route(
                "/api/v1/instances/:id/status",
                get(workflow_handler::get_instance_status),
            )
            .route(
                "/api/v1/instances/:id/cancel",
                post(workflow_handler::cancel_instance),
            )
            .route("/api/v1/tasks", get(workflow_handler::list_tasks))
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
    };

    public_routes
        .merge(internal_routes)
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
