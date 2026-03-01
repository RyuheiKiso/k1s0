use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::table_relationship::TableRelationship;

#[async_trait]
pub trait TableRelationshipRepository: Send + Sync {
    async fn find_all(&self) -> anyhow::Result<Vec<TableRelationship>>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableRelationship>>;
    async fn find_by_source_table(&self, table_id: Uuid) -> anyhow::Result<Vec<TableRelationship>>;
    async fn create(&self, rel: &TableRelationship) -> anyhow::Result<TableRelationship>;
    async fn update(&self, id: Uuid, rel: &TableRelationship) -> anyhow::Result<TableRelationship>;
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
}
