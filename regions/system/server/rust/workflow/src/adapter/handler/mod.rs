// ハンドラモジュール
// DTO定義、ワークフロー・インスタンス・タスクの各ハンドラ、ヘルスチェックを公開する

pub mod dto;
pub mod health;
pub mod instance_handler;
pub mod task_handler;
pub mod workflow_handler;

pub use dto::AppState;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::adapter::middleware::auth::auth_middleware;
use crate::adapter::middleware::rbac::require_permission;

/// REST APIルーターを構築する
pub fn router(state: AppState, metrics_enabled: bool, metrics_path: &str) -> Router {
    // 認証不要のエンドポイント
    let mut public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz));
    if metrics_enabled {
        public_routes = public_routes.route(metrics_path, get(metrics_handler));
    }

    // 期限超過タスクチェック用の内部エンドポイント
    let internal_routes = Router::new().route(
        "/internal/tasks/check-overdue",
        post(task_handler::check_overdue_tasks),
    );

    // 認証が設定されている場合は RBAC 付きルーティング
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> workflows/read 権限が必要なルート
        let read_routes = Router::new()
            .route("/api/v1/workflows", get(workflow_handler::list_workflows))
            .route("/api/v1/workflows/{id}", get(workflow_handler::get_workflow))
            .route("/api/v1/instances", get(instance_handler::list_instances))
            .route("/api/v1/instances/{id}", get(instance_handler::get_instance))
            .route(
                "/api/v1/instances/{id}/status",
                get(instance_handler::get_instance_status),
            )
            .route("/api/v1/tasks", get(task_handler::list_tasks))
            .route_layer(axum::middleware::from_fn(require_permission(
                "workflows",
                "read",
            )));

        // POST/PUT/execute/approve/reject/reassign -> workflows/write 権限が必要なルート
        let write_routes = Router::new()
            .route("/api/v1/workflows", post(workflow_handler::create_workflow))
            .route(
                "/api/v1/workflows/{id}",
                put(workflow_handler::update_workflow),
            )
            .route(
                "/api/v1/workflows/{id}/execute",
                post(instance_handler::execute_workflow),
            )
            .route(
                "/api/v1/tasks/{id}/approve",
                post(task_handler::approve_task),
            )
            .route("/api/v1/tasks/{id}/reject", post(task_handler::reject_task))
            .route(
                "/api/v1/tasks/{id}/reassign",
                post(task_handler::reassign_task),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "workflows",
                "write",
            )));

        // DELETE/cancel -> workflows/admin 権限が必要なルート
        let admin_routes = Router::new()
            .route(
                "/api/v1/workflows/{id}",
                delete(workflow_handler::delete_workflow),
            )
            .route(
                "/api/v1/instances/{id}/cancel",
                post(instance_handler::cancel_instance),
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
                "/api/v1/workflows/{id}",
                get(workflow_handler::get_workflow)
                    .put(workflow_handler::update_workflow)
                    .delete(workflow_handler::delete_workflow),
            )
            .route(
                "/api/v1/workflows/{id}/execute",
                post(instance_handler::execute_workflow),
            )
            .route("/api/v1/instances", get(instance_handler::list_instances))
            .route("/api/v1/instances/{id}", get(instance_handler::get_instance))
            .route(
                "/api/v1/instances/{id}/status",
                get(instance_handler::get_instance_status),
            )
            .route(
                "/api/v1/instances/{id}/cancel",
                post(instance_handler::cancel_instance),
            )
            .route("/api/v1/tasks", get(task_handler::list_tasks))
            .route(
                "/api/v1/tasks/{id}/approve",
                post(task_handler::approve_task),
            )
            .route("/api/v1/tasks/{id}/reject", post(task_handler::reject_task))
            .route(
                "/api/v1/tasks/{id}/reassign",
                post(task_handler::reassign_task),
            )
    };

    public_routes
        .merge(internal_routes)
        .merge(api_routes)
        .with_state(state)
}

/// メトリクスエンドポイントのハンドラ
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
