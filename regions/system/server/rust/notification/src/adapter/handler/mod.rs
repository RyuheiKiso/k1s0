pub mod health;
pub mod notification_handler;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::adapter::middleware::auth::{auth_middleware, NotificationAuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::domain::repository::NotificationLogRepository;
use crate::usecase::{
    CreateChannelUseCase, CreateTemplateUseCase, DeleteChannelUseCase, DeleteTemplateUseCase,
    GetChannelUseCase, GetTemplateUseCase, ListChannelsUseCase, ListTemplatesUseCase,
    RetryNotificationUseCase, SendNotificationUseCase, UpdateChannelUseCase,
    UpdateTemplateUseCase,
};

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub send_notification_uc: Arc<SendNotificationUseCase>,
    pub retry_notification_uc: Arc<RetryNotificationUseCase>,
    pub log_repo: Arc<dyn NotificationLogRepository>,
    pub create_channel_uc: Arc<CreateChannelUseCase>,
    pub list_channels_uc: Arc<ListChannelsUseCase>,
    pub get_channel_uc: Arc<GetChannelUseCase>,
    pub update_channel_uc: Arc<UpdateChannelUseCase>,
    pub delete_channel_uc: Arc<DeleteChannelUseCase>,
    pub create_template_uc: Arc<CreateTemplateUseCase>,
    pub list_templates_uc: Arc<ListTemplatesUseCase>,
    pub get_template_uc: Arc<GetTemplateUseCase>,
    pub update_template_uc: Arc<UpdateTemplateUseCase>,
    pub delete_template_uc: Arc<DeleteTemplateUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<NotificationAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: NotificationAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler));

    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> notifications/read
        let read_routes = Router::new()
            .route(
                "/api/v1/notifications",
                get(notification_handler::list_notifications),
            )
            .route(
                "/api/v1/notifications/:id",
                get(notification_handler::get_notification),
            )
            .route(
                "/api/v1/channels",
                get(notification_handler::list_channels),
            )
            .route(
                "/api/v1/channels/:id",
                get(notification_handler::get_channel),
            )
            .route(
                "/api/v1/templates",
                get(notification_handler::list_templates),
            )
            .route(
                "/api/v1/templates/:id",
                get(notification_handler::get_template),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "notifications", "read",
            )));

        // POST/PUT/retry/DELETE -> notifications/write
        let write_routes = Router::new()
            .route(
                "/api/v1/notifications",
                post(notification_handler::send_notification),
            )
            .route(
                "/api/v1/notifications/:id/retry",
                post(notification_handler::retry_notification),
            )
            .route(
                "/api/v1/channels",
                post(notification_handler::create_channel),
            )
            .route(
                "/api/v1/channels/:id",
                put(notification_handler::update_channel),
            )
            .route(
                "/api/v1/channels/:id",
                delete(notification_handler::delete_channel),
            )
            .route(
                "/api/v1/templates",
                post(notification_handler::create_template),
            )
            .route(
                "/api/v1/templates/:id",
                put(notification_handler::update_template),
            )
            .route(
                "/api/v1/templates/:id",
                delete(notification_handler::delete_template),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "notifications", "write",
            )));

        Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
    } else {
        Router::new()
            // Notifications
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
            .route(
                "/api/v1/notifications/:id/retry",
                post(notification_handler::retry_notification),
            )
            // Channels
            .route(
                "/api/v1/channels",
                post(notification_handler::create_channel),
            )
            .route(
                "/api/v1/channels",
                get(notification_handler::list_channels),
            )
            .route(
                "/api/v1/channels/:id",
                get(notification_handler::get_channel),
            )
            .route(
                "/api/v1/channels/:id",
                put(notification_handler::update_channel),
            )
            .route(
                "/api/v1/channels/:id",
                delete(notification_handler::delete_channel),
            )
            // Templates
            .route(
                "/api/v1/templates",
                post(notification_handler::create_template),
            )
            .route(
                "/api/v1/templates",
                get(notification_handler::list_templates),
            )
            .route(
                "/api/v1/templates/:id",
                get(notification_handler::get_template),
            )
            .route(
                "/api/v1/templates/:id",
                put(notification_handler::update_template),
            )
            .route(
                "/api/v1/templates/:id",
                delete(notification_handler::delete_template),
            )
    };

    Router::new()
        .merge(public_routes)
        .merge(api_routes)
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
