// ステータス定義バージョン取得ユースケース。
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::status_definition_version::StatusDefinitionVersion;
use crate::domain::repository::version_repository::VersionRepository;

pub struct GetStatusDefinitionVersionsUseCase {
    repo: Arc<dyn VersionRepository>,
}

impl GetStatusDefinitionVersionsUseCase {
    pub fn new(repo: Arc<dyn VersionRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        status_definition_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<(Vec<StatusDefinitionVersion>, i64)> {
        let versions = self
            .repo
            .find_by_status_definition(status_definition_id, limit, offset)
            .await?;
        let total = self
            .repo
            .count_by_status_definition(status_definition_id)
            .await?;
        Ok((versions, total))
    }
}
