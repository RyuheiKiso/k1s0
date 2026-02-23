use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::policy::Policy;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PolicyRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Policy>>;
    async fn find_all(&self) -> anyhow::Result<Vec<Policy>>;
    async fn create(&self, policy: &Policy) -> anyhow::Result<()>;
    async fn update(&self, policy: &Policy) -> anyhow::Result<()>;
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool>;
    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool>;
}
