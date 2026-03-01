use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::table_relationship::TableRelationship;

#[async_trait]
pub trait TableRelationshipRepository: Send + Sync {
    async fn find_all(&self) -> anyhow::Result<Vec<TableRelationship>>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableRelationship>>;
    async fn find_by_table_id(&self, table_id: Uuid) -> anyhow::Result<Vec<TableRelationship>>;
    async fn create(&self, relationship: &TableRelationship) -> anyhow::Result<TableRelationship>;
    async fn update(&self, id: Uuid, relationship: &TableRelationship) -> anyhow::Result<TableRelationship>;
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
}
