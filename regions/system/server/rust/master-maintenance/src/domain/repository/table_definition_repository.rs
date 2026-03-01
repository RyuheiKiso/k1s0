use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::table_definition::{TableDefinition, CreateTableDefinition, UpdateTableDefinition};

#[async_trait]
pub trait TableDefinitionRepository: Send + Sync {
    async fn find_all(&self, category: Option<&str>, active_only: bool) -> anyhow::Result<Vec<TableDefinition>>;
    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<TableDefinition>>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableDefinition>>;
    async fn create(&self, input: &CreateTableDefinition, created_by: &str) -> anyhow::Result<TableDefinition>;
    async fn update(&self, name: &str, input: &UpdateTableDefinition) -> anyhow::Result<TableDefinition>;
    async fn delete(&self, name: &str) -> anyhow::Result<()>;
}
