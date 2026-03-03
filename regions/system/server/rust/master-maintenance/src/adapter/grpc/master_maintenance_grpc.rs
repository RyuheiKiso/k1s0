use std::sync::Arc;
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::repository::table_relationship_repository::TableRelationshipRepository;
use crate::usecase;

pub struct MasterMaintenanceGrpcService {
    pub manage_tables_uc: Arc<usecase::manage_table_definitions::ManageTableDefinitionsUseCase>,
    pub manage_columns_uc: Arc<usecase::manage_column_definitions::ManageColumnDefinitionsUseCase>,
    pub crud_records_uc: Arc<usecase::crud_records::CrudRecordsUseCase>,
    pub check_consistency_uc: Arc<usecase::check_consistency::CheckConsistencyUseCase>,
    pub get_audit_logs_uc: Arc<usecase::get_audit_logs::GetAuditLogsUseCase>,
    pub manage_relationships_uc: Arc<usecase::manage_relationships::ManageRelationshipsUseCase>,
    pub manage_display_configs_uc: Arc<usecase::manage_display_configs::ManageDisplayConfigsUseCase>,
    pub import_export_uc: Arc<usecase::import_export::ImportExportUseCase>,
    pub column_repo: Arc<dyn ColumnDefinitionRepository>,
    pub relationship_repo: Arc<dyn TableRelationshipRepository>,
}

impl MasterMaintenanceGrpcService {
    pub fn new(
        manage_tables_uc: Arc<usecase::manage_table_definitions::ManageTableDefinitionsUseCase>,
        manage_columns_uc: Arc<usecase::manage_column_definitions::ManageColumnDefinitionsUseCase>,
        crud_records_uc: Arc<usecase::crud_records::CrudRecordsUseCase>,
        check_consistency_uc: Arc<usecase::check_consistency::CheckConsistencyUseCase>,
        get_audit_logs_uc: Arc<usecase::get_audit_logs::GetAuditLogsUseCase>,
        manage_relationships_uc: Arc<usecase::manage_relationships::ManageRelationshipsUseCase>,
        manage_display_configs_uc: Arc<usecase::manage_display_configs::ManageDisplayConfigsUseCase>,
        import_export_uc: Arc<usecase::import_export::ImportExportUseCase>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
        relationship_repo: Arc<dyn TableRelationshipRepository>,
    ) -> Self {
        Self {
            manage_tables_uc,
            manage_columns_uc,
            crud_records_uc,
            check_consistency_uc,
            get_audit_logs_uc,
            manage_relationships_uc,
            manage_display_configs_uc,
            import_export_uc,
            column_repo,
            relationship_repo,
        }
    }
}
