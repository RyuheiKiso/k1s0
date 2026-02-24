pub mod health;
pub mod vault_handler;

pub use vault_handler::AppState;

use axum::routing::{get, post};
use axum::Router;

/// REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
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
