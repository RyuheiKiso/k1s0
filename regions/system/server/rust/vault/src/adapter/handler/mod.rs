pub mod health;

use axum::routing::get;
use axum::Router;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Option<sqlx::PgPool>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .with_state(state)
}
