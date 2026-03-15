use crate::domain::model::{
    CreateChannelPayload, CreateTemplatePayload, DeleteChannelPayload, DeleteTemplatePayload,
    RetryNotificationPayload, SendNotificationPayload, UpdateChannelPayload, UpdateTemplatePayload,
    UserError,
};
use crate::infrastructure::grpc::NotificationGrpcClient;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::instrument;

pub struct NotificationMutationResolver {
    client: Arc<NotificationGrpcClient>,
}

impl NotificationMutationResolver {
    pub fn new(client: Arc<NotificationGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self, template_variables), fields(service = "graphql-gateway"))]
    pub async fn send_notification(
        &self,
        channel_id: &str,
        template_id: Option<&str>,
        template_variables: &HashMap<String, String>,
        recipient: &str,
        subject: Option<&str>,
        body: Option<&str>,
    ) -> SendNotificationPayload {
        match self
            .client
            .send_notification(
                channel_id,
                template_id,
                template_variables,
                recipient,
                subject,
                body,
            )
            .await
        {
            Ok((notification_id, status, _created_at)) => SendNotificationPayload {
                notification_id: Some(notification_id),
                status: Some(status),
                errors: vec![],
            },
            Err(e) => SendNotificationPayload {
                notification_id: None,
                status: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn retry_notification(&self, notification_id: &str) -> RetryNotificationPayload {
        match self.client.retry_notification(notification_id).await {
            Ok(notification) => RetryNotificationPayload {
                notification: Some(notification),
                errors: vec![],
            },
            Err(e) => RetryNotificationPayload {
                notification: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_channel(
        &self,
        name: &str,
        channel_type: &str,
        config_json: &str,
        enabled: bool,
    ) -> CreateChannelPayload {
        match self
            .client
            .create_channel(name, channel_type, config_json, enabled)
            .await
        {
            Ok(channel) => CreateChannelPayload {
                channel: Some(channel),
                errors: vec![],
            },
            Err(e) => CreateChannelPayload {
                channel: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_channel(
        &self,
        id: &str,
        name: Option<&str>,
        enabled: Option<bool>,
        config_json: Option<&str>,
    ) -> UpdateChannelPayload {
        match self
            .client
            .update_channel(id, name, enabled, config_json)
            .await
        {
            Ok(channel) => UpdateChannelPayload {
                channel: Some(channel),
                errors: vec![],
            },
            Err(e) => UpdateChannelPayload {
                channel: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_channel(&self, id: &str) -> DeleteChannelPayload {
        match self.client.delete_channel(id).await {
            Ok(success) => DeleteChannelPayload {
                success,
                errors: vec![],
            },
            Err(e) => DeleteChannelPayload {
                success: false,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_template(
        &self,
        name: &str,
        channel_type: &str,
        subject_template: Option<&str>,
        body_template: &str,
    ) -> CreateTemplatePayload {
        match self
            .client
            .create_template(name, channel_type, subject_template, body_template)
            .await
        {
            Ok(template) => CreateTemplatePayload {
                template: Some(template),
                errors: vec![],
            },
            Err(e) => CreateTemplatePayload {
                template: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_template(
        &self,
        id: &str,
        name: Option<&str>,
        subject_template: Option<&str>,
        body_template: Option<&str>,
    ) -> UpdateTemplatePayload {
        match self
            .client
            .update_template(id, name, subject_template, body_template)
            .await
        {
            Ok(template) => UpdateTemplatePayload {
                template: Some(template),
                errors: vec![],
            },
            Err(e) => UpdateTemplatePayload {
                template: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_template(&self, id: &str) -> DeleteTemplatePayload {
        match self.client.delete_template(id).await {
            Ok(success) => DeleteTemplatePayload {
                success,
                errors: vec![],
            },
            Err(e) => DeleteTemplatePayload {
                success: false,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }
}
