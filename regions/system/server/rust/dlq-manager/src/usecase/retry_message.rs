use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::DlqMessage;
use crate::domain::repository::DlqMessageRepository;
use crate::infrastructure::kafka::producer::DlqEventPublisher;

/// RetryMessageUseCase は DLQ メッセージの再処理を担う。
pub struct RetryMessageUseCase {
    repo: Arc<dyn DlqMessageRepository>,
    publisher: Option<Arc<dyn DlqEventPublisher>>,
}

impl RetryMessageUseCase {
    pub fn new(
        repo: Arc<dyn DlqMessageRepository>,
        publisher: Option<Arc<dyn DlqEventPublisher>>,
    ) -> Self {
        Self { repo, publisher }
    }

    /// DLQ メッセージを再処理する。
    pub async fn execute(&self, id: Uuid) -> anyhow::Result<DlqMessage> {
        let mut message = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("dlq message not found: {}", id))?;

        if !message.is_retryable() {
            anyhow::bail!(
                "message is not retryable: status={}, retry_count={}/{}",
                message.status,
                message.retry_count,
                message.max_retries
            );
        }

        message.mark_retrying();

        // Kafka に再発行（producer がある場合）
        if let Some(ref publisher) = self.publisher {
            match publisher
                .publish_to_topic(&message.original_topic, &message.payload)
                .await
            {
                Ok(()) => {
                    tracing::info!(
                        message_id = %message.id,
                        topic = %message.original_topic,
                        "message republished to original topic"
                    );
                    message.mark_resolved();
                }
                Err(e) => {
                    tracing::warn!(
                        message_id = %message.id,
                        error = %e,
                        "failed to republish message, keeping retrying status"
                    );
                }
            }
        } else {
            // Kafka なしの場合は Resolved にする
            message.mark_resolved();
        }

        self.repo.update(&message).await?;
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{DlqMessage, DlqStatus};
    use crate::domain::repository::dlq_message_repository::MockDlqMessageRepository;

    #[tokio::test]
    async fn test_retry_message_success_no_publisher() {
        let msg = DlqMessage::new(
            "orders.events.v1".to_string(),
            "failed".to_string(),
            serde_json::json!({}),
            3,
        );
        let msg_id = msg.id;
        let msg_clone = msg.clone();

        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(msg_clone.clone())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = RetryMessageUseCase::new(Arc::new(mock), None);
        let result = uc.execute(msg_id).await.unwrap();
        assert_eq!(result.status, DlqStatus::Resolved);
    }

    #[tokio::test]
    async fn test_retry_message_not_found() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = RetryMessageUseCase::new(Arc::new(mock), None);
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_retry_message_not_retryable() {
        let mut msg = DlqMessage::new(
            "orders.events.v1".to_string(),
            "failed".to_string(),
            serde_json::json!({}),
            3,
        );
        msg.mark_dead();
        let msg_id = msg.id;
        let msg_clone = msg.clone();

        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(msg_clone.clone())));

        let uc = RetryMessageUseCase::new(Arc::new(mock), None);
        let result = uc.execute(msg_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not retryable"));
    }
}
