pub mod config_handler;
pub mod error;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;

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
}

impl AppState {
    pub fn new(config_repo: Arc<dyn ConfigRepository>) -> Self {
        Self {
            get_config_uc: Arc::new(GetConfigUseCase::new(config_repo.clone())),
            list_configs_uc: Arc::new(ListConfigsUseCase::new(config_repo.clone())),
            update_config_uc: Arc::new(UpdateConfigUseCase::new(config_repo.clone())),
            delete_config_uc: Arc::new(DeleteConfigUseCase::new(config_repo.clone())),
            get_service_config_uc: Arc::new(GetServiceConfigUseCase::new(config_repo)),
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-config-server")),
        }
    }
}

/// REST API ルーターを構築する。
pub fn router(state: AppState) -> Router {
    Router::new()
        // Health / Readiness / Metrics
        .route("/healthz", get(config_handler::healthz))
        .route("/readyz", get(config_handler::readyz))
        .route("/metrics", get(config_handler::metrics))
        // Service config endpoint (static "services" segment must be before dynamic :namespace)
        .route(
            "/api/v1/config/services/:service_name",
            get(config_handler::get_service_config),
        )
        // Config endpoints
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
        .with_state(state)
}

/// ErrorResponse は統一エラーレスポンス。
#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, serde::Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub request_id: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, serde::Serialize)]
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
