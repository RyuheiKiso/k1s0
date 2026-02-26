use async_trait::async_trait;

use crate::domain::entity::access_log::SecretAccessLog;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AccessLogRepository: Send + Sync {
    async fn record(&self, log: &SecretAccessLog) -> anyhow::Result<()>;
    async fn list(&self, offset: u32, limit: u32) -> anyhow::Result<Vec<SecretAccessLog>>;
}
