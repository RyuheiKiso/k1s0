use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::notification_template::NotificationTemplate;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NotificationTemplateRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<NotificationTemplate>>;
    async fn find_all(&self) -> anyhow::Result<Vec<NotificationTemplate>>;
    async fn create(&self, template: &NotificationTemplate) -> anyhow::Result<()>;
    async fn update(&self, template: &NotificationTemplate) -> anyhow::Result<()>;
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool>;
}
