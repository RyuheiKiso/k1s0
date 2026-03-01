pub mod table_handler;
pub mod record_handler;
pub mod rule_handler;
pub mod relationship_handler;
pub mod import_export_handler;
pub mod audit_handler;
pub mod display_config_handler;
pub mod error;

use std::sync::Arc;
use axum::routing::{get, post, put, delete};
use axum::Router;
use axum::middleware::from_fn_with_state;
use tower_http::trace::TraceLayer;
use crate::adapter::middleware::auth::{MasterMaintenanceAuthState, auth_middleware};
use crate::adapter::middleware::rbac::require_permission;
use crate::usecase;

#[derive(Clone)]
pub struct AppState {
    pub manage_tables_uc: Arc<usecase::manage_table_definitions::ManageTableDefinitionsUseCase>,
    pub manage_columns_uc: Arc<usecase::manage_column_definitions::ManageColumnDefinitionsUseCase>,
    pub crud_records_uc: Arc<usecase::crud_records::CrudRecordsUseCase>,
    pub manage_rules_uc: Arc<usecase::manage_rules::ManageRulesUseCase>,
    pub check_consistency_uc: Arc<usecase::check_consistency::CheckConsistencyUseCase>,
    pub get_audit_logs_uc: Arc<usecase::get_audit_logs::GetAuditLogsUseCase>,
    pub manage_relationships_uc: Arc<usecase::manage_relationships::ManageRelationshipsUseCase>,
    pub manage_display_configs_uc: Arc<usecase::manage_display_configs::ManageDisplayConfigsUseCase>,
    pub import_export_uc: Arc<usecase::import_export::ImportExportUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<MasterMaintenanceAuthState>,
}

pub fn router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/healthz", get(table_handler::healthz))
        .route("/readyz", get(table_handler::readyz))
        .route("/metrics", get(table_handler::metrics_handler));

    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // Read-only routes (sys_auditor 以上)
        let read_routes = Router::new()
            .route("/api/v1/tables", get(table_handler::list_tables))
            .route("/api/v1/tables/:name", get(table_handler::get_table))
            .route("/api/v1/tables/:name/schema", get(table_handler::get_table_schema))
            .route("/api/v1/tables/:name/columns", get(table_handler::list_columns))
            .route("/api/v1/tables/:name/records", get(record_handler::list_records))
            .route("/api/v1/tables/:name/records/:id", get(record_handler::get_record))
            .route("/api/v1/relationships", get(relationship_handler::list_relationships))
            .route("/api/v1/tables/:name/related-records/:id", get(relationship_handler::get_related_records))
            .route("/api/v1/rules", get(rule_handler::list_rules))
            .route("/api/v1/rules/:id", get(rule_handler::get_rule))
            .route("/api/v1/tables/:name/export", get(import_export_handler::export_records))
            .route("/api/v1/import-jobs/:id", get(import_export_handler::get_import_job))
            .route("/api/v1/tables/:name/audit-logs", get(audit_handler::list_table_audit_logs))
            .route("/api/v1/tables/:name/records/:id/audit-logs", get(audit_handler::list_record_audit_logs))
            .route("/api/v1/tables/:name/display-configs", get(display_config_handler::list_display_configs))
            .route("/api/v1/tables/:name/display-configs/:id", get(display_config_handler::get_display_config))
            .route_layer(axum::middleware::from_fn(move |req, next| {
                let perm = require_permission("master_maintenance", "read");
                perm(req, next)
            }));

        // Write routes (sys_operator 以上)
        let write_routes = Router::new()
            .route("/api/v1/tables", post(table_handler::create_table))
            .route("/api/v1/tables/:name", put(table_handler::update_table))
            .route("/api/v1/tables/:name/columns", post(table_handler::create_columns))
            .route("/api/v1/tables/:name/columns/:column", put(table_handler::update_column))
            .route("/api/v1/tables/:name/records", post(record_handler::create_record))
            .route("/api/v1/tables/:name/records/:id", put(record_handler::update_record))
            .route("/api/v1/relationships", post(relationship_handler::create_relationship))
            .route("/api/v1/relationships/:id", put(relationship_handler::update_relationship))
            .route("/api/v1/rules", post(rule_handler::create_rule))
            .route("/api/v1/rules/:id", put(rule_handler::update_rule))
            .route("/api/v1/rules/:id/execute", post(rule_handler::execute_rule))
            .route("/api/v1/rules/check", post(rule_handler::check_rules))
            .route("/api/v1/tables/:name/import", post(import_export_handler::import_records))
            .route("/api/v1/tables/:name/display-configs", post(display_config_handler::create_display_config))
            .route("/api/v1/tables/:name/display-configs/:id", put(display_config_handler::update_display_config))
            .route_layer(axum::middleware::from_fn(move |req, next| {
                let perm = require_permission("master_maintenance", "write");
                perm(req, next)
            }));

        // Admin routes (sys_admin のみ)
        let admin_routes = Router::new()
            .route("/api/v1/tables/:name", delete(table_handler::delete_table))
            .route("/api/v1/tables/:name/columns/:column", delete(table_handler::delete_column))
            .route("/api/v1/tables/:name/records/:id", delete(record_handler::delete_record))
            .route("/api/v1/relationships/:id", delete(relationship_handler::delete_relationship))
            .route("/api/v1/rules/:id", delete(rule_handler::delete_rule))
            .route("/api/v1/tables/:name/display-configs/:id", delete(display_config_handler::delete_display_config))
            .route_layer(axum::middleware::from_fn(move |req, next| {
                let perm = require_permission("master_maintenance", "admin");
                perm(req, next)
            }));

        read_routes
            .merge(write_routes)
            .merge(admin_routes)
            .layer(from_fn_with_state(auth_state.clone(), auth_middleware))
    } else {
        // 認証なし（開発環境用）
        Router::new()
            .route("/api/v1/tables", get(table_handler::list_tables).post(table_handler::create_table))
            .route("/api/v1/tables/:name", get(table_handler::get_table).put(table_handler::update_table).delete(table_handler::delete_table))
            .route("/api/v1/tables/:name/schema", get(table_handler::get_table_schema))
            .route("/api/v1/tables/:name/columns", get(table_handler::list_columns).post(table_handler::create_columns))
            .route("/api/v1/tables/:name/columns/:column", put(table_handler::update_column).delete(table_handler::delete_column))
            .route("/api/v1/tables/:name/records", get(record_handler::list_records).post(record_handler::create_record))
            .route("/api/v1/tables/:name/records/:id", get(record_handler::get_record).put(record_handler::update_record).delete(record_handler::delete_record))
            .route("/api/v1/relationships", get(relationship_handler::list_relationships).post(relationship_handler::create_relationship))
            .route("/api/v1/relationships/:id", put(relationship_handler::update_relationship).delete(relationship_handler::delete_relationship))
            .route("/api/v1/tables/:name/related-records/:id", get(relationship_handler::get_related_records))
            .route("/api/v1/rules", get(rule_handler::list_rules).post(rule_handler::create_rule))
            .route("/api/v1/rules/:id", get(rule_handler::get_rule).put(rule_handler::update_rule).delete(rule_handler::delete_rule))
            .route("/api/v1/rules/:id/execute", post(rule_handler::execute_rule))
            .route("/api/v1/rules/check", post(rule_handler::check_rules))
            .route("/api/v1/tables/:name/import", post(import_export_handler::import_records))
            .route("/api/v1/tables/:name/export", get(import_export_handler::export_records))
            .route("/api/v1/import-jobs/:id", get(import_export_handler::get_import_job))
            .route("/api/v1/tables/:name/audit-logs", get(audit_handler::list_table_audit_logs))
            .route("/api/v1/tables/:name/records/:id/audit-logs", get(audit_handler::list_record_audit_logs))
            .route("/api/v1/tables/:name/display-configs", get(display_config_handler::list_display_configs).post(display_config_handler::create_display_config))
            .route("/api/v1/tables/:name/display-configs/:id", get(display_config_handler::get_display_config).put(display_config_handler::update_display_config).delete(display_config_handler::delete_display_config))
    };

    public_routes
        .merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
