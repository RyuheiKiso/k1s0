pub mod error;
pub mod health;
pub mod schema_handler;

use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

use crate::adapter::middleware::auth::auth_middleware;
use crate::usecase::*;

#[derive(Clone)]
pub struct AppState {
    pub list_schemas_uc: Arc<ListSchemasUseCase>,
    pub register_schema_uc: Arc<RegisterSchemaUseCase>,
    pub get_schema_uc: Arc<GetSchemaUseCase>,
    pub list_versions_uc: Arc<ListVersionsUseCase>,
    pub register_version_uc: Arc<RegisterVersionUseCase>,
    pub get_schema_version_uc: Arc<GetSchemaVersionUseCase>,
    pub delete_version_uc: Arc<DeleteVersionUseCase>,
    pub check_compatibility_uc: Arc<CheckCompatibilityUseCase>,
    pub get_diff_uc: Arc<GetDiffUseCase>,
}

pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route(
            "/api/v1/schemas",
            post(schema_handler::register_schema),
        )
        .route(
            "/api/v1/schemas/{name}/versions",
            post(schema_handler::register_version),
        )
        .route(
            "/api/v1/schemas/{name}/versions/{version}",
            delete(schema_handler::delete_version),
        )
        .route_layer(middleware::from_fn(auth_middleware));

    let public = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(health::metrics))
        .route("/api/v1/schemas", get(schema_handler::list_schemas))
        .route("/api/v1/schemas/{name}", get(schema_handler::get_schema))
        .route(
            "/api/v1/schemas/{name}/versions",
            get(schema_handler::list_versions),
        )
        .route(
            "/api/v1/schemas/{name}/versions/{version}",
            get(schema_handler::get_schema_version),
        )
        .route(
            "/api/v1/schemas/{name}/compatibility",
            post(schema_handler::check_compatibility),
        )
        .route(
            "/api/v1/schemas/{name}/diff",
            get(schema_handler::get_diff),
        );

    Router::new()
        .merge(protected)
        .merge(public)
        .with_state(state)
}
