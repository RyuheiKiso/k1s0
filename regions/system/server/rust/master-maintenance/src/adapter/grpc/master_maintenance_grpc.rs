use std::sync::Arc;
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::repository::table_relationship_repository::TableRelationshipRepository;
use crate::usecase;

pub struct MasterMaintenanceGrpcService {
    pub manage_tables_uc: Arc<usecase::manage_table_definitions::ManageTableDefinitionsUseCase>,
    pub crud_records_uc: Arc<usecase::crud_records::CrudRecordsUseCase>,
    pub check_consistency_uc: Arc<usecase::check_consistency::CheckConsistencyUseCase>,
    pub column_repo: Arc<dyn ColumnDefinitionRepository>,
    pub relationship_repo: Arc<dyn TableRelationshipRepository>,
}

impl MasterMaintenanceGrpcService {
    pub fn new(
        manage_tables_uc: Arc<usecase::manage_table_definitions::ManageTableDefinitionsUseCase>,
        crud_records_uc: Arc<usecase::crud_records::CrudRecordsUseCase>,
        check_consistency_uc: Arc<usecase::check_consistency::CheckConsistencyUseCase>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
        relationship_repo: Arc<dyn TableRelationshipRepository>,
    ) -> Self {
        Self {
            manage_tables_uc,
            crud_records_uc,
            check_consistency_uc,
            column_repo,
            relationship_repo,
        }
    }
}
