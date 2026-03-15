use crate::domain::model::{NotificationChannel, NotificationLog, NotificationTemplate};
use crate::infrastructure::grpc::NotificationGrpcClient;
use std::sync::Arc;
use tracing::instrument;

pub struct NotificationQueryResolver {
    client: Arc<NotificationGrpcClient>,
}

impl NotificationQueryResolver {
    pub fn new(client: Arc<NotificationGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_notification(
        &self,
        notification_id: &str,
    ) -> anyhow::Result<Option<NotificationLog>> {
        self.client.get_notification(notification_id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_notifications(
        &self,
        channel_id: Option<&str>,
        status: Option<&str>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> anyhow::Result<Vec<NotificationLog>> {
        self.client
            .list_notifications(channel_id, status, page, page_size)
            .await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_channel(&self, id: &str) -> anyhow::Result<Option<NotificationChannel>> {
        self.client.get_channel(id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_channels(
        &self,
        channel_type: Option<&str>,
        enabled_only: bool,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> anyhow::Result<Vec<NotificationChannel>> {
        self.client
            .list_channels(channel_type, enabled_only, page, page_size)
            .await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_template(&self, id: &str) -> anyhow::Result<Option<NotificationTemplate>> {
        self.client.get_template(id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_templates(
        &self,
        channel_type: Option<&str>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> anyhow::Result<Vec<NotificationTemplate>> {
        self.client
            .list_templates(channel_type, page, page_size)
            .await
    }
}
