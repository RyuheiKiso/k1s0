pub mod board_handler;
pub mod health;

use axum::routing::{get, post, put};
use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use crate::usecase;

#[derive(Clone)]
pub struct AppState {
    pub increment_column_uc: Arc<usecase::increment_column::IncrementColumnUseCase>,
    pub decrement_column_uc: Arc<usecase::decrement_column::DecrementColumnUseCase>,
    pub get_board_column_uc: Arc<usecase::get_board_column::GetBoardColumnUseCase>,
    pub list_board_columns_uc: Arc<usecase::list_board_columns::ListBoardColumnsUseCase>,
    pub update_wip_limit_uc: Arc<usecase::update_wip_limit::UpdateWipLimitUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(health::metrics_handler))
        .route("/api/v1/board-columns", get(board_handler::list_board_columns))
        .route("/api/v1/board-columns/{id}", get(board_handler::get_board_column).put(board_handler::update_wip_limit))
        .route("/api/v1/board-columns/increment", post(board_handler::increment_column))
        .route("/api/v1/board-columns/decrement", post(board_handler::decrement_column))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
