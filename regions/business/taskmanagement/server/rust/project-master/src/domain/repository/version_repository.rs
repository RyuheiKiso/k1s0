// バージョンリポジトリ trait。
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::status_definition_version::StatusDefinitionVersion;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait VersionRepository: Send + Sync {
    async fn find_by_status_definition(
        &self,
        status_definition_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<StatusDefinitionVersion>>;
    async fn count_by_status_definition(
        &self,
        status_definition_id: Uuid,
    ) -> anyhow::Result<i64>;
}
