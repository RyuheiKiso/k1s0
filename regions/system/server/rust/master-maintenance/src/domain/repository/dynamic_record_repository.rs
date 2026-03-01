use async_trait::async_trait;
use serde_json::Value;
use crate::domain::entity::table_definition::TableDefinition;
use crate::domain::entity::column_definition::ColumnDefinition;

#[async_trait]
pub trait DynamicRecordRepository: Send + Sync {
    async fn find_all(
        &self,
        table_def: &TableDefinition,
        columns: &[ColumnDefinition],
        page: i32,
        page_size: i32,
        sort: Option<&str>,
        filter: Option<&str>,
        search: Option<&str>,
    ) -> anyhow::Result<(Vec<Value>, i64)>;

    async fn find_by_id(
        &self,
        table_def: &TableDefinition,
        columns: &[ColumnDefinition],
        record_id: &str,
    ) -> anyhow::Result<Option<Value>>;

    async fn create(
        &self,
        table_def: &TableDefinition,
        columns: &[ColumnDefinition],
        data: &Value,
    ) -> anyhow::Result<Value>;

    async fn update(
        &self,
        table_def: &TableDefinition,
        columns: &[ColumnDefinition],
        record_id: &str,
        data: &Value,
    ) -> anyhow::Result<Value>;

    async fn delete(
        &self,
        table_def: &TableDefinition,
        record_id: &str,
    ) -> anyhow::Result<()>;
}
