use async_trait::async_trait;

use crate::domain::entity::notification_template::NotificationTemplate;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NotificationTemplateRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<NotificationTemplate>>;
    async fn find_all(&self) -> anyhow::Result<Vec<NotificationTemplate>>;
    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationTemplate>, u64)>;
    async fn create(&self, template: &NotificationTemplate) -> anyhow::Result<()>;
    async fn update(&self, template: &NotificationTemplate) -> anyhow::Result<()>;
    async fn delete(&self, id: &str) -> anyhow::Result<bool>;
}
