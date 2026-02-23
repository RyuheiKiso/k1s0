use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::feature_flag::FeatureFlag;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FeatureFlagRepository: Send + Sync {
    async fn find_by_key(&self, flag_key: &str) -> anyhow::Result<FeatureFlag>;
    async fn find_all(&self) -> anyhow::Result<Vec<FeatureFlag>>;
    async fn create(&self, flag: &FeatureFlag) -> anyhow::Result<()>;
    async fn update(&self, flag: &FeatureFlag) -> anyhow::Result<()>;
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool>;
    async fn exists_by_key(&self, flag_key: &str) -> anyhow::Result<bool>;
}
