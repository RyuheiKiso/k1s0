// REST ハンドラルーター定義。
// axum の Router を組み立て、認証ミドルウェアと RBAC を設定する。
pub mod error;
pub mod health;
pub mod project_type_handler;
pub mod status_definition_handler;
pub mod version_handler;
pub mod tenant_extension_handler;

use axum::{Router, routing::{get, post, put, delete}};
use std::sync::Arc;

use crate::infrastructure::config::auth_config::AuthState;

/// AppState はハンドラが共有するアプリケーション状態
#[derive(Clone)]
pub struct AppState {
    pub manage_project_types_uc: Arc<crate::usecase::manage_project_types::ManageProjectTypesUseCase>,
    pub manage_status_definitions_uc: Arc<crate::usecase::manage_status_definitions::ManageStatusDefinitionsUseCase>,
    pub get_versions_uc: Arc<crate::usecase::get_status_definition_versions::GetStatusDefinitionVersionsUseCase>,
    pub manage_tenant_extensions_uc: Arc<crate::usecase::manage_tenant_extensions::ManageTenantExtensionsUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<AuthState>,
}

/// REST ルーターを組み立てる
pub fn router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(health::metrics_handler));

    let api_routes = Router::new()
        // プロジェクトタイプ
        .route("/api/v1/project-types", get(project_type_handler::list_project_types).post(project_type_handler::create_project_type))
        .route("/api/v1/project-types/{project_type_id}", get(project_type_handler::get_project_type).put(project_type_handler::update_project_type).delete(project_type_handler::delete_project_type))
        // ステータス定義
        .route("/api/v1/project-types/{project_type_id}/statuses", get(status_definition_handler::list_status_definitions).post(status_definition_handler::create_status_definition))
        .route("/api/v1/project-types/{project_type_id}/statuses/{status_id}", get(status_definition_handler::get_status_definition).put(status_definition_handler::update_status_definition).delete(status_definition_handler::delete_status_definition))
        // バージョン
        .route("/api/v1/statuses/{status_id}/versions", get(version_handler::list_versions))
        // テナント拡張
        .route("/api/v1/tenants/{tenant_id}/statuses/{status_id}", get(tenant_extension_handler::get_extension).put(tenant_extension_handler::upsert_extension).delete(tenant_extension_handler::delete_extension))
        .route("/api/v1/tenants/{tenant_id}/project-types/{project_type_id}/statuses", get(tenant_extension_handler::list_tenant_statuses));

    public_routes
        .merge(api_routes)
        .with_state(state)
}
