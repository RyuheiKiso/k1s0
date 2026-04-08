use std::sync::Arc;
use std::time::Duration;

use crate::error::OutboxError;
use crate::message::OutboxMessage;
use crate::store::OutboxStore;
use tokio_util::sync::CancellationToken;

/// `OutboxPublisher` はアウトボックスメッセージの発行インターフェース。
#[async_trait::async_trait]
pub trait OutboxPublisher: Send + Sync {
    async fn publish(&self, message: &OutboxMessage) -> Result<(), OutboxError>;
}

/// デフォルトの並列処理数
const DEFAULT_CONCURRENCY: usize = 4;

/// PROCESSING 状態のスタックを検出するデフォルト閾値（分）
/// この時間を超えて PROCESSING のままのメッセージをリカバリ対象とする（M-12 監査対応）
const DEFAULT_STALE_PROCESSING_MINUTES: u32 = 10;

/// `OutboxProcessor` はアウトボックスメッセージの定期処理を担う。
/// `fetch_pending` → publish → update のサイクルを並列実行する。
/// M-12 監査対応: PROCESSING スタックを防ぐためリカバリ処理も定期実行する。
pub struct OutboxProcessor {
    store: Arc<dyn OutboxStore>,
    publisher: Arc<dyn OutboxPublisher>,
    /// 1回のポーリングで処理するメッセージ数
    batch_size: u32,
    /// 並列処理数（デフォルト: 4）
    concurrency: usize,
    /// PROCESSING スタック検出閾値（分）。この分数を超えた PROCESSING メッセージを PENDING に戻す（M-12 監査対応）
    stale_processing_minutes: u32,
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
            stale_processing_minutes: DEFAULT_STALE_PROCESSING_MINUTES,
        }
    }

    /// 並列処理数を設定する。
    #[allow(dead_code)]
    #[must_use] 
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// PROCESSING スタック検出閾値を設定する（M-12 監査対応）。
    /// デフォルトは `DEFAULT_STALE_PROCESSING_MINUTES` 分。
    #[allow(dead_code)]
    #[must_use] 
    pub fn with_stale_processing_minutes(mut self, minutes: u32) -> Self {
        self.stale_processing_minutes = minutes;
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
                            // M-12 監査対応: mark_delivered 後の DB Update 失敗を安全に処理する。
                            // ? で即時伝播すると PROCESSING 状態が永続化される（Dead Silence）ため、
                            // エラーをログに記録し Ok(false) を返して次のリカバリサイクルに委ねる。
                            // recover_stale_processing が定期的に PROCESSING スタックを PENDING に戻す。
                            match store.update(&message).await {
                                Ok(()) => Ok(true),
                                Err(e) => {
                                    tracing::error!(
                                        error = %e,
                                        message_id = %message.id,
                                        "mark_delivered 後の DB Update 失敗: メッセージが PROCESSING 状態でスタックする可能性あり。リカバリサイクルで自動復旧される"
                                    );
                                    Ok(false)
                                }
                            }
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

    /// PROCESSING 状態でスタックしたメッセージを PENDING に戻すリカバリ処理（M-12 監査対応）。
    /// `mark_delivered` 後の DB Update 失敗によって PROCESSING のまま残ったメッセージを自動復旧する。
    pub async fn recover_stale_messages(&self) -> Result<(), OutboxError> {
        match self
            .store
            .recover_stale_processing(self.stale_processing_minutes)
            .await
        {
            Ok(count) if count > 0 => {
                tracing::warn!(
                    count = count,
                    stale_minutes = self.stale_processing_minutes,
                    "PROCESSING スタックメッセージをリカバリしました（M-12）"
                );
            }
            Ok(_) => {}
            Err(e) => {
                tracing::error!(error = %e, "PROCESSING スタックメッセージのリカバリに失敗しました");
            }
        }
        Ok(())
    }

    /// `process_batch` を interval ごとに実行する。
    /// M-12 監査対応: リカバリ処理も定期実行し PROCESSING スタックを自動解消する。
    /// `cancellation_token` がキャンセルされたら終了する。
    pub async fn run(
        &self,
        interval: Duration,
        cancellation_token: CancellationToken,
    ) -> Result<(), OutboxError> {
        let mut ticker = tokio::time::interval(interval);
        // リカバリは処理インターバルの10倍周期で実行する（過剰なDB負荷を避けるため）
        let mut recovery_tick_count: u64 = 0;
        const RECOVERY_INTERVAL_TICKS: u64 = 10;
        loop {
            tokio::select! {
                () = cancellation_token.cancelled() => break,
                _ = ticker.tick() => {
                    self.process_batch().await?;
                    recovery_tick_count += 1;
                    // H-02 監査対応: % 演算による手動実装を is_multiple_of() に変更（clippy::manual_is_multiple_of 対応）
                    // 一定周期ごとに PROCESSING スタックメッセージをリカバリする（M-12 監査対応）
                    if recovery_tick_count.is_multiple_of(RECOVERY_INTERVAL_TICKS) {
                        self.recover_stale_messages().await?;
                    }
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
