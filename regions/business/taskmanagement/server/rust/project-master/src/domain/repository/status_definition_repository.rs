// ステータス定義リポジトリ trait。
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::status_definition::{
    CreateStatusDefinition, StatusDefinition, StatusDefinitionFilter, UpdateStatusDefinition,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait StatusDefinitionRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<StatusDefinition>>;
    async fn find_all(
        &self,
        filter: &StatusDefinitionFilter,
    ) -> anyhow::Result<Vec<StatusDefinition>>;
    async fn count(&self, filter: &StatusDefinitionFilter) -> anyhow::Result<i64>;
    async fn create(
        &self,
        input: &CreateStatusDefinition,
        created_by: &str,
    ) -> anyhow::Result<StatusDefinition>;
    async fn update(
        &self,
        id: Uuid,
        input: &UpdateStatusDefinition,
        updated_by: &str,
    ) -> anyhow::Result<StatusDefinition>;
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
}
