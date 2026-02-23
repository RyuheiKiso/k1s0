use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::notification_log::NotificationLog;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NotificationLogRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<NotificationLog>>;
    async fn find_by_channel_id(&self, channel_id: &Uuid) -> anyhow::Result<Vec<NotificationLog>>;
    async fn create(&self, log: &NotificationLog) -> anyhow::Result<()>;
    async fn update(&self, log: &NotificationLog) -> anyhow::Result<()>;
}
