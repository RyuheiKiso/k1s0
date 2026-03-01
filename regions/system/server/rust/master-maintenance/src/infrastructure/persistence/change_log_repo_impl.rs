use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entity::change_log::ChangeLog;
use crate::domain::repository::change_log_repository::ChangeLogRepository;

pub struct ChangeLogPostgresRepository {
    pool: PgPool,
}

impl ChangeLogPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChangeLogRepository for ChangeLogPostgresRepository {
    async fn create(&self, _log: &ChangeLog) -> anyhow::Result<ChangeLog> {
        todo!()
    }
    async fn find_by_table(&self, _table_name: &str, _page: i32, _page_size: i32) -> anyhow::Result<(Vec<ChangeLog>, i64)> {
        todo!()
    }
    async fn find_by_record(&self, _table_name: &str, _record_id: &str, _page: i32, _page_size: i32) -> anyhow::Result<(Vec<ChangeLog>, i64)> {
        todo!()
    }
}
