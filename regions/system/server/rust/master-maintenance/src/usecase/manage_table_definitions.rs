use crate::domain::entity::table_definition::{
    CreateTableDefinition, TableDefinition, UpdateTableDefinition,
};
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::service::metadata_service::SchemaGeneratorService;
use crate::domain::value_object::domain_filter::DomainFilter;
use crate::infrastructure::schema::PhysicalSchemaManager;
use std::sync::Arc;
use uuid::Uuid;

pub struct ManageTableDefinitionsUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
    schema_manager: Arc<PhysicalSchemaManager>,
}

impl ManageTableDefinitionsUseCase {
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

    pub async fn list_tables(
        &self,
        category: Option<&str>,
        active_only: bool,
        domain_filter: &DomainFilter,
    ) -> anyhow::Result<Vec<TableDefinition>> {
        self.table_repo
            .find_all(category, active_only, domain_filter)
            .await
    }

    pub async fn get_table(
        &self,
        name: &str,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<Option<TableDefinition>> {
        self.table_repo.find_by_name(name, domain_scope).await
    }

    pub async fn get_table_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableDefinition>> {
        self.table_repo.find_by_id(id).await
    }

    pub async fn create_table(
        &self,
        input: &CreateTableDefinition,
        created_by: &str,
    ) -> anyhow::Result<TableDefinition> {
        self.schema_manager.create_table(input).await?;
        self.table_repo.create(input, created_by).await
    }

    pub async fn update_table(
        &self,
        name: &str,
        input: &UpdateTableDefinition,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<TableDefinition> {
        self.table_repo.update(name, input, domain_scope).await
    }

    pub async fn delete_table(&self, name: &str, domain_scope: Option<&str>) -> anyhow::Result<()> {
        let table = self
            .table_repo
            .find_by_name(name, domain_scope)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", name))?;
        self.schema_manager.delete_table(&table).await?;
        self.table_repo.delete(name, domain_scope).await
    }

    pub async fn get_table_schema(
        &self,
        name: &str,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        let table = self
            .table_repo
            .find_by_name(name, domain_scope)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", name))?;
        let columns = self.column_repo.find_by_table_id(table.id).await?;
        Ok(SchemaGeneratorService::generate_json_schema(
            &table, &columns,
        ))
    }

    pub async fn list_domains(&self) -> anyhow::Result<Vec<(String, i64)>> {
        self.table_repo.find_domains().await
    }
}
