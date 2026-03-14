use std::sync::Arc;
use std::time::Duration;

use crate::error::OutboxError;
use crate::message::OutboxMessage;
use crate::store::OutboxStore;
use tokio_util::sync::CancellationToken;

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

    /// process_batch を interval ごとに実行する。
    /// cancellation_token がキャンセルされたら終了する。
    pub async fn run(
        &self,
        interval: Duration,
        cancellation_token: CancellationToken,
    ) -> Result<(), OutboxError> {
        let mut ticker = tokio::time::interval(interval);
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => break,
                _ = ticker.tick() => {
                    self.process_batch().await?;
                }
            }
        }
        Ok(())
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

    // 処理待ちメッセージが空の場合に process_batch が 0 を返すことを確認する。
    #[tokio::test]
    async fn test_process_batch_empty() {
        let mut store = MockOutboxStore::new();
        store.expect_fetch_pending().returning(|_| Ok(vec![]));

        let processor = OutboxProcessor::new(Arc::new(store), Arc::new(AlwaysSuccessPublisher), 10);
        let count = processor.process_batch().await.unwrap();
        assert_eq!(count, 0);
    }

    // メッセージが正常に発行された場合に process_batch が 1 を返しストアが 2 回更新されることを確認する。
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

        let processor = OutboxProcessor::new(Arc::new(store), Arc::new(AlwaysSuccessPublisher), 10);
        let count = processor.process_batch().await.unwrap();
        assert_eq!(count, 1);
    }

    // パブリッシュが失敗した場合に process_batch が 0 を返しストアが 2 回更新されることを確認する。
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

        let processor = OutboxProcessor::new(Arc::new(store), Arc::new(AlwaysFailPublisher), 10);
        let count = processor.process_batch().await.unwrap();
        assert_eq!(count, 0); // 失敗したので 0
    }

    // キャンセルトークンがキャンセルされると run が正常終了することを確認する。
    #[tokio::test]
    async fn test_run_stops_when_cancelled() {
        let mut store = MockOutboxStore::new();
        store
            .expect_fetch_pending()
            .times(0..)
            .returning(|_| Ok(vec![]));

        let processor = OutboxProcessor::new(Arc::new(store), Arc::new(AlwaysSuccessPublisher), 10);
        let token = CancellationToken::new();
        token.cancel();

        let result = processor.run(Duration::from_millis(10), token).await;
        assert!(result.is_ok());
    }
}
