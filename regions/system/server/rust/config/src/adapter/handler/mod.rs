pub mod config_handler;
pub mod error;

use std::sync::Arc;

use axum::routing::{delete, get, put};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::adapter::middleware::auth::{auth_middleware, ConfigAuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::domain::repository::ConfigRepository;
use crate::usecase::{
    DeleteConfigUseCase, GetConfigUseCase, GetServiceConfigUseCase, ListConfigsUseCase,
    UpdateConfigUseCase,
};

/// AppState はアプリケーション全体の共有状態を表す。
#[derive(Clone)]
pub struct AppState {
    pub get_config_uc: Arc<GetConfigUseCase>,
    pub list_configs_uc: Arc<ListConfigsUseCase>,
    pub update_config_uc: Arc<UpdateConfigUseCase>,
    pub delete_config_uc: Arc<DeleteConfigUseCase>,
    pub get_service_config_uc: Arc<GetServiceConfigUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub config_repo: Arc<dyn ConfigRepository>,
    pub kafka_configured: bool,
    pub auth_state: Option<ConfigAuthState>,
}

impl AppState {
    pub fn new(config_repo: Arc<dyn ConfigRepository>) -> Self {
        Self {
            get_config_uc: Arc::new(GetConfigUseCase::new(config_repo.clone())),
            list_configs_uc: Arc::new(ListConfigsUseCase::new(config_repo.clone())),
            update_config_uc: Arc::new(UpdateConfigUseCase::new(config_repo.clone())),
            delete_config_uc: Arc::new(DeleteConfigUseCase::new(config_repo.clone())),
            get_service_config_uc: Arc::new(GetServiceConfigUseCase::new(config_repo.clone())),
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-config-server")),
            config_repo,
            kafka_configured: false,
            auth_state: None,
        }
    }

    pub fn with_kafka(mut self) -> Self {
        self.kafka_configured = true;
        self
    }

    pub fn with_auth(mut self, auth_state: ConfigAuthState) -> Self {
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
    ),
    components(schemas(
        crate::domain::entity::config_entry::ConfigEntry,
        crate::domain::entity::config_entry::Pagination,
        crate::domain::entity::config_entry::ConfigListResult,
        crate::domain::entity::config_entry::ServiceConfigEntry,
        crate::domain::entity::config_entry::ServiceConfigResult,
        config_handler::UpdateConfigRequest,
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
                "/api/v1/config/services/:service_name",
                get(config_handler::get_service_config),
            )
            .route(
                "/api/v1/config/:namespace/:key",
                get(config_handler::get_config),
            )
            .route(
                "/api/v1/config/:namespace",
                get(config_handler::list_configs),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "config", "read",
            )));

        // PUT/POST -> config/write
        let write_routes = Router::new()
            .route(
                "/api/v1/config/:namespace/:key",
                put(config_handler::update_config),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "config", "write",
            )));

        // DELETE -> config/admin
        let admin_routes = Router::new()
            .route(
                "/api/v1/config/:namespace/:key",
                delete(config_handler::delete_config),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "config", "admin",
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
                "/api/v1/config/services/:service_name",
                get(config_handler::get_service_config),
            )
            .route(
                "/api/v1/config/:namespace/:key",
                get(config_handler::get_config)
                    .put(config_handler::update_config)
                    .delete(config_handler::delete_config),
            )
            .route(
                "/api/v1/config/:namespace",
                get(config_handler::list_configs),
            )
    };

    public_routes
        .merge(api_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}

/// ErrorResponse は統一エラーレスポンス。
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub request_id: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ErrorDetail {
    pub field: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
                details: vec![],
            },
        }
    }

    pub fn with_details(code: &str, message: &str, details: Vec<ErrorDetail>) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
                details,
            },
        }
    }
}
