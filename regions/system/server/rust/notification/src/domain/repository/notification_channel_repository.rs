use async_trait::async_trait;

use crate::domain::entity::notification_channel::NotificationChannel;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NotificationChannelRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<NotificationChannel>>;
    #[allow(dead_code)]
    async fn find_all(&self) -> anyhow::Result<Vec<NotificationChannel>>;
    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
        enabled_only: bool,
    ) -> anyhow::Result<(Vec<NotificationChannel>, u64)>;
    async fn create(&self, channel: &NotificationChannel) -> anyhow::Result<()>;
    async fn update(&self, channel: &NotificationChannel) -> anyhow::Result<()>;
    async fn delete(&self, id: &str) -> anyhow::Result<bool>;
}
