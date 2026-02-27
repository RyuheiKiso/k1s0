use std::collections::HashMap;
use std::sync::Arc;

use handlebars::Handlebars;
use uuid::Uuid;

use crate::domain::entity::notification_log::NotificationLog;
use crate::domain::repository::NotificationChannelRepository;
use crate::domain::repository::NotificationLogRepository;
use crate::domain::service::DeliveryClient;
#[cfg(test)]
use crate::domain::service::DeliveryError;

#[derive(Debug, Clone)]
pub struct SendNotificationInput {
    pub channel_id: Uuid,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
    pub template_variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct SendNotificationOutput {
    pub log_id: Uuid,
    pub status: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SendNotificationError {
    #[error("channel not found: {0}")]
    ChannelNotFound(Uuid),

    #[error("channel disabled: {0}")]
    ChannelDisabled(Uuid),

    #[error("no delivery client for channel type: {0}")]
    NoDeliveryClient(String),

    #[error("template rendering failed: {0}")]
    TemplateError(String),

    #[error("delivery failed: {0}")]
    DeliveryFailed(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct SendNotificationUseCase {
    channel_repo: Arc<dyn NotificationChannelRepository>,
    log_repo: Arc<dyn NotificationLogRepository>,
    delivery_clients: HashMap<String, Arc<dyn DeliveryClient>>,
}

impl SendNotificationUseCase {
    pub fn new(
        channel_repo: Arc<dyn NotificationChannelRepository>,
        log_repo: Arc<dyn NotificationLogRepository>,
    ) -> Self {
        Self {
            channel_repo,
            log_repo,
            delivery_clients: HashMap::new(),
        }
    }

    pub fn with_delivery_clients(
        channel_repo: Arc<dyn NotificationChannelRepository>,
        log_repo: Arc<dyn NotificationLogRepository>,
        delivery_clients: HashMap<String, Arc<dyn DeliveryClient>>,
    ) -> Self {
        Self {
            channel_repo,
            log_repo,
            delivery_clients,
        }
    }

    fn render_template(template: &str, variables: &HashMap<String, String>) -> Result<String, SendNotificationError> {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(false);
        hbs.register_template_string("t", template)
            .map_err(|e| SendNotificationError::TemplateError(e.to_string()))?;
        hbs.render("t", variables)
            .map_err(|e| SendNotificationError::TemplateError(e.to_string()))
    }

    pub async fn execute(&self, input: &SendNotificationInput) -> Result<SendNotificationOutput, SendNotificationError> {
        let channel = self
            .channel_repo
            .find_by_id(&input.channel_id)
            .await
            .map_err(|e| SendNotificationError::Internal(e.to_string()))?
            .ok_or(SendNotificationError::ChannelNotFound(input.channel_id))?;

        if !channel.enabled {
            return Err(SendNotificationError::ChannelDisabled(input.channel_id));
        }

        // Render templates with variables if provided
        let (subject, body) = if let Some(ref vars) = input.template_variables {
            let rendered_subject = match &input.subject {
                Some(s) => Some(Self::render_template(s, vars)?),
                None => None,
            };
            let rendered_body = Self::render_template(&input.body, vars)?;
            (rendered_subject, rendered_body)
        } else {
            (input.subject.clone(), input.body.clone())
        };

        let mut log = NotificationLog::new(
            input.channel_id,
            input.recipient.clone(),
            subject.clone(),
            body.clone(),
        );

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

        Ok(SendNotificationOutput {
            log_id: log.id,
            status: log.status,
        })
    }
}

#[cfg(test)]
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
        let channel_id = channel.id;
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf(move |id| *id == channel_id)
            .returning(move |_| Ok(Some(return_channel.clone())));

        log_mock.expect_create().returning(|_| Ok(()));

        let uc = SendNotificationUseCase::new(Arc::new(channel_mock), Arc::new(log_mock));
        let input = SendNotificationInput {
            channel_id,
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
        let missing_id = Uuid::new_v4();

        channel_mock
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let uc = SendNotificationUseCase::new(Arc::new(channel_mock), Arc::new(log_mock));
        let input = SendNotificationInput {
            channel_id: missing_id,
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
        let channel_id = channel.id;
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf(move |id| *id == channel_id)
            .returning(move |_| Ok(Some(return_channel.clone())));

        let uc = SendNotificationUseCase::new(Arc::new(channel_mock), Arc::new(log_mock));
        let input = SendNotificationInput {
            channel_id,
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
        let channel_id = channel.id;
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf(move |id| *id == channel_id)
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
            channel_id,
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
        let channel_id = channel.id;
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf(move |id| *id == channel_id)
            .returning(move |_| Ok(Some(return_channel.clone())));

        log_mock
            .expect_create()
            .withf(|log: &NotificationLog| {
                log.status == "failed" && log.error_message.is_some()
            })
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
            channel_id,
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
        let channel_id = channel.id;
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf(move |id| *id == channel_id)
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
            channel_id,
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
        let channel_id = channel.id;
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf(move |id| *id == channel_id)
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
