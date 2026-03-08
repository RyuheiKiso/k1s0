pub mod health;
pub mod order_handler;

use crate::usecase;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post, put};
use axum::Router;
use k1s0_server_common::middleware::auth_middleware::{auth_middleware, AuthState};
use k1s0_server_common::middleware::rbac::{require_permission, Tier};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub create_order_uc: Arc<usecase::create_order::CreateOrderUseCase>,
    pub get_order_uc: Arc<usecase::get_order::GetOrderUseCase>,
    pub update_order_status_uc: Arc<usecase::update_order_status::UpdateOrderStatusUseCase>,
    pub list_orders_uc: Arc<usecase::list_orders::ListOrdersUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<AuthState>,
}

pub fn router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(health::metrics_handler));

    let api_routes = if let Some(ref auth_state) = state.auth_state {
        let read_routes = Router::new()
            .route("/api/v1/orders", get(order_handler::list_orders))
            .route("/api/v1/orders/:order_id", get(order_handler::get_order))
            .route_layer(axum::middleware::from_fn(move |req, next| {
                let perm = require_permission(Tier::Service, "order", "read");
                perm(req, next)
            }));

        let write_routes = Router::new()
            .route("/api/v1/orders", post(order_handler::create_order))
            .route(
                "/api/v1/orders/:order_id/status",
                put(order_handler::update_order_status),
            )
            .route_layer(axum::middleware::from_fn(move |req, next| {
                let perm = require_permission(Tier::Service, "order", "write");
                perm(req, next)
            }));

        read_routes
            .merge(write_routes)
            .layer(from_fn_with_state(auth_state.clone(), auth_middleware))
    } else {
        Router::new()
            .route(
                "/api/v1/orders",
                get(order_handler::list_orders).post(order_handler::create_order),
            )
            .route(
                "/api/v1/orders/:order_id",
                get(order_handler::get_order),
            )
            .route(
                "/api/v1/orders/:order_id/status",
                put(order_handler::update_order_status),
            )
    };

    public_routes
        .merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
