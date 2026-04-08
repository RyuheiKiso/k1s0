use std::sync::Arc;

use crate::domain::entity::feature_flag::FeatureFlag;
use crate::domain::repository::FeatureFlagRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetFlagError {
    #[error("flag not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetFlagUseCase {
    repo: Arc<dyn FeatureFlagRepository>,
}

impl GetFlagUseCase {
    pub fn new(repo: Arc<dyn FeatureFlagRepository>) -> Self {
        Self { repo }
    }

    /// STATIC-CRITICAL-001 監査対応: テナントスコープでフィーチャーフラグを取得する。
    /// HIGH-005 対応: `tenant_id` は &str 型（migration 006 で DB の TEXT 型に変更済み）。
    pub async fn execute(&self, tenant_id: &str, flag_key: &str) -> Result<FeatureFlag, GetFlagError> {
        self.repo.find_by_key(tenant_id, flag_key).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GetFlagError::NotFound(flag_key.to_string())
            } else {
                GetFlagError::Internal(msg)
            }
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::feature_flag::FeatureFlag;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;
    use chrono::Utc;
    use uuid::Uuid;

    /// システムテナント文字列: テスト共通（HIGH-005 対応: TEXT 型）
    fn system_tenant() -> &'static str {
        "00000000-0000-0000-0000-000000000001"
    }

    #[tokio::test]
    async fn found() {
        let mut mock = MockFeatureFlagRepository::new();
        let flag = FeatureFlag {
            id: Uuid::new_v4(),
            tenant_id: system_tenant().to_string(),
            flag_key: "dark-mode".to_string(),
            description: "Dark mode".to_string(),
            enabled: true,
            variants: vec![],
            rules: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let return_flag = flag.clone();

        // STATIC-CRITICAL-001: tenant_id を含む2引数シグネチャ
        mock.expect_find_by_key()
            .withf(|_tid, key| key == "dark-mode")
            .returning(move |_, _| Ok(return_flag.clone()));

        let uc = GetFlagUseCase::new(Arc::new(mock));
        let result = uc.execute(system_tenant(), "dark-mode").await;
        assert!(result.is_ok());

        let flag = result.unwrap();
        assert_eq!(flag.flag_key, "dark-mode");
        assert!(flag.enabled);
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_find_by_key()
            .returning(|_, _| Err(anyhow::anyhow!("flag not found")));

        let uc = GetFlagUseCase::new(Arc::new(mock));
        let result = uc.execute(system_tenant(), "nonexistent").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetFlagError::NotFound(key) => assert_eq!(key, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
