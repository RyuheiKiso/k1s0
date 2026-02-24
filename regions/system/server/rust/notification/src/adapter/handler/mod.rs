pub mod health;
pub mod notification_handler;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::domain::repository::NotificationLogRepository;
use crate::usecase::SendNotificationUseCase;

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub send_notification_uc: Arc<SendNotificationUseCase>,
    pub log_repo: Arc<dyn NotificationLogRepository>,
}

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route(
            "/api/v1/notifications",
            post(notification_handler::send_notification),
        )
        .route(
            "/api/v1/notifications",
            get(notification_handler::list_notifications),
        )
        .route(
            "/api/v1/notifications/:id",
            get(notification_handler::get_notification),
        )
        .with_state(state)
}
