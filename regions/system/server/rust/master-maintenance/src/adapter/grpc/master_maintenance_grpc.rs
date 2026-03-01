use std::sync::Arc;
use crate::usecase;

pub struct MasterMaintenanceGrpcService {
    pub manage_tables_uc: Arc<usecase::manage_table_definitions::ManageTableDefinitionsUseCase>,
    pub crud_records_uc: Arc<usecase::crud_records::CrudRecordsUseCase>,
    pub check_consistency_uc: Arc<usecase::check_consistency::CheckConsistencyUseCase>,
}

impl MasterMaintenanceGrpcService {
    pub fn new(
        manage_tables_uc: Arc<usecase::manage_table_definitions::ManageTableDefinitionsUseCase>,
        crud_records_uc: Arc<usecase::crud_records::CrudRecordsUseCase>,
        check_consistency_uc: Arc<usecase::check_consistency::CheckConsistencyUseCase>,
    ) -> Self {
        Self {
            manage_tables_uc,
            crud_records_uc,
            check_consistency_uc,
        }
    }
}
