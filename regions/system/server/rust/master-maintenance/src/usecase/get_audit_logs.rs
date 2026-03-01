use std::sync::Arc;
use crate::domain::entity::change_log::ChangeLog;
use crate::domain::repository::change_log_repository::ChangeLogRepository;

pub struct GetAuditLogsUseCase {
    change_log_repo: Arc<dyn ChangeLogRepository>,
}

impl GetAuditLogsUseCase {
    pub fn new(change_log_repo: Arc<dyn ChangeLogRepository>) -> Self {
        Self { change_log_repo }
    }

    pub async fn get_table_logs(&self, table_name: &str, page: i32, page_size: i32) -> anyhow::Result<(Vec<ChangeLog>, i64)> {
        self.change_log_repo.find_by_table(table_name, page, page_size).await
    }

    pub async fn get_record_logs(&self, table_name: &str, record_id: &str, page: i32, page_size: i32) -> anyhow::Result<(Vec<ChangeLog>, i64)> {
        self.change_log_repo.find_by_record(table_name, record_id, page, page_size).await
    }
}
