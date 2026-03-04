use async_trait::async_trait;

use crate::domain::entity::notification_log::NotificationLog;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NotificationLogRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<NotificationLog>>;
    async fn find_by_channel_id(&self, channel_id: &str) -> anyhow::Result<Vec<NotificationLog>>;
    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        channel_id: Option<String>,
        status: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationLog>, u64)>;
    async fn create(&self, log: &NotificationLog) -> anyhow::Result<()>;
    async fn update(&self, log: &NotificationLog) -> anyhow::Result<()>;
}
