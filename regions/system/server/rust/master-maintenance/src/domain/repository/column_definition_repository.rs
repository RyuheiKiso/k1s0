use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::column_definition::{ColumnDefinition, CreateColumnDefinition};

#[async_trait]
pub trait ColumnDefinitionRepository: Send + Sync {
    async fn find_by_table_id(&self, table_id: Uuid) -> anyhow::Result<Vec<ColumnDefinition>>;
    async fn find_by_table_and_column(&self, table_id: Uuid, column_name: &str) -> anyhow::Result<Option<ColumnDefinition>>;
    async fn create_batch(&self, table_id: Uuid, columns: &[CreateColumnDefinition]) -> anyhow::Result<Vec<ColumnDefinition>>;
    async fn update(&self, table_id: Uuid, column_name: &str, input: &CreateColumnDefinition) -> anyhow::Result<ColumnDefinition>;
    async fn delete(&self, table_id: Uuid, column_name: &str) -> anyhow::Result<()>;
}
