use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, error, info};

use super::store::OutboxStore;
use crate::error::OutboxError;
use crate::publisher::EventPublisher;

/// Outbox テーブルからイベントをポーリングして発行するリレー。
pub struct OutboxRelay {
    store: Arc<dyn OutboxStore>,
    publisher: Arc<dyn EventPublisher>,
    poll_interval: Duration,
    batch_size: i64,
    max_retries: i32,
}

impl OutboxRelay {
    /// ビルダーを返す。
    #[must_use]
    pub fn builder(
        store: Arc<dyn OutboxStore>,
        publisher: Arc<dyn EventPublisher>,
    ) -> OutboxRelayBuilder {
        OutboxRelayBuilder {
            store,
            publisher,
            poll_interval: Duration::from_secs(5),
            batch_size: 100,
            max_retries: 3,
        }
    }

    /// リレーを開始する。キャンセルされるまでポーリングを続ける。
    pub async fn run(&self, mut cancel: tokio::sync::oneshot::Receiver<()>) {
        info!(
            poll_interval_ms = self.poll_interval.as_millis() as u64,
            batch_size = self.batch_size,
            "outbox relay started"
        );

        loop {
            tokio::select! {
                _ = &mut cancel => {
                    info!("outbox relay stopping");
                    break;
                }
                _ = tokio::time::sleep(self.poll_interval) => {
                    if let Err(e) = self.process_batch().await {
                        error!(error = %e, "outbox relay batch failed");
                    }
                }
            }
        }
    }

    async fn process_batch(&self) -> Result<(), OutboxError> {
        let entries = self.store.fetch_pending(self.batch_size).await?;
        if entries.is_empty() {
            return Ok(());
        }

        debug!(count = entries.len(), "processing outbox entries");

        for entry in entries {
            let envelope = crate::envelope::EventEnvelope {
                event_type: entry.event_type.clone(),
                metadata: crate::envelope::EventMetadata::new("outbox-relay"),
                payload: entry.payload.clone(),
            };

            match self.publisher.publish(envelope).await {
                Ok(()) => {
                    self.store.mark_published(entry.id).await?;
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    if entry.retry_count >= self.max_retries {
                        error!(
                            entry_id = %entry.id,
                            retries = entry.retry_count,
                            "outbox entry exceeded max retries, marking failed"
                        );
                        self.store.mark_failed(entry.id, &error_msg).await?;
                    } else {
                        self.store.mark_failed(entry.id, &error_msg).await?;
                    }
                }
            }
        }

        Ok(())
    }
}

/// `OutboxRelay` のビルダー。
pub struct OutboxRelayBuilder {
    store: Arc<dyn OutboxStore>,
    publisher: Arc<dyn EventPublisher>,
    poll_interval: Duration,
    batch_size: i64,
    max_retries: i32,
}

impl OutboxRelayBuilder {
    /// ポーリング間隔を設定する。
    #[must_use]
    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// バッチサイズを設定する。
    #[must_use]
    pub fn batch_size(mut self, size: i64) -> Self {
        self.batch_size = size;
        self
    }

    /// 最大リトライ回数を設定する。
    #[must_use]
    pub fn max_retries(mut self, retries: i32) -> Self {
        self.max_retries = retries;
        self
    }

    /// `OutboxRelay` を構築する。
    #[must_use]
    pub fn build(self) -> OutboxRelay {
        OutboxRelay {
            store: self.store,
            publisher: self.publisher,
            poll_interval: self.poll_interval,
            batch_size: self.batch_size,
            max_retries: self.max_retries,
        }
    }
}
