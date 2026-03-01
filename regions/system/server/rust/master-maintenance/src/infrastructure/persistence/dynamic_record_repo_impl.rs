use async_trait::async_trait;
use sqlx::PgPool;
use serde_json::Value;
use crate::domain::entity::table_definition::TableDefinition;
use crate::domain::entity::column_definition::ColumnDefinition;
use crate::domain::repository::dynamic_record_repository::DynamicRecordRepository;

pub struct DynamicRecordPostgresRepository {
    pool: PgPool,
}

impl DynamicRecordPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DynamicRecordRepository for DynamicRecordPostgresRepository {
    async fn find_all(
        &self, _table_def: &TableDefinition, _columns: &[ColumnDefinition],
        _page: i32, _page_size: i32, _sort: Option<&str>, _filter: Option<&str>, _search: Option<&str>,
    ) -> anyhow::Result<(Vec<Value>, i64)> {
        todo!()
    }
    async fn find_by_id(&self, _table_def: &TableDefinition, _columns: &[ColumnDefinition], _record_id: &str) -> anyhow::Result<Option<Value>> {
        todo!()
    }
    async fn create(&self, _table_def: &TableDefinition, _columns: &[ColumnDefinition], _data: &Value) -> anyhow::Result<Value> {
        todo!()
    }
    async fn update(&self, _table_def: &TableDefinition, _columns: &[ColumnDefinition], _record_id: &str, _data: &Value) -> anyhow::Result<Value> {
        todo!()
    }
    async fn delete(&self, _table_def: &TableDefinition, _record_id: &str) -> anyhow::Result<()> {
        todo!()
    }
}
