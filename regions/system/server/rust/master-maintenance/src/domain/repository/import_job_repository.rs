use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::import_job::ImportJob;

#[async_trait]
pub trait ImportJobRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ImportJob>>;
    async fn create(&self, job: &ImportJob) -> anyhow::Result<ImportJob>;
    async fn update(&self, id: Uuid, job: &ImportJob) -> anyhow::Result<ImportJob>;
}
