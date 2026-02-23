use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::notification_log::NotificationLog;
use crate::domain::repository::NotificationChannelRepository;
use crate::domain::repository::NotificationLogRepository;

#[derive(Debug, Clone)]
pub struct SendNotificationInput {
    pub channel_id: Uuid,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
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

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct SendNotificationUseCase {
    channel_repo: Arc<dyn NotificationChannelRepository>,
    log_repo: Arc<dyn NotificationLogRepository>,
}

impl SendNotificationUseCase {
    pub fn new(
        channel_repo: Arc<dyn NotificationChannelRepository>,
        log_repo: Arc<dyn NotificationLogRepository>,
    ) -> Self {
        Self {
            channel_repo,
            log_repo,
        }
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

        let mut log = NotificationLog::new(
            input.channel_id,
            input.recipient.clone(),
            input.subject.clone(),
            input.body.clone(),
        );
        log.status = "sent".to_string();
        log.sent_at = Some(chrono::Utc::now());

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
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SendNotificationError::ChannelDisabled(id) => assert_eq!(id, channel_id),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
