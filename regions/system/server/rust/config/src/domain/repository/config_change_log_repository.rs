use async_trait::async_trait;

use crate::domain::entity::config_change_log::ConfigChangeLog;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ConfigChangeLogRepository: Send + Sync {
    async fn record_change_log(&self, log: &ConfigChangeLog) -> anyhow::Result<()>;

    async fn list_change_logs(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Vec<ConfigChangeLog>>;
}
