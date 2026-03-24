// REST ハンドラーのモジュール宣言とルーター定義。
// 認証ミドルウェアと RBAC を設定し、/healthz・/readyz・/metrics は認証除外とする。
pub mod health;
pub mod task_handler;

use axum::routing::{delete, get, post, put};
use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use crate::usecase;
use crate::adapter::middleware::auth::{auth_middleware, AuthState};
use crate::adapter::middleware::rbac::require_permission;

/// AppState はハンドラが共有するアプリケーション状態
#[derive(Clone)]
pub struct AppState {
    pub create_task_uc: Arc<usecase::create_task::CreateTaskUseCase>,
    pub get_task_uc: Arc<usecase::get_task::GetTaskUseCase>,
    pub list_tasks_uc: Arc<usecase::list_tasks::ListTasksUseCase>,
    pub update_task_status_uc: Arc<usecase::update_task_status::UpdateTaskStatusUseCase>,
    pub update_task_uc: Arc<usecase::update_task::UpdateTaskUseCase>,
    pub create_checklist_item_uc: Arc<usecase::create_checklist_item::CreateChecklistItemUseCase>,
    pub update_checklist_item_uc: Arc<usecase::update_checklist_item::UpdateChecklistItemUseCase>,
    pub delete_checklist_item_uc: Arc<usecase::delete_checklist_item::DeleteChecklistItemUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    /// 認証状態。None の場合は認証なし（dev/test 環境のみ許可）
    pub auth_state: Option<AuthState>,
}

/// REST ルーターを組み立てる。
/// /healthz・/readyz・/metrics は認証除外。API ルートは認証設定がある場合のみ RBAC を適用する。
pub fn router(state: AppState) -> Router {
    // 認証不要の公開エンドポイント
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(health::metrics_handler));

    // 認証が設定されている場合は RBAC 付きルーティングを構築する
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> tasks/read
        let read_routes = Router::new()
            .route("/api/v1/tasks", get(task_handler::list_tasks))
            .route("/api/v1/tasks/{id}", get(task_handler::get_task))
            .route("/api/v1/tasks/{id}/checklist", get(task_handler::get_checklist))
            .route_layer(axum::middleware::from_fn(require_permission("tasks", "read")));

        // POST/PUT/DELETE -> tasks/write
        let write_routes = Router::new()
            .route("/api/v1/tasks", post(task_handler::create_task))
            .route("/api/v1/tasks/{id}", put(task_handler::update_task))
            .route("/api/v1/tasks/{id}/status", put(task_handler::update_task_status))
            .route("/api/v1/tasks/{id}/checklist", post(task_handler::create_checklist_item))
            .route("/api/v1/tasks/{id}/checklist/{item_id}", put(task_handler::update_checklist_item).delete(task_handler::delete_checklist_item))
            .route_layer(axum::middleware::from_fn(require_permission("tasks", "write")));

        // 認証ミドルウェアを全 API ルートに適用する
        Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
    } else {
        // 認証なし（dev モード / テスト）: 従来どおりのルーティング
        Router::new()
            .route("/api/v1/tasks", get(task_handler::list_tasks).post(task_handler::create_task))
            .route("/api/v1/tasks/{id}", get(task_handler::get_task))
            .route("/api/v1/tasks/{id}/status", put(task_handler::update_task_status))
            .route("/api/v1/tasks/{id}/checklist", get(task_handler::get_checklist).post(task_handler::create_checklist_item))
            .route("/api/v1/tasks/{id}/checklist/{item_id}", put(task_handler::update_checklist_item).delete(task_handler::delete_checklist_item))
    };

    // 公開ルートと API ルートを結合し、TraceLayer と状態を適用する
    Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
