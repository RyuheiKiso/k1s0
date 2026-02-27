pub mod error;
pub mod health;
pub mod schema_handler;

use std::sync::Arc;

use axum::routing::{delete, get, post};
use axum::Router;

use crate::adapter::middleware::auth::{auth_middleware, ApiRegistryAuthState};
use crate::adapter::middleware::rbac::require_permission;
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
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<ApiRegistryAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: ApiRegistryAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

pub fn router(state: AppState) -> Router {
    // 認証不要のエンドポイント
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(health::metrics));

    // 認証が設定されている場合は RBAC 付きルーティング
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> schemas/read
        let read_routes = Router::new()
            .route("/api/v1/schemas", get(schema_handler::list_schemas))
            .route("/api/v1/schemas/:name", get(schema_handler::get_schema))
            .route(
                "/api/v1/schemas/:name/versions",
                get(schema_handler::list_versions),
            )
            .route(
                "/api/v1/schemas/:name/versions/:version",
                get(schema_handler::get_schema_version),
            )
            .route(
                "/api/v1/schemas/:name/diff",
                get(schema_handler::get_diff),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "schemas", "read",
            )));

        // POST/PUT -> schemas/write
        let write_routes = Router::new()
            .route("/api/v1/schemas", post(schema_handler::register_schema))
            .route(
                "/api/v1/schemas/:name/versions",
                post(schema_handler::register_version),
            )
            .route(
                "/api/v1/schemas/:name/compatibility",
                post(schema_handler::check_compatibility),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "schemas", "write",
            )));

        // DELETE -> schemas/admin
        let admin_routes = Router::new()
            .route(
                "/api/v1/schemas/:name/versions/:version",
                delete(schema_handler::delete_version),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "schemas", "admin",
            )));

        // 認証ミドルウェアを全 API ルートに適用
        Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .merge(admin_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
    } else {
        // 認証なし（dev モード / テスト）: 従来どおり
        Router::new()
            .route("/api/v1/schemas", get(schema_handler::list_schemas))
            .route(
                "/api/v1/schemas",
                post(schema_handler::register_schema),
            )
            .route("/api/v1/schemas/:name", get(schema_handler::get_schema))
            .route(
                "/api/v1/schemas/:name/versions",
                get(schema_handler::list_versions),
            )
            .route(
                "/api/v1/schemas/:name/versions",
                post(schema_handler::register_version),
            )
            .route(
                "/api/v1/schemas/:name/versions/:version",
                get(schema_handler::get_schema_version),
            )
            .route(
                "/api/v1/schemas/:name/versions/:version",
                delete(schema_handler::delete_version),
            )
            .route(
                "/api/v1/schemas/:name/compatibility",
                post(schema_handler::check_compatibility),
            )
            .route(
                "/api/v1/schemas/:name/diff",
                get(schema_handler::get_diff),
            )
    };

    Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .with_state(state)
}
