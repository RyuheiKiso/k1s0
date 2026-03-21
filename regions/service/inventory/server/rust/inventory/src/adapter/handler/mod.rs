pub mod health;
pub mod inventory_handler;

use crate::usecase;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post, put};
use axum::Router;
use k1s0_server_common::middleware::auth_middleware::{auth_middleware, AuthState};
use k1s0_server_common::middleware::rbac::{require_permission, Tier};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

/// auth_state が None のとき（テスト・開発環境）、バイパス用 Claims を注入するミドルウェア。
/// 本番環境では auth_state = Some(...) が設定されるため、このミドルウェアは呼ばれない。
async fn bypass_auth_middleware(
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    req.extensions_mut().insert(k1s0_auth::Claims {
        sub: "test-bypass".to_string(),
        iss: "test".to_string(),
        aud: Default::default(),
        exp: u64::MAX,
        iat: 0,
        jti: None,
        typ: None,
        azp: None,
        scope: None,
        preferred_username: Some("test-bypass".to_string()),
        email: None,
        realm_access: None,
        resource_access: None,
        tier_access: None,
    });
    next.run(req).await
}

#[derive(Clone)]
pub struct AppState {
    pub reserve_stock_uc: Arc<usecase::reserve_stock::ReserveStockUseCase>,
    pub release_stock_uc: Arc<usecase::release_stock::ReleaseStockUseCase>,
    pub get_inventory_uc: Arc<usecase::get_inventory::GetInventoryUseCase>,
    pub list_inventory_uc: Arc<usecase::list_inventory::ListInventoryUseCase>,
    pub update_stock_uc: Arc<usecase::update_stock::UpdateStockUseCase>,
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
            .route("/api/v1/inventory", get(inventory_handler::list_inventory))
            .route(
                "/api/v1/inventory/{inventory_id}",
                get(inventory_handler::get_inventory),
            )
            .route_layer(axum::middleware::from_fn(move |req, next| {
                let perm = require_permission(Tier::Service, "inventory", "read");
                perm(req, next)
            }));

        let write_routes = Router::new()
            .route(
                "/api/v1/inventory/reserve",
                post(inventory_handler::reserve_stock),
            )
            .route(
                "/api/v1/inventory/release",
                post(inventory_handler::release_stock),
            )
            .route(
                "/api/v1/inventory/{inventory_id}/stock",
                put(inventory_handler::update_stock),
            )
            .route_layer(axum::middleware::from_fn(move |req, next| {
                let perm = require_permission(Tier::Service, "inventory", "write");
                perm(req, next)
            }));

        read_routes
            .merge(write_routes)
            .layer(from_fn_with_state(auth_state.clone(), auth_middleware))
    } else {
        Router::new()
            .route("/api/v1/inventory", get(inventory_handler::list_inventory))
            .route(
                "/api/v1/inventory/{inventory_id}",
                get(inventory_handler::get_inventory),
            )
            .route(
                "/api/v1/inventory/reserve",
                post(inventory_handler::reserve_stock),
            )
            .route(
                "/api/v1/inventory/release",
                post(inventory_handler::release_stock),
            )
            .route(
                "/api/v1/inventory/{inventory_id}/stock",
                put(inventory_handler::update_stock),
            )
            .layer(axum::middleware::from_fn(bypass_auth_middleware))
    };

    public_routes
        .merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
