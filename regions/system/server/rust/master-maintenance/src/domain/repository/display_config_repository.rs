use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::display_config::{DisplayConfig, CreateDisplayConfig};

#[async_trait]
pub trait DisplayConfigRepository: Send + Sync {
    async fn find_by_table_id(&self, table_id: Uuid) -> anyhow::Result<Vec<DisplayConfig>>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<DisplayConfig>>;
    async fn create(&self, table_id: Uuid, input: &CreateDisplayConfig, created_by: &str) -> anyhow::Result<DisplayConfig>;
    async fn update(&self, id: Uuid, input: &CreateDisplayConfig) -> anyhow::Result<DisplayConfig>;
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
}
