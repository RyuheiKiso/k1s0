use std::sync::Arc;

use crate::domain::repository::DlqMessageRepository;
use crate::infrastructure::kafka::producer::DlqEventPublisher;

/// RetryAllUseCase はトピック内の全 DLQ メッセージを一括再処理する。
pub struct RetryAllUseCase {
    repo: Arc<dyn DlqMessageRepository>,
    publisher: Option<Arc<dyn DlqEventPublisher>>,
}

impl RetryAllUseCase {
    pub fn new(
        repo: Arc<dyn DlqMessageRepository>,
        publisher: Option<Arc<dyn DlqEventPublisher>>,
    ) -> Self {
        Self { repo, publisher }
    }

    /// トピック内の全メッセージを再処理する。成功件数を返す。
    pub async fn execute(&self, topic: &str) -> anyhow::Result<i64> {
        let mut retried = 0i64;
        let mut page = 1;
        let page_size = 100;

        loop {
            let (messages, _total) = self.repo.find_by_topic(topic, page, page_size).await?;
            if messages.is_empty() {
                break;
            }

            for mut message in messages {
                if !message.is_retryable() {
                    continue;
                }

                message.mark_retrying();

                if let Some(ref publisher) = self.publisher {
                    match publisher
                        .publish_to_topic(&message.original_topic, &message.payload)
                        .await
                    {
                        Ok(()) => {
                            message.mark_resolved();
                        }
                        Err(e) => {
                            tracing::warn!(
                                message_id = %message.id,
                                error = %e,
                                "failed to republish message during retry-all"
                            );
                        }
                    }
                } else {
                    message.mark_resolved();
                }

                self.repo.update(&message).await?;
                retried += 1;
            }

            page += 1;
        }

        tracing::info!(topic = %topic, retried = retried, "retry-all completed");
        Ok(retried)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::DlqMessage;
    use crate::domain::repository::dlq_message_repository::MockDlqMessageRepository;

    #[tokio::test]
    async fn test_retry_all_empty_topic() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_topic()
            .returning(|_, _, _| Ok((vec![], 0)));

        let uc = RetryAllUseCase::new(Arc::new(mock), None);
        let retried = uc.execute("orders.dlq.v1").await.unwrap();
        assert_eq!(retried, 0);
    }

    #[tokio::test]
    async fn test_retry_all_with_messages() {
        let mut mock = MockDlqMessageRepository::new();
        let mut call_count = 0;
        mock.expect_find_by_topic().returning(move |_, _, _| {
            call_count += 1;
            if call_count == 1 {
                let msg = DlqMessage::new(
                    "orders.events.v1".to_string(),
                    "failed".to_string(),
                    serde_json::json!({}),
                    3,
                );
                Ok((vec![msg], 1))
            } else {
                Ok((vec![], 0))
            }
        });
        mock.expect_update().returning(|_| Ok(()));

        let uc = RetryAllUseCase::new(Arc::new(mock), None);
        let retried = uc.execute("orders.dlq.v1").await.unwrap();
        assert_eq!(retried, 1);
    }
}
