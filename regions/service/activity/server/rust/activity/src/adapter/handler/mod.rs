// REST ハンドラーのモジュール宣言とルーター定義。
// 認証ミドルウェアと RBAC を設定し、/healthz・/readyz・/metrics は認証除外とする。
pub mod activity_handler;
pub mod health;

use axum::routing::{get, post, put};
use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use crate::usecase;
use crate::adapter::middleware::auth::{auth_middleware, AuthState};
use crate::adapter::middleware::rbac::require_permission;

/// AppState はハンドラが共有するアプリケーション状態
#[derive(Clone)]
pub struct AppState {
    pub create_activity_uc: Arc<usecase::create_activity::CreateActivityUseCase>,
    pub get_activity_uc: Arc<usecase::get_activity::GetActivityUseCase>,
    pub list_activities_uc: Arc<usecase::list_activities::ListActivitiesUseCase>,
    pub submit_activity_uc: Arc<usecase::submit_activity::SubmitActivityUseCase>,
    pub approve_activity_uc: Arc<usecase::approve_activity::ApproveActivityUseCase>,
    pub reject_activity_uc: Arc<usecase::reject_activity::RejectActivityUseCase>,
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
        // GET -> activities/read
        let read_routes = Router::new()
            .route("/api/v1/activities", get(activity_handler::list_activities))
            .route("/api/v1/activities/{id}", get(activity_handler::get_activity))
            .route_layer(axum::middleware::from_fn(require_permission("activities", "read")));

        // POST/PUT 通常操作 -> activities/write
        let write_routes = Router::new()
            .route("/api/v1/activities", post(activity_handler::create_activity))
            .route("/api/v1/activities/{id}/submit", put(activity_handler::submit_activity))
            .route_layer(axum::middleware::from_fn(require_permission("activities", "write")));

        // 承認・拒否 -> activities/admin（svc_admin のみ）
        let admin_routes = Router::new()
            .route("/api/v1/activities/{id}/approve", put(activity_handler::approve_activity))
            .route("/api/v1/activities/{id}/reject", put(activity_handler::reject_activity))
            .route_layer(axum::middleware::from_fn(require_permission("activities", "admin")));

        // 認証ミドルウェアを全 API ルートに適用する
        Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .merge(admin_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
    } else {
        // 認証なし（dev モード / テスト）: 従来どおりのルーティング
        Router::new()
            .route("/api/v1/activities", get(activity_handler::list_activities).post(activity_handler::create_activity))
            .route("/api/v1/activities/{id}", get(activity_handler::get_activity))
            .route("/api/v1/activities/{id}/submit", put(activity_handler::submit_activity))
            .route("/api/v1/activities/{id}/approve", put(activity_handler::approve_activity))
            .route("/api/v1/activities/{id}/reject", put(activity_handler::reject_activity))
    };

    // 公開ルートと API ルートを結合し、TraceLayer と状態を適用する
    Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
