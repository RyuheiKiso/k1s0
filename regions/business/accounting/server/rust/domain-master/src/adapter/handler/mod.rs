pub mod category_handler;
pub mod error;
pub mod item_handler;
pub mod tenant_handler;

use crate::usecase;
use axum::middleware::from_fn_with_state;
use axum::routing::{delete, get, post, put};
use axum::Router;
use k1s0_auth::Claims;
use k1s0_server_common::middleware::auth_middleware::{auth_middleware, AuthState};
use k1s0_server_common::middleware::rbac::{require_permission, Tier};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub manage_categories_uc: Arc<usecase::manage_categories::ManageCategoriesUseCase>,
    pub manage_items_uc: Arc<usecase::manage_items::ManageItemsUseCase>,
    pub get_item_versions_uc: Arc<usecase::get_item_versions::GetItemVersionsUseCase>,
    pub manage_tenant_extensions_uc:
        Arc<usecase::manage_tenant_extensions::ManageTenantExtensionsUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<AuthState>,
}

pub fn actor_from_claims(claims: Option<&Claims>) -> String {
    claims
        .and_then(|claims| {
            claims
                .preferred_username
                .as_ref()
                .filter(|value| !value.is_empty())
                .cloned()
                .or_else(|| {
                    claims
                        .email
                        .as_ref()
                        .filter(|value| !value.is_empty())
                        .cloned()
                })
                .or_else(|| (!claims.sub.is_empty()).then(|| claims.sub.clone()))
        })
        .unwrap_or_else(|| "system".to_string())
}

pub fn router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/healthz", get(category_handler::healthz))
        .route("/readyz", get(category_handler::readyz))
        .route("/metrics", get(category_handler::metrics_handler));

    let api_routes = if let Some(ref auth_state) = state.auth_state {
        let read_routes = Router::new()
            .route("/api/v1/categories", get(category_handler::list_categories))
            .route(
                "/api/v1/categories/:code",
                get(category_handler::get_category),
            )
            .route(
                "/api/v1/categories/:code/items",
                get(item_handler::list_items),
            )
            .route(
                "/api/v1/categories/:code/items/:item_code",
                get(item_handler::get_item),
            )
            .route(
                "/api/v1/categories/:code/items/:item_code/versions",
                get(item_handler::list_versions),
            )
            .route(
                "/api/v1/tenants/:tenant_id/items/:item_id",
                get(tenant_handler::get_tenant_extension),
            )
            .route(
                "/api/v1/tenants/:tenant_id/categories/:code/items",
                get(tenant_handler::list_tenant_items),
            )
            .route_layer(axum::middleware::from_fn(move |req, next| {
                let perm = require_permission(Tier::Business, "domain-master", "read");
                perm(req, next)
            }));

        let write_routes = Router::new()
            .route(
                "/api/v1/categories",
                post(category_handler::create_category),
            )
            .route(
                "/api/v1/categories/:code",
                put(category_handler::update_category),
            )
            .route(
                "/api/v1/categories/:code/items",
                post(item_handler::create_item),
            )
            .route(
                "/api/v1/categories/:code/items/:item_code",
                put(item_handler::update_item),
            )
            .route(
                "/api/v1/tenants/:tenant_id/items/:item_id",
                put(tenant_handler::upsert_tenant_extension),
            )
            .route_layer(axum::middleware::from_fn(move |req, next| {
                let perm = require_permission(Tier::Business, "domain-master", "write");
                perm(req, next)
            }));

        let admin_routes = Router::new()
            .route(
                "/api/v1/categories/:code",
                delete(category_handler::delete_category),
            )
            .route(
                "/api/v1/categories/:code/items/:item_code",
                delete(item_handler::delete_item),
            )
            .route(
                "/api/v1/tenants/:tenant_id/items/:item_id",
                delete(tenant_handler::delete_tenant_extension),
            )
            .route_layer(axum::middleware::from_fn(move |req, next| {
                let perm = require_permission(Tier::Business, "domain-master", "admin");
                perm(req, next)
            }));

        read_routes
            .merge(write_routes)
            .merge(admin_routes)
            .layer(from_fn_with_state(auth_state.clone(), auth_middleware))
    } else {
        Router::new()
            .route(
                "/api/v1/categories",
                get(category_handler::list_categories).post(category_handler::create_category),
            )
            .route(
                "/api/v1/categories/:code",
                get(category_handler::get_category)
                    .put(category_handler::update_category)
                    .delete(category_handler::delete_category),
            )
            .route(
                "/api/v1/categories/:code/items",
                get(item_handler::list_items).post(item_handler::create_item),
            )
            .route(
                "/api/v1/categories/:code/items/:item_code",
                get(item_handler::get_item)
                    .put(item_handler::update_item)
                    .delete(item_handler::delete_item),
            )
            .route(
                "/api/v1/categories/:code/items/:item_code/versions",
                get(item_handler::list_versions),
            )
            .route(
                "/api/v1/tenants/:tenant_id/items/:item_id",
                get(tenant_handler::get_tenant_extension)
                    .put(tenant_handler::upsert_tenant_extension)
                    .delete(tenant_handler::delete_tenant_extension),
            )
            .route(
                "/api/v1/tenants/:tenant_id/categories/:code/items",
                get(tenant_handler::list_tenant_items),
            )
    };

    public_routes
        .merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
