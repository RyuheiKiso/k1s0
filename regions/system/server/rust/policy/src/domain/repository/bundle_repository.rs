use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::policy_bundle::PolicyBundle;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PolicyBundleRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<PolicyBundle>>;
    async fn find_all(&self) -> anyhow::Result<Vec<PolicyBundle>>;
    async fn create(&self, bundle: &PolicyBundle) -> anyhow::Result<()>;
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool>;
}
