use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::notification_log::NotificationLog;
use crate::domain::repository::NotificationChannelRepository;
use crate::domain::repository::NotificationLogRepository;

#[derive(Debug, Clone)]
pub struct RetryNotificationInput {
    pub notification_id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum RetryNotificationError {
    #[error("notification not found: {0}")]
    NotFound(Uuid),

    #[error("notification already sent: {0}")]
    AlreadySent(Uuid),

    #[error("channel not found: {0}")]
    ChannelNotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct RetryNotificationUseCase {
    log_repo: Arc<dyn NotificationLogRepository>,
    channel_repo: Arc<dyn NotificationChannelRepository>,
}

impl RetryNotificationUseCase {
    pub fn new(
        log_repo: Arc<dyn NotificationLogRepository>,
        channel_repo: Arc<dyn NotificationChannelRepository>,
    ) -> Self {
        Self {
            log_repo,
            channel_repo,
        }
    }

    pub async fn execute(
        &self,
        input: &RetryNotificationInput,
    ) -> Result<NotificationLog, RetryNotificationError> {
        let mut log = self
            .log_repo
            .find_by_id(&input.notification_id)
            .await
            .map_err(|e| RetryNotificationError::Internal(e.to_string()))?
            .ok_or(RetryNotificationError::NotFound(input.notification_id))?;

        if log.status == "sent" {
            return Err(RetryNotificationError::AlreadySent(input.notification_id));
        }

        // Verify channel still exists
        self.channel_repo
            .find_by_id(&log.channel_id)
            .await
            .map_err(|e| RetryNotificationError::Internal(e.to_string()))?
            .ok_or(RetryNotificationError::ChannelNotFound(log.channel_id))?;

        // Mark as retried/sent
        log.status = "sent".to_string();
        log.sent_at = Some(chrono::Utc::now());
        log.error_message = None;

        self.log_repo
            .update(&log)
            .await
            .map_err(|e| RetryNotificationError::Internal(e.to_string()))?;

        Ok(log)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::notification_channel::NotificationChannel;
    use crate::domain::repository::notification_channel_repository::MockNotificationChannelRepository;
    use crate::domain::repository::notification_log_repository::MockNotificationLogRepository;

    fn failed_log() -> NotificationLog {
        let mut log = NotificationLog::new(
            Uuid::new_v4(),
            "user@example.com".to_string(),
            Some("Hello".to_string()),
            "Body".to_string(),
        );
        log.status = "failed".to_string();
        log.error_message = Some("timeout".to_string());
        log
    }

    #[tokio::test]
    async fn success() {
        let mut log_mock = MockNotificationLogRepository::new();
        let mut channel_mock = MockNotificationChannelRepository::new();

        let log = failed_log();
        let log_id = log.id;
        let channel_id = log.channel_id;
        let return_log = log.clone();

        log_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(return_log.clone())));
        log_mock.expect_update().returning(|_| Ok(()));

        let channel = NotificationChannel::new(
            "email".to_string(),
            "email".to_string(),
            serde_json::json!({}),
            true,
        );
        let mut return_channel = channel.clone();
        return_channel.id = channel_id;
        channel_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(return_channel.clone())));

        let uc = RetryNotificationUseCase::new(Arc::new(log_mock), Arc::new(channel_mock));
        let input = RetryNotificationInput {
            notification_id: log_id,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let retried = result.unwrap();
        assert_eq!(retried.status, "sent");
        assert!(retried.error_message.is_none());
    }

    #[tokio::test]
    async fn not_found() {
        let mut log_mock = MockNotificationLogRepository::new();
        let channel_mock = MockNotificationChannelRepository::new();

        log_mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = RetryNotificationUseCase::new(Arc::new(log_mock), Arc::new(channel_mock));
        let input = RetryNotificationInput {
            notification_id: Uuid::new_v4(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            RetryNotificationError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn already_sent() {
        let mut log_mock = MockNotificationLogRepository::new();
        let channel_mock = MockNotificationChannelRepository::new();

        let mut log = failed_log();
        log.status = "sent".to_string();
        let log_id = log.id;
        let return_log = log.clone();

        log_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(return_log.clone())));

        let uc = RetryNotificationUseCase::new(Arc::new(log_mock), Arc::new(channel_mock));
        let input = RetryNotificationInput {
            notification_id: log_id,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            RetryNotificationError::AlreadySent(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
