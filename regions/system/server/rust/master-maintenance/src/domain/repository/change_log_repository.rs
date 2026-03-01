use async_trait::async_trait;
use crate::domain::entity::change_log::ChangeLog;

#[async_trait]
pub trait ChangeLogRepository: Send + Sync {
    async fn create(&self, log: &ChangeLog) -> anyhow::Result<ChangeLog>;
    async fn find_by_table(&self, table_name: &str, page: i32, page_size: i32) -> anyhow::Result<(Vec<ChangeLog>, i64)>;
    async fn find_by_record(&self, table_name: &str, record_id: &str, page: i32, page_size: i32) -> anyhow::Result<(Vec<ChangeLog>, i64)>;
}
