use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::feature_flag::FeatureFlag;
use crate::domain::repository::FeatureFlagRepository;

#[derive(Debug, thiserror::Error)]
pub enum ListFlagsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListFlagsUseCase {
    repo: Arc<dyn FeatureFlagRepository>,
}

impl ListFlagsUseCase {
    pub fn new(repo: Arc<dyn FeatureFlagRepository>) -> Self {
        Self { repo }
    }

    /// STATIC-CRITICAL-001 監査対応: テナントスコープのフィーチャーフラグ一覧を取得する。
    pub async fn execute(&self, tenant_id: Uuid) -> Result<Vec<FeatureFlag>, ListFlagsError> {
        self.repo
            .find_all(tenant_id)
            .await
            .map_err(|e| ListFlagsError::Internal(e.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;

    /// システムテナントUUID: テスト共通
    fn system_tenant() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    #[tokio::test]
    async fn test_list_flags_success() {
        let mut repo = MockFeatureFlagRepository::new();
        // STATIC-CRITICAL-001: tenant_id を含む1引数シグネチャ
        repo.expect_find_all().returning(|_| Ok(vec![]));

        let uc = ListFlagsUseCase::new(Arc::new(repo));
        let flags = uc.execute(system_tenant()).await.unwrap();
        assert!(flags.is_empty());
    }
}
