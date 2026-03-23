// REST ハンドラーのモジュール宣言とルーター定義。
// 認証ミドルウェアと RBAC を設定し、/healthz・/readyz・/metrics は認証除外とする。
pub mod board_handler;
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
    pub increment_column_uc: Arc<usecase::increment_column::IncrementColumnUseCase>,
    pub decrement_column_uc: Arc<usecase::decrement_column::DecrementColumnUseCase>,
    pub get_board_column_uc: Arc<usecase::get_board_column::GetBoardColumnUseCase>,
    pub list_board_columns_uc: Arc<usecase::list_board_columns::ListBoardColumnsUseCase>,
    pub update_wip_limit_uc: Arc<usecase::update_wip_limit::UpdateWipLimitUseCase>,
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
        // GET -> board-columns/read
        let read_routes = Router::new()
            .route("/api/v1/board-columns", get(board_handler::list_board_columns))
            .route("/api/v1/board-columns/{id}", get(board_handler::get_board_column))
            .route_layer(axum::middleware::from_fn(require_permission("board-columns", "read")));

        // POST/PUT -> board-columns/write
        let write_routes = Router::new()
            .route("/api/v1/board-columns/{id}", put(board_handler::update_wip_limit))
            .route("/api/v1/board-columns/increment", post(board_handler::increment_column))
            .route("/api/v1/board-columns/decrement", post(board_handler::decrement_column))
            .route_layer(axum::middleware::from_fn(require_permission("board-columns", "write")));

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
            .route("/api/v1/board-columns", get(board_handler::list_board_columns))
            .route("/api/v1/board-columns/{id}", get(board_handler::get_board_column).put(board_handler::update_wip_limit))
            .route("/api/v1/board-columns/increment", post(board_handler::increment_column))
            .route("/api/v1/board-columns/decrement", post(board_handler::decrement_column))
    };

    // 公開ルートと API ルートを結合し、TraceLayer と状態を適用する
    Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
