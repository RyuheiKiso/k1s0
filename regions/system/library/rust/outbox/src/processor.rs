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

/// デフォルトの並列処理数
const DEFAULT_CONCURRENCY: usize = 4;

/// OutboxProcessor はアウトボックスメッセージの定期処理を担う。
/// fetch_pending → publish → update のサイクルを並列実行する。
pub struct OutboxProcessor {
    store: Arc<dyn OutboxStore>,
    publisher: Arc<dyn OutboxPublisher>,
    /// 1回のポーリングで処理するメッセージ数
    batch_size: u32,
    /// 並列処理数（デフォルト: 4）
    concurrency: usize,
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
            concurrency: DEFAULT_CONCURRENCY,
        }
    }

    /// 並列処理数を設定する。
    #[allow(dead_code)]
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// 1回分のアウトボックス処理を並列実行する。
    /// 処理したメッセージ数を返す。
    pub async fn process_batch(&self) -> Result<u32, OutboxError> {
        let messages = self.store.fetch_pending(self.batch_size).await?;
        if messages.is_empty() {
            return Ok(0);
        }

        let mut processed = 0u32;
        // メッセージを並列度ごとのチャンクに分割して処理する
        for chunk in messages.chunks(self.concurrency) {
            let mut join_set = tokio::task::JoinSet::new();

            for message in chunk.iter().cloned() {
                let store = Arc::clone(&self.store);
                let publisher = Arc::clone(&self.publisher);

                join_set.spawn(async move {
                    let mut message = message;
                    // 処理中ステータスに遷移
                    message.mark_processing();
                    if let Err(e) = store.update(&message).await {
                        return Err(OutboxError::StoreError(e.to_string()));
                    }

                    match publisher.publish(&message).await {
                        Ok(()) => {
                            message.mark_delivered();
                            store.update(&message).await?;
                            Ok(true)
                        }
                        Err(e) => {
                            message.mark_failed(e.to_string());
                            store.update(&message).await?;
                            Ok(false)
                        }
                    }
                });
            }

            // 全タスクの完了を待つ
            while let Some(result) = join_set.join_next().await {
                match result {
                    Ok(Ok(true)) => processed += 1,
                    Ok(Ok(false)) => {} // 発行失敗（リトライ対象）
                    Ok(Err(e)) => {
                        tracing::error!(error = %e, "アウトボックスメッセージ処理中のストアエラー");
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "アウトボックスタスクがパニック");
                    }
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
#[allow(clippy::unwrap_used)]
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
