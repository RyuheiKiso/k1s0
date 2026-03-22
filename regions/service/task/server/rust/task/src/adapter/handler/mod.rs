// REST ハンドラーのモジュール宣言とルーター定義。
pub mod health;
pub mod task_handler;

use axum::routing::{get, post, put};
use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use crate::usecase;

#[derive(Clone)]
pub struct AppState {
    pub create_task_uc: Arc<usecase::create_task::CreateTaskUseCase>,
    pub get_task_uc: Arc<usecase::get_task::GetTaskUseCase>,
    pub list_tasks_uc: Arc<usecase::list_tasks::ListTasksUseCase>,
    pub update_task_status_uc: Arc<usecase::update_task_status::UpdateTaskStatusUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(health::metrics_handler))
        .route("/api/v1/tasks", get(task_handler::list_tasks).post(task_handler::create_task))
        .route(
            "/api/v1/tasks/{id}",
            get(task_handler::get_task),
        )
        .route(
            "/api/v1/tasks/{id}/status",
            put(task_handler::update_task_status),
        )
        .route("/api/v1/tasks/{id}/checklist", get(task_handler::get_checklist))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
