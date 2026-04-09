use std::sync::Arc;

use crate::domain::entity::notification_log::NotificationLog;
use crate::domain::repository::NotificationChannelRepository;
use crate::domain::repository::NotificationLogRepository;
use crate::domain::service::NotificationDomainService;

/// MEDIUM-RUST-001 監査対応: `tenant_id` を追加してチャンネル検索時の RLS を有効化する。
#[derive(Debug, Clone)]
pub struct RetryNotificationInput {
    pub notification_id: String,
    /// JWT クレームから取得したテナント ID。チャンネル存在確認の RLS に使用する。
    pub tenant_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RetryNotificationError {
    #[error("notification not found: {0}")]
    NotFound(String),

    #[error("notification already sent: {0}")]
    AlreadySent(String),

    #[error("channel not found: {0}")]
    ChannelNotFound(String),

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

    /// MEDIUM-RUST-001 監査対応: `tenant_id` を input から受け取りチャンネル検索時の RLS を有効化する。
    pub async fn execute(
        &self,
        input: &RetryNotificationInput,
    ) -> Result<NotificationLog, RetryNotificationError> {
        // テナントスコープで通知ログを検索する
        let mut log = self
            .log_repo
            .find_by_id(&input.notification_id, &input.tenant_id)
            .await
            .map_err(|e| RetryNotificationError::Internal(e.to_string()))?
            .ok_or_else(|| RetryNotificationError::NotFound(input.notification_id.clone()))?;

        if !NotificationDomainService::is_retryable_status(&log.status) {
            return Err(RetryNotificationError::AlreadySent(
                input.notification_id.clone(),
            ));
        }

        // MEDIUM-RUST-001 監査対応: チャンネル確認時に tenant_id を伝播して RLS を有効化する
        // 通知ログの channel_id からチャンネルを検索する際、呼び出し元テナントの権限で検索する
        self.channel_repo
            .find_by_id(&log.channel_id, &input.tenant_id)
            .await
            .map_err(|e| RetryNotificationError::Internal(e.to_string()))?
            .ok_or_else(|| RetryNotificationError::ChannelNotFound(log.channel_id.clone()))?;

        // Mark as retried/sent
        log.status = "sent".to_string();
        log.retry_count = log.retry_count.saturating_add(1);
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::notification_channel::NotificationChannel;
    use crate::domain::repository::notification_channel_repository::MockNotificationChannelRepository;
    use crate::domain::repository::notification_log_repository::MockNotificationLogRepository;

    fn failed_log() -> NotificationLog {
        let mut log = NotificationLog::new(
            "tenant_a".to_string(),
            "ch_00000000000000000000000000000000".to_string(),
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
        let log_id = log.id.clone();
        let channel_id = log.channel_id.clone();
        let return_log = log.clone();

        log_mock
            .expect_find_by_id()
            .returning(move |_, _| Ok(Some(return_log.clone())));
        log_mock.expect_update().returning(|_| Ok(()));

        let channel = NotificationChannel::new(
            "email".to_string(),
            "email".to_string(),
            serde_json::json!({}),
            "system".to_string(),
            true,
        );
        let mut return_channel = channel.clone();
        return_channel.id = channel_id.clone();
        channel_mock
            .expect_find_by_id()
            .returning(move |_, _| Ok(Some(return_channel.clone())));

        let uc = RetryNotificationUseCase::new(Arc::new(log_mock), Arc::new(channel_mock));
        let input = RetryNotificationInput {
            notification_id: log_id.clone(),
            tenant_id: "tenant_a".to_string(),
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

        log_mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = RetryNotificationUseCase::new(Arc::new(log_mock), Arc::new(channel_mock));
        let input = RetryNotificationInput {
            notification_id: "notif_missing".to_string(),
            tenant_id: "tenant_a".to_string(),
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
        let log_id = log.id.clone();
        let return_log = log.clone();

        log_mock
            .expect_find_by_id()
            .returning(move |_, _| Ok(Some(return_log.clone())));

        let uc = RetryNotificationUseCase::new(Arc::new(log_mock), Arc::new(channel_mock));
        let input = RetryNotificationInput {
            notification_id: log_id.clone(),
            tenant_id: "tenant_a".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            RetryNotificationError::AlreadySent(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
