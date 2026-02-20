use std::sync::Arc;

use crate::error::OutboxError;
use crate::message::OutboxMessage;
use crate::store::OutboxStore;

/// OutboxPublisher はアウトボックスメッセージの発行インターフェース。
#[async_trait::async_trait]
pub trait OutboxPublisher: Send + Sync {
    async fn publish(&self, message: &OutboxMessage) -> Result<(), OutboxError>;
}

/// OutboxProcessor はアウトボックスメッセージの定期処理を担う。
/// fetch_pending → publish → update のサイクルを実行する。
pub struct OutboxProcessor {
    store: Arc<dyn OutboxStore>,
    publisher: Arc<dyn OutboxPublisher>,
    /// 1回のポーリングで処理するメッセージ数
    batch_size: u32,
}

impl OutboxProcessor {
    pub fn new(
        store: Arc<dyn OutboxStore>,
        publisher: Arc<dyn OutboxPublisher>,
        batch_size: u32,
    ) -> Self {
        Self {
            store,
            publisher,
            batch_size,
        }
    }

    /// 1回分のアウトボックス処理を実行する。
    /// 処理したメッセージ数を返す。
    pub async fn process_batch(&self) -> Result<u32, OutboxError> {
        let messages = self.store.fetch_pending(self.batch_size).await?;
        let mut processed = 0u32;

        for mut message in messages {
            message.mark_processing();
            self.store.update(&message).await?;

            match self.publisher.publish(&message).await {
                Ok(()) => {
                    message.mark_delivered();
                    self.store.update(&message).await?;
                    processed += 1;
                }
                Err(e) => {
                    message.mark_failed(e.to_string());
                    self.store.update(&message).await?;
                }
            }
        }

        Ok(processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MockOutboxStore;

    struct AlwaysSuccessPublisher;

    #[async_trait::async_trait]
    impl OutboxPublisher for AlwaysSuccessPublisher {
        async fn publish(&self, _message: &OutboxMessage) -> Result<(), OutboxError> {
            Ok(())
        }
    }

    struct AlwaysFailPublisher;

    #[async_trait::async_trait]
    impl OutboxPublisher for AlwaysFailPublisher {
        async fn publish(&self, _message: &OutboxMessage) -> Result<(), OutboxError> {
            Err(OutboxError::PublishError("kafka unavailable".to_string()))
        }
    }

    #[tokio::test]
    async fn test_process_batch_empty() {
        let mut store = MockOutboxStore::new();
        store.expect_fetch_pending().returning(|_| Ok(vec![]));

        let processor = OutboxProcessor::new(
            Arc::new(store),
            Arc::new(AlwaysSuccessPublisher),
            10,
        );
        let count = processor.process_batch().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_process_batch_success() {
        let msg = OutboxMessage::new(
            "k1s0.test.topic.v1",
            "key-1",
            serde_json::json!({"test": true}),
        );
        let msg_clone = msg.clone();

        let mut store = MockOutboxStore::new();
        store
            .expect_fetch_pending()
            .returning(move |_| Ok(vec![msg_clone.clone()]));
        store
            .expect_update()
            .times(2) // processing + delivered
            .returning(|_| Ok(()));

        let processor = OutboxProcessor::new(
            Arc::new(store),
            Arc::new(AlwaysSuccessPublisher),
            10,
        );
        let count = processor.process_batch().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_process_batch_publish_failure() {
        let msg = OutboxMessage::new(
            "k1s0.test.topic.v1",
            "key-1",
            serde_json::json!({"test": true}),
        );
        let msg_clone = msg.clone();

        let mut store = MockOutboxStore::new();
        store
            .expect_fetch_pending()
            .returning(move |_| Ok(vec![msg_clone.clone()]));
        store
            .expect_update()
            .times(2) // processing + failed
            .returning(|_| Ok(()));

        let processor = OutboxProcessor::new(
            Arc::new(store),
            Arc::new(AlwaysFailPublisher),
            10,
        );
        let count = processor.process_batch().await.unwrap();
        assert_eq!(count, 0); // 失敗したので 0
    }
}
