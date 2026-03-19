use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use handlebars::Handlebars;

use crate::domain::entity::notification_log::NotificationLog;
use crate::domain::repository::NotificationChannelRepository;
use crate::domain::repository::NotificationLogRepository;
use crate::domain::repository::NotificationTemplateRepository;
use crate::domain::service::DeliveryClient;
#[cfg(test)]
#[allow(clippy::unwrap_used)]
use crate::domain::service::DeliveryError;
use crate::infrastructure::kafka_producer::{
    NoopNotificationEventPublisher, NotificationEventPublisher,
};

#[derive(Debug, Clone)]
pub struct SendNotificationInput {
    pub channel_id: String,
    pub template_id: Option<String>,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
    pub template_variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct SendNotificationOutput {
    pub log_id: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum SendNotificationError {
    #[error("channel not found: {0}")]
    ChannelNotFound(String),

    #[error("channel disabled: {0}")]
    ChannelDisabled(String),

    #[error("template not found: {0}")]
    TemplateNotFound(String),

    #[error("no delivery client for channel type: {0}")]
    #[allow(dead_code)]
    NoDeliveryClient(String),

    #[error("template rendering failed: {0}")]
    TemplateError(String),

    #[error("delivery failed: {0}")]
    #[allow(dead_code)]
    DeliveryFailed(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct SendNotificationUseCase {
    channel_repo: Arc<dyn NotificationChannelRepository>,
    log_repo: Arc<dyn NotificationLogRepository>,
    template_repo: Option<Arc<dyn NotificationTemplateRepository>>,
    delivery_clients: HashMap<String, Arc<dyn DeliveryClient>>,
    event_publisher: Arc<dyn NotificationEventPublisher>,
}

impl SendNotificationUseCase {
    #[allow(dead_code)]
    pub fn new(
        channel_repo: Arc<dyn NotificationChannelRepository>,
        log_repo: Arc<dyn NotificationLogRepository>,
    ) -> Self {
        Self {
            channel_repo,
            log_repo,
            template_repo: None,
            delivery_clients: HashMap::new(),
            event_publisher: Arc::new(NoopNotificationEventPublisher),
        }
    }

    pub fn with_template_repo(
        channel_repo: Arc<dyn NotificationChannelRepository>,
        log_repo: Arc<dyn NotificationLogRepository>,
        template_repo: Arc<dyn NotificationTemplateRepository>,
    ) -> Self {
        Self {
            channel_repo,
            log_repo,
            template_repo: Some(template_repo),
            delivery_clients: HashMap::new(),
            event_publisher: Arc::new(NoopNotificationEventPublisher),
        }
    }

    #[allow(dead_code)]
    pub fn with_delivery_clients(
        channel_repo: Arc<dyn NotificationChannelRepository>,
        log_repo: Arc<dyn NotificationLogRepository>,
        delivery_clients: HashMap<String, Arc<dyn DeliveryClient>>,
    ) -> Self {
        Self {
            channel_repo,
            log_repo,
            template_repo: None,
            delivery_clients,
            event_publisher: Arc::new(NoopNotificationEventPublisher),
        }
    }

    pub fn with_delivery_clients_and_template_repo(
        channel_repo: Arc<dyn NotificationChannelRepository>,
        log_repo: Arc<dyn NotificationLogRepository>,
        template_repo: Arc<dyn NotificationTemplateRepository>,
        delivery_clients: HashMap<String, Arc<dyn DeliveryClient>>,
    ) -> Self {
        Self {
            channel_repo,
            log_repo,
            template_repo: Some(template_repo),
            delivery_clients,
            event_publisher: Arc::new(NoopNotificationEventPublisher),
        }
    }

    pub fn with_event_publisher(
        mut self,
        event_publisher: Arc<dyn NotificationEventPublisher>,
    ) -> Self {
        self.event_publisher = event_publisher;
        self
    }

    fn render_template(
        template: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, SendNotificationError> {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(false);
        hbs.register_template_string("t", template)
            .map_err(|e| SendNotificationError::TemplateError(e.to_string()))?;
        hbs.render("t", variables)
            .map_err(|e| SendNotificationError::TemplateError(e.to_string()))
    }

    pub async fn execute(
        &self,
        input: &SendNotificationInput,
    ) -> Result<SendNotificationOutput, SendNotificationError> {
        let channel = self
            .channel_repo
            .find_by_id(&input.channel_id)
            .await
            .map_err(|e| SendNotificationError::Internal(e.to_string()))?
            .ok_or_else(|| SendNotificationError::ChannelNotFound(input.channel_id.clone()))?;

        if !channel.enabled {
            return Err(SendNotificationError::ChannelDisabled(
                input.channel_id.clone(),
            ));
        }

        let (resolved_template_id, base_subject, base_body) = if let Some(template_id) =
            &input.template_id
        {
            let repo = self.template_repo.as_ref().ok_or_else(|| {
                SendNotificationError::Internal("template repository is not configured".to_string())
            })?;
            let template = repo
                .find_by_id(template_id)
                .await
                .map_err(|e| SendNotificationError::Internal(e.to_string()))?
                .ok_or_else(|| SendNotificationError::TemplateNotFound(template_id.clone()))?;
            (
                Some(template_id.clone()),
                template.subject_template,
                template.body_template,
            )
        } else {
            (None, input.subject.clone(), input.body.clone())
        };

        // Render templates with variables if provided.
        let (subject, body) = if let Some(ref vars) = input.template_variables {
            let rendered_subject = match &base_subject {
                Some(s) => Some(Self::render_template(s, vars)?),
                None => None,
            };
            let rendered_body = Self::render_template(&base_body, vars)?;
            (rendered_subject, rendered_body)
        } else {
            (base_subject, base_body)
        };

        let mut log = NotificationLog::new(
            input.channel_id.clone(),
            input.recipient.clone(),
            subject.clone(),
            body.clone(),
        );
        log.template_id = resolved_template_id;

        // Attempt delivery if a client is available for this channel type
        if let Some(client) = self.delivery_clients.get(&channel.channel_type) {
            let subject_str = subject.as_deref().unwrap_or("");
            match client.send(&input.recipient, subject_str, &body).await {
                Ok(()) => {
                    log.status = "sent".to_string();
                    log.sent_at = Some(chrono::Utc::now());
                }
                Err(e) => {
                    log.status = "failed".to_string();
                    log.error_message = Some(e.to_string());
                }
            }
        } else if self.delivery_clients.is_empty() {
            // No delivery clients configured at all: mark as sent for backward compatibility
            log.status = "sent".to_string();
            log.sent_at = Some(chrono::Utc::now());
        } else {
            // Delivery clients exist but none for this channel type
            log.status = "failed".to_string();
            log.error_message = Some(format!(
                "no delivery client for channel type: {}",
                channel.channel_type
            ));
        }

        self.log_repo
            .create(&log)
            .await
            .map_err(|e| SendNotificationError::Internal(e.to_string()))?;

        if let Err(e) = self.event_publisher.publish_notification_sent(&log).await {
            tracing::warn!(
                error = %e,
                notification_id = %log.id,
                "failed to publish notification sent event"
            );
        }

        Ok(SendNotificationOutput {
            log_id: log.id.clone(),
            status: log.status,
            created_at: log.created_at,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::notification_channel::NotificationChannel;
    use crate::domain::repository::notification_channel_repository::MockNotificationChannelRepository;
    use crate::domain::repository::notification_log_repository::MockNotificationLogRepository;
    use crate::domain::service::delivery_client::MockDeliveryClient;

    #[tokio::test]
    async fn success() {
        let mut channel_mock = MockNotificationChannelRepository::new();
        let mut log_mock = MockNotificationLogRepository::new();

        let channel = NotificationChannel::new(
            "email".to_string(),
            "email".to_string(),
            serde_json::json!({}),
            true,
        );
        let channel_id = channel.id.clone();
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf({
                let channel_id = channel_id.clone();
                move |id| id == channel_id.as_str()
            })
            .returning(move |_| Ok(Some(return_channel.clone())));

        log_mock.expect_create().returning(|_| Ok(()));

        let uc = SendNotificationUseCase::new(Arc::new(channel_mock), Arc::new(log_mock));
        let input = SendNotificationInput {
            channel_id: channel_id.clone(),
            template_id: None,
            recipient: "user@example.com".to_string(),
            subject: Some("Hello".to_string()),
            body: "Test notification".to_string(),
            template_variables: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.status, "sent");
    }

    #[tokio::test]
    async fn channel_not_found() {
        let mut channel_mock = MockNotificationChannelRepository::new();
        let log_mock = MockNotificationLogRepository::new();
        let missing_id = "ch_missing".to_string();

        channel_mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = SendNotificationUseCase::new(Arc::new(channel_mock), Arc::new(log_mock));
        let input = SendNotificationInput {
            channel_id: missing_id.clone(),
            template_id: None,
            recipient: "user@example.com".to_string(),
            subject: None,
            body: "Test".to_string(),
            template_variables: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SendNotificationError::ChannelNotFound(id) => assert_eq!(id, missing_id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn channel_disabled() {
        let mut channel_mock = MockNotificationChannelRepository::new();
        let log_mock = MockNotificationLogRepository::new();

        let channel = NotificationChannel::new(
            "sms".to_string(),
            "sms".to_string(),
            serde_json::json!({}),
            false,
        );
        let channel_id = channel.id.clone();
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf({
                let channel_id = channel_id.clone();
                move |id| id == channel_id.as_str()
            })
            .returning(move |_| Ok(Some(return_channel.clone())));

        let uc = SendNotificationUseCase::new(Arc::new(channel_mock), Arc::new(log_mock));
        let input = SendNotificationInput {
            channel_id: channel_id.clone(),
            template_id: None,
            recipient: "user@example.com".to_string(),
            subject: None,
            body: "Test".to_string(),
            template_variables: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SendNotificationError::ChannelDisabled(id) => assert_eq!(id, channel_id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn delivery_client_success() {
        let mut channel_mock = MockNotificationChannelRepository::new();
        let mut log_mock = MockNotificationLogRepository::new();

        let channel = NotificationChannel::new(
            "email".to_string(),
            "email".to_string(),
            serde_json::json!({}),
            true,
        );
        let channel_id = channel.id.clone();
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf({
                let channel_id = channel_id.clone();
                move |id| id == channel_id.as_str()
            })
            .returning(move |_| Ok(Some(return_channel.clone())));

        log_mock.expect_create().returning(|_| Ok(()));

        let mut mock_client = MockDeliveryClient::new();
        mock_client.expect_send().returning(|_, _, _| Ok(()));

        let mut clients: HashMap<String, Arc<dyn DeliveryClient>> = HashMap::new();
        clients.insert("email".to_string(), Arc::new(mock_client));

        let uc = SendNotificationUseCase::with_delivery_clients(
            Arc::new(channel_mock),
            Arc::new(log_mock),
            clients,
        );
        let input = SendNotificationInput {
            channel_id: channel_id.clone(),
            template_id: None,
            recipient: "user@example.com".to_string(),
            subject: Some("Hello".to_string()),
            body: "Test".to_string(),
            template_variables: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "sent");
    }

    #[tokio::test]
    async fn delivery_client_failure_records_error() {
        let mut channel_mock = MockNotificationChannelRepository::new();
        let mut log_mock = MockNotificationLogRepository::new();

        let channel = NotificationChannel::new(
            "slack".to_string(),
            "slack".to_string(),
            serde_json::json!({}),
            true,
        );
        let channel_id = channel.id.clone();
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf({
                let channel_id = channel_id.clone();
                move |id| id == channel_id.as_str()
            })
            .returning(move |_| Ok(Some(return_channel.clone())));

        log_mock
            .expect_create()
            .withf(|log: &NotificationLog| log.status == "failed" && log.error_message.is_some())
            .returning(|_| Ok(()));

        let mut mock_client = MockDeliveryClient::new();
        mock_client
            .expect_send()
            .returning(|_, _, _| Err(DeliveryError::ConnectionFailed("timeout".to_string())));

        let mut clients: HashMap<String, Arc<dyn DeliveryClient>> = HashMap::new();
        clients.insert("slack".to_string(), Arc::new(mock_client));

        let uc = SendNotificationUseCase::with_delivery_clients(
            Arc::new(channel_mock),
            Arc::new(log_mock),
            clients,
        );
        let input = SendNotificationInput {
            channel_id: channel_id.clone(),
            template_id: None,
            recipient: "#general".to_string(),
            subject: None,
            body: "Test".to_string(),
            template_variables: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "failed");
    }

    #[tokio::test]
    async fn no_client_for_channel_type() {
        let mut channel_mock = MockNotificationChannelRepository::new();
        let mut log_mock = MockNotificationLogRepository::new();

        let channel = NotificationChannel::new(
            "sms".to_string(),
            "sms".to_string(),
            serde_json::json!({}),
            true,
        );
        let channel_id = channel.id.clone();
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf({
                let channel_id = channel_id.clone();
                move |id| id == channel_id.as_str()
            })
            .returning(move |_| Ok(Some(return_channel.clone())));

        log_mock
            .expect_create()
            .withf(|log: &NotificationLog| log.status == "failed")
            .returning(|_| Ok(()));

        // Clients map has "email" but channel is "sms"
        let mut clients: HashMap<String, Arc<dyn DeliveryClient>> = HashMap::new();
        let mut mock_client = MockDeliveryClient::new();
        mock_client.expect_send().returning(|_, _, _| Ok(()));
        clients.insert("email".to_string(), Arc::new(mock_client));

        let uc = SendNotificationUseCase::with_delivery_clients(
            Arc::new(channel_mock),
            Arc::new(log_mock),
            clients,
        );
        let input = SendNotificationInput {
            channel_id: channel_id.clone(),
            template_id: None,
            recipient: "+1234567890".to_string(),
            subject: None,
            body: "Test".to_string(),
            template_variables: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "failed");
    }

    #[tokio::test]
    async fn template_variable_substitution() {
        let mut channel_mock = MockNotificationChannelRepository::new();
        let mut log_mock = MockNotificationLogRepository::new();

        let channel = NotificationChannel::new(
            "email".to_string(),
            "email".to_string(),
            serde_json::json!({}),
            true,
        );
        let channel_id = channel.id.clone();
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf({
                let channel_id = channel_id.clone();
                move |id| *id == channel_id
            })
            .returning(move |_| Ok(Some(return_channel.clone())));

        log_mock
            .expect_create()
            .withf(|log: &NotificationLog| {
                log.body == "Hello Alice, your order 123 is ready."
                    && log.subject.as_deref() == Some("Order 123 Confirmation")
            })
            .returning(|_| Ok(()));

        let mut mock_client = MockDeliveryClient::new();
        mock_client.expect_send().returning(|_, _, _| Ok(()));

        let mut clients: HashMap<String, Arc<dyn DeliveryClient>> = HashMap::new();
        clients.insert("email".to_string(), Arc::new(mock_client));

        let uc = SendNotificationUseCase::with_delivery_clients(
            Arc::new(channel_mock),
            Arc::new(log_mock),
            clients,
        );

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("order_id".to_string(), "123".to_string());

        let input = SendNotificationInput {
            channel_id,
            template_id: None,
            recipient: "alice@example.com".to_string(),
            subject: Some("Order {{order_id}} Confirmation".to_string()),
            body: "Hello {{name}}, your order {{order_id}} is ready.".to_string(),
            template_variables: Some(vars),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "sent");
    }
}
