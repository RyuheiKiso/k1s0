use crate::domain::entity::column_definition::ColumnDefinition;
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::infrastructure::schema::PhysicalSchemaManager;
use std::sync::Arc;

pub struct ManageColumnDefinitionsUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
    schema_manager: Arc<PhysicalSchemaManager>,
}

impl ManageColumnDefinitionsUseCase {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
        schema_manager: Arc<PhysicalSchemaManager>,
    ) -> Self {
        Self {
            table_repo,
            column_repo,
            schema_manager,
        }
    }

    pub async fn list_columns(
        &self,
        table_name: &str,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<Vec<ColumnDefinition>> {
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table not found"))?;
        self.column_repo.find_by_table_id(table.id).await
    }

    pub async fn create_columns(
        &self,
        table_name: &str,
        input: &serde_json::Value,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<Vec<ColumnDefinition>> {
        let table = self
            .table_repo
            .find_by_name(table_name, domain_scope)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table not found"))?;
        let columns: Vec<crate::domain::entity::column_definition::CreateColumnDefinition> =
            serde_json::from_value(input.get("columns").cloned().unwrap_or_default())?;
        self.schema_manager.add_columns(&table, &columns).await?;
        self.column_repo.create_batch(table.id, &columns).await
    }

    pub async fn update_column(
        &self,
        table_name: &str,
        column_name: &str,
        input: &serde_json::Value,
    ) -> anyhow::Result<ColumnDefinition> {
        let table = self
            .table_repo
            .find_by_name(table_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table not found"))?;
        let existing = self
            .column_repo
            .find_by_table_and_column(table.id, column_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Column not found"))?;
        let col_def: crate::domain::entity::column_definition::CreateColumnDefinition =
            serde_json::from_value(input.clone())?;
        self.schema_manager
            .update_column(&table, &existing, &col_def)
            .await?;
        self.column_repo
            .update(table.id, column_name, &col_def)
            .await
    }

    pub async fn delete_column(&self, table_name: &str, column_name: &str) -> anyhow::Result<()> {
        let table = self
            .table_repo
            .find_by_name(table_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table not found"))?;
        self.schema_manager
            .delete_column(&table, column_name)
            .await?;
        self.column_repo.delete(table.id, column_name).await
    }
}
