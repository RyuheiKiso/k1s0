pub mod health;
pub mod vault_handler;

pub use vault_handler::AppState;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;

/// REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler))
        .route(
            "/api/v1/secrets",
            post(vault_handler::create_secret),
        )
        .route(
            "/api/v1/secrets/:key",
            get(vault_handler::get_secret)
                .put(vault_handler::update_secret)
                .delete(vault_handler::delete_secret),
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
