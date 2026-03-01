use std::sync::Arc;
use crate::domain::entity::column_definition::ColumnDefinition;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;

pub struct ManageColumnDefinitionsUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
}

impl ManageColumnDefinitionsUseCase {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
    ) -> Self {
        Self { table_repo, column_repo }
    }

    pub async fn list_columns(&self, table_name: &str) -> anyhow::Result<Vec<ColumnDefinition>> {
        let table = self.table_repo.find_by_name(table_name).await?
            .ok_or_else(|| anyhow::anyhow!("Table not found"))?;
        self.column_repo.find_by_table_id(table.id).await
    }

    pub async fn create_columns(&self, table_name: &str, input: &serde_json::Value) -> anyhow::Result<Vec<ColumnDefinition>> {
        let table = self.table_repo.find_by_name(table_name).await?
            .ok_or_else(|| anyhow::anyhow!("Table not found"))?;
        let columns: Vec<crate::domain::entity::column_definition::CreateColumnDefinition> =
            serde_json::from_value(input.get("columns").cloned().unwrap_or_default())?;
        self.column_repo.create_batch(table.id, &columns).await
    }

    pub async fn update_column(&self, table_name: &str, column_name: &str, input: &serde_json::Value) -> anyhow::Result<ColumnDefinition> {
        let table = self.table_repo.find_by_name(table_name).await?
            .ok_or_else(|| anyhow::anyhow!("Table not found"))?;
        let col_def: crate::domain::entity::column_definition::CreateColumnDefinition = serde_json::from_value(input.clone())?;
        self.column_repo.update(table.id, column_name, &col_def).await
    }

    pub async fn delete_column(&self, table_name: &str, column_name: &str) -> anyhow::Result<()> {
        let table = self.table_repo.find_by_name(table_name).await?
            .ok_or_else(|| anyhow::anyhow!("Table not found"))?;
        self.column_repo.delete(table.id, column_name).await
    }
}
