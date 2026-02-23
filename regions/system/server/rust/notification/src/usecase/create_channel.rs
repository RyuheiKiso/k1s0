use std::sync::Arc;

use crate::domain::entity::notification_channel::NotificationChannel;
use crate::domain::repository::NotificationChannelRepository;

#[derive(Debug, Clone)]
pub struct CreateChannelInput {
    pub name: String,
    pub channel_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateChannelError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateChannelUseCase {
    repo: Arc<dyn NotificationChannelRepository>,
}

impl CreateChannelUseCase {
    pub fn new(repo: Arc<dyn NotificationChannelRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &CreateChannelInput) -> Result<NotificationChannel, CreateChannelError> {
        let channel = NotificationChannel::new(
            input.name.clone(),
            input.channel_type.clone(),
            input.config.clone(),
            input.enabled,
        );

        self.repo
            .create(&channel)
            .await
            .map_err(|e| CreateChannelError::Internal(e.to_string()))?;

        Ok(channel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::notification_channel_repository::MockNotificationChannelRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreateChannelUseCase::new(Arc::new(mock));
        let input = CreateChannelInput {
            name: "email-channel".to_string(),
            channel_type: "email".to_string(),
            config: serde_json::json!({"smtp_host": "localhost"}),
            enabled: true,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let channel = result.unwrap();
        assert_eq!(channel.name, "email-channel");
        assert_eq!(channel.channel_type, "email");
        assert!(channel.enabled);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_create()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = CreateChannelUseCase::new(Arc::new(mock));
        let input = CreateChannelInput {
            name: "sms-channel".to_string(),
            channel_type: "sms".to_string(),
            config: serde_json::json!({}),
            enabled: true,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateChannelError::Internal(msg) => assert!(msg.contains("db error")),
        }
    }
}
