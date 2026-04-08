pub mod config_handler;
pub mod config_schema_handler;
pub mod error;

use std::sync::Arc;

use axum::routing::{delete, get, put};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::adapter::middleware::auth::{auth_middleware, AuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::domain::repository::ConfigRepository;
use crate::usecase::{
    DeleteConfigUseCase, GetConfigSchemaUseCase, GetConfigUseCase, GetServiceConfigUseCase,
    ListConfigSchemasUseCase, ListConfigsUseCase, UpdateConfigUseCase, UpsertConfigSchemaUseCase,
};

/// `AppState` はアプリケーション全体の共有状態を表す。
#[derive(Clone)]
pub struct AppState {
    pub get_config_uc: Arc<GetConfigUseCase>,
    pub list_configs_uc: Arc<ListConfigsUseCase>,
    pub update_config_uc: Arc<UpdateConfigUseCase>,
    pub delete_config_uc: Arc<DeleteConfigUseCase>,
    pub get_service_config_uc: Arc<GetServiceConfigUseCase>,
    pub get_config_schema_uc: Arc<GetConfigSchemaUseCase>,
    pub list_config_schemas_uc: Arc<ListConfigSchemasUseCase>,
    pub upsert_config_schema_uc: Arc<UpsertConfigSchemaUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub config_repo: Arc<dyn ConfigRepository>,
    pub kafka_configured: bool,
    pub auth_state: Option<AuthState>,
}

impl AppState {
    pub fn new(
        config_repo: Arc<dyn ConfigRepository>,
        schema_repo: Arc<dyn crate::domain::repository::ConfigSchemaRepository>,
    ) -> Self {
        Self {
            get_config_uc: Arc::new(GetConfigUseCase::new(config_repo.clone())),
            list_configs_uc: Arc::new(ListConfigsUseCase::new(config_repo.clone())),
            update_config_uc: Arc::new(
                UpdateConfigUseCase::new(config_repo.clone()).with_schema_repo(schema_repo.clone()),
            ),
            delete_config_uc: Arc::new(DeleteConfigUseCase::new(config_repo.clone())),
            get_service_config_uc: Arc::new(GetServiceConfigUseCase::new(config_repo.clone())),
            get_config_schema_uc: Arc::new(GetConfigSchemaUseCase::new(schema_repo.clone())),
            list_config_schemas_uc: Arc::new(ListConfigSchemasUseCase::new(schema_repo.clone())),
            upsert_config_schema_uc: Arc::new(UpsertConfigSchemaUseCase::new(schema_repo)),
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-config-server")),
            config_repo,
            kafka_configured: false,
            auth_state: None,
        }
    }

    #[must_use] 
    pub fn with_kafka(mut self) -> Self {
        self.kafka_configured = true;
        self
    }

    #[must_use] 
    pub fn with_auth(mut self, auth_state: AuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        config_handler::healthz,
        config_handler::readyz,
        config_handler::metrics,
        config_handler::get_config,
        config_handler::list_configs,
        config_handler::update_config,
        config_handler::delete_config,
        config_handler::get_service_config,
        config_schema_handler::list_config_schemas,
        config_schema_handler::get_config_schema,
        config_schema_handler::upsert_config_schema,
    ),
    components(schemas(
        crate::domain::entity::config_entry::ConfigEntry,
        crate::domain::entity::config_entry::Pagination,
        crate::domain::entity::config_entry::ConfigListResult,
        crate::domain::entity::config_entry::ServiceConfigEntry,
        crate::domain::entity::config_entry::ServiceConfigResult,
        crate::adapter::presentation::ConfigEditorSchemaDto,
        crate::adapter::presentation::ConfigCategorySchemaDto,
        crate::adapter::presentation::ConfigFieldSchemaDto,
        crate::adapter::presentation::ConfigFieldType,
        config_handler::UpdateConfigRequest,
        crate::adapter::presentation::UpsertConfigSchemaRequestDto,
        ErrorResponse,
        ErrorBody,
        ErrorDetail,
    )),
    security(("bearer_auth" = [])),
)]
struct ApiDoc;

/// REST API ルーターを構築する。
pub fn router(state: AppState) -> Router {
    // 認証不要のエンドポイント
    let public_routes = Router::new()
        .route("/healthz", get(config_handler::healthz))
        .route("/readyz", get(config_handler::readyz))
        .route("/metrics", get(config_handler::metrics));

    // 認証が設定されている場合は RBAC 付きルーティング、そうでなければオープンアクセス
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> config/read
        let read_routes = Router::new()
            .route(
                "/api/v1/config/services/{service_name}",
                get(config_handler::get_service_config),
            )
            .route(
                "/api/v1/config/{namespace}/{key}",
                get(config_handler::get_config),
            )
            .route(
                "/api/v1/config/{namespace}",
                get(config_handler::list_configs),
            )
            .route(
                "/api/v1/config-schema",
                get(config_schema_handler::list_config_schemas),
            )
            .route(
                "/api/v1/config-schema/{service_name}",
                get(config_schema_handler::get_config_schema),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "configs", "read",
            )));

        // PUT/POST -> config/write
        let write_routes = Router::new()
            .route(
                "/api/v1/config/{namespace}/{key}",
                put(config_handler::update_config),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "configs", "write",
            )));

        // DELETE + schema admin -> config/admin
        let admin_routes = Router::new()
            .route(
                "/api/v1/config/{namespace}/{key}",
                delete(config_handler::delete_config),
            )
            .route(
                "/api/v1/config-schema/{service_name}",
                put(config_schema_handler::upsert_config_schema),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "configs", "admin",
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
            .route(
                "/api/v1/config/services/{service_name}",
                get(config_handler::get_service_config),
            )
            .route(
                "/api/v1/config/{namespace}/{key}",
                get(config_handler::get_config)
                    .put(config_handler::update_config)
                    .delete(config_handler::delete_config),
            )
            .route(
                "/api/v1/config/{namespace}",
                get(config_handler::list_configs),
            )
            .route(
                "/api/v1/config-schema",
                get(config_schema_handler::list_config_schemas),
            )
            .route(
                "/api/v1/config-schema/{service_name}",
                get(config_schema_handler::get_config_schema)
                    .put(config_schema_handler::upsert_config_schema),
            )
    };

    // with_state で Router<()> に変換後、SwaggerUI を merge する
    public_routes
        .merge(api_routes)
        .with_state(state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

/// server-common の統一エラーレスポンス型を再エクスポートする。
/// 各サーバーで重複定義を避け、ErrorResponse / `ErrorBody` / `ErrorDetail` を共通化する。
pub use k1s0_server_common::{ErrorBody, ErrorDetail, ErrorResponse};
