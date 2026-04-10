// REST ハンドラルーター定義。
// axum の Router を組み立て、認証ミドルウェアと RBAC を設定する。
// /healthz・/readyz・/metrics は認証除外とする。
pub mod error;
pub mod health;
pub mod project_type_handler;
pub mod status_definition_handler;
pub mod version_handler;
pub mod tenant_extension_handler;

use axum::{Router, routing::{get, post, put, delete}};
use std::sync::Arc;

use crate::adapter::middleware::auth::{auth_middleware, AuthState};
use crate::adapter::middleware::rbac::require_permission;

/// AppState はハンドラが共有するアプリケーション状態
#[derive(Clone)]
pub struct AppState {
    pub manage_project_types_uc: Arc<crate::usecase::manage_project_types::ManageProjectTypesUseCase>,
    pub manage_status_definitions_uc: Arc<crate::usecase::manage_status_definitions::ManageStatusDefinitionsUseCase>,
    pub get_versions_uc: Arc<crate::usecase::get_status_definition_versions::GetStatusDefinitionVersionsUseCase>,
    pub manage_tenant_extensions_uc: Arc<crate::usecase::manage_tenant_extensions::ManageTenantExtensionsUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    /// 認証状態。None の場合は認証なし（dev/test 環境のみ許可）
    pub auth_state: Option<AuthState>,
    /// MEDIUM-001 監査対応: readyz で DB 疎通確認を行うためのコネクションプール。
    /// task-rust と同一パターンで SELECT 1 による確認を実装する。
    pub db_pool: sqlx::PgPool,
}

/// REST ルーターを組み立てる。
/// 認証が設定されている場合は RBAC 付きルーティングを構築し、なければ認証なしで動作する。
pub fn router(state: AppState) -> Router {
    // 認証不要の公開エンドポイント
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(health::metrics_handler));

    // 認証が設定されている場合は RBAC 付きルーティングを構築する
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> project-types/read
        let read_routes = Router::new()
            .route("/api/v1/project-types", get(project_type_handler::list_project_types))
            .route("/api/v1/project-types/{project_type_id}", get(project_type_handler::get_project_type))
            // ステータス定義 GET
            .route("/api/v1/project-types/{project_type_id}/statuses", get(status_definition_handler::list_status_definitions))
            .route("/api/v1/project-types/{project_type_id}/statuses/{status_id}", get(status_definition_handler::get_status_definition))
            // バージョン GET
            .route("/api/v1/statuses/{status_id}/versions", get(version_handler::list_versions))
            // テナント拡張 GET
            .route("/api/v1/tenants/{tenant_id}/statuses/{status_id}", get(tenant_extension_handler::get_extension))
            .route("/api/v1/tenants/{tenant_id}/project-types/{project_type_id}/statuses", get(tenant_extension_handler::list_tenant_statuses))
            .route_layer(axum::middleware::from_fn(require_permission("project-master", "read")));

        // POST/PUT -> project-types/write
        let write_routes = Router::new()
            .route("/api/v1/project-types", post(project_type_handler::create_project_type))
            .route("/api/v1/project-types/{project_type_id}", put(project_type_handler::update_project_type))
            // ステータス定義 POST/PUT
            .route("/api/v1/project-types/{project_type_id}/statuses", post(status_definition_handler::create_status_definition))
            .route("/api/v1/project-types/{project_type_id}/statuses/{status_id}", put(status_definition_handler::update_status_definition))
            // テナント拡張 PUT
            .route("/api/v1/tenants/{tenant_id}/statuses/{status_id}", put(tenant_extension_handler::upsert_extension))
            .route_layer(axum::middleware::from_fn(require_permission("project-master", "write")));

        // DELETE -> project-types/admin（biz_taskmanagement_admin のみ）
        let admin_routes = Router::new()
            .route("/api/v1/project-types/{project_type_id}", delete(project_type_handler::delete_project_type))
            .route("/api/v1/project-types/{project_type_id}/statuses/{status_id}", delete(status_definition_handler::delete_status_definition))
            .route("/api/v1/tenants/{tenant_id}/statuses/{status_id}", delete(tenant_extension_handler::delete_extension))
            .route_layer(axum::middleware::from_fn(require_permission("project-master", "admin")));

        // 認証ミドルウェアを全 API ルートに適用する
        Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .merge(admin_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
    } else {
        // 認証なし（dev モード / テスト）: 従来どおりのルーティング
        Router::new()
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
            .route("/api/v1/tenants/{tenant_id}/project-types/{project_type_id}/statuses", get(tenant_extension_handler::list_tenant_statuses))
    };

    // 公開ルートと API ルートを結合して状態を適用する
    public_routes
        .merge(api_routes)
        .with_state(state)
}
