use std::sync::Arc;
use uuid::Uuid;
use crate::domain::entity::table_definition::{TableDefinition, CreateTableDefinition, UpdateTableDefinition};
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::service::metadata_service::SchemaGeneratorService;

pub struct ManageTableDefinitionsUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
}

impl ManageTableDefinitionsUseCase {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
    ) -> Self {
        Self { table_repo, column_repo }
    }

    pub async fn list_tables(&self, category: Option<&str>, active_only: bool) -> anyhow::Result<Vec<TableDefinition>> {
        self.table_repo.find_all(category, active_only).await
    }

    pub async fn get_table(&self, name: &str) -> anyhow::Result<Option<TableDefinition>> {
        self.table_repo.find_by_name(name).await
    }

    pub async fn get_table_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableDefinition>> {
        self.table_repo.find_by_id(id).await
    }

    pub async fn create_table(&self, input: &CreateTableDefinition, created_by: &str) -> anyhow::Result<TableDefinition> {
        self.table_repo.create(input, created_by).await
    }

    pub async fn update_table(&self, name: &str, input: &UpdateTableDefinition) -> anyhow::Result<TableDefinition> {
        self.table_repo.update(name, input).await
    }

    pub async fn delete_table(&self, name: &str) -> anyhow::Result<()> {
        self.table_repo.delete(name).await
    }

    pub async fn get_table_schema(&self, name: &str) -> anyhow::Result<serde_json::Value> {
        let table = self.table_repo.find_by_name(name).await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", name))?;
        let columns = self.column_repo.find_by_table_id(table.id).await?;
        Ok(SchemaGeneratorService::generate_json_schema(&table, &columns))
    }
}
