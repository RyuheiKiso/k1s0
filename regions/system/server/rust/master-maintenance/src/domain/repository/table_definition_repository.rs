use crate::domain::entity::table_definition::{
    CreateTableDefinition, TableDefinition, UpdateTableDefinition,
};
use crate::domain::value_object::domain_filter::DomainFilter;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait TableDefinitionRepository: Send + Sync {
    async fn find_all(
        &self,
        category: Option<&str>,
        active_only: bool,
        domain_filter: &DomainFilter,
    ) -> anyhow::Result<Vec<TableDefinition>>;
    async fn find_by_name(
        &self,
        name: &str,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<Option<TableDefinition>>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableDefinition>>;
    async fn create(
        &self,
        input: &CreateTableDefinition,
        created_by: &str,
    ) -> anyhow::Result<TableDefinition>;
    async fn update(
        &self,
        name: &str,
        input: &UpdateTableDefinition,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<TableDefinition>;
    async fn delete(&self, name: &str, domain_scope: Option<&str>) -> anyhow::Result<()>;
    async fn find_domains(&self) -> anyhow::Result<Vec<(String, i64)>>;
}
