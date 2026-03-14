pub mod health;
pub mod payment_handler;

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
    pub initiate_payment_uc: Arc<usecase::initiate_payment::InitiatePaymentUseCase>,
    pub get_payment_uc: Arc<usecase::get_payment::GetPaymentUseCase>,
    pub list_payments_uc: Arc<usecase::list_payments::ListPaymentsUseCase>,
    pub complete_payment_uc: Arc<usecase::complete_payment::CompletePaymentUseCase>,
    pub fail_payment_uc: Arc<usecase::fail_payment::FailPaymentUseCase>,
    pub refund_payment_uc: Arc<usecase::refund_payment::RefundPaymentUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<AuthState>,
    pub db_pool: Option<sqlx::PgPool>,
}

pub fn router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(health::metrics_handler));

    let api_routes = if let Some(ref auth_state) = state.auth_state {
        let read_routes = Router::new()
            .route("/api/v1/payments", get(payment_handler::list_payments))
            .route(
                "/api/v1/payments/:payment_id",
                get(payment_handler::get_payment),
            )
            .route_layer(axum::middleware::from_fn(move |req, next| {
                let perm = require_permission(Tier::Service, "payment", "read");
                perm(req, next)
            }));

        let write_routes = Router::new()
            .route("/api/v1/payments", post(payment_handler::initiate_payment))
            .route(
                "/api/v1/payments/:payment_id/complete",
                put(payment_handler::complete_payment),
            )
            .route(
                "/api/v1/payments/:payment_id/fail",
                put(payment_handler::fail_payment),
            )
            .route(
                "/api/v1/payments/:payment_id/refund",
                put(payment_handler::refund_payment),
            )
            .route_layer(axum::middleware::from_fn(move |req, next| {
                let perm = require_permission(Tier::Service, "payment", "write");
                perm(req, next)
            }));

        read_routes
            .merge(write_routes)
            .layer(from_fn_with_state(auth_state.clone(), auth_middleware))
    } else {
        Router::new()
            .route(
                "/api/v1/payments",
                get(payment_handler::list_payments).post(payment_handler::initiate_payment),
            )
            .route(
                "/api/v1/payments/:payment_id",
                get(payment_handler::get_payment),
            )
            .route(
                "/api/v1/payments/:payment_id/complete",
                put(payment_handler::complete_payment),
            )
            .route(
                "/api/v1/payments/:payment_id/fail",
                put(payment_handler::fail_payment),
            )
            .route(
                "/api/v1/payments/:payment_id/refund",
                put(payment_handler::refund_payment),
            )
    };

    public_routes
        .merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
