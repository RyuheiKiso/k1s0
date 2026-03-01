use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::import_job::ImportJob;

#[async_trait]
pub trait ImportJobRepository: Send + Sync {
    async fn create(&self, job: &ImportJob) -> anyhow::Result<ImportJob>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ImportJob>>;
    async fn update_progress(&self, id: Uuid, processed: i32, errors: i32) -> anyhow::Result<()>;
    async fn complete(&self, id: Uuid, error_details: Option<serde_json::Value>) -> anyhow::Result<()>;
}
