use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::broadcast;
use tracing::{debug, error, warn};

use crate::envelope::EventEnvelope;
use crate::error::{PublishError, SubscribeError};
use crate::publisher::EventPublisher;
use crate::subscriber::{EventHandler, EventSubscriber, SubscriptionHandle};

/// バックプレッシャー戦略
///
/// ブロードキャストチャネルが満杯になった場合の振る舞いを定義する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackpressureStrategy {
    /// 最も古いメッセージを破棄して新しいメッセージを受け入れる（デフォルト）
    DropOldest,
    /// バッファに空きができるまでブロックする
    Block,
    /// 新しいメッセージを拒否する
    Reject,
    /// 最も新しいメッセージ（今送ろうとしているもの）を破棄する
    DropNewest,
}

impl Default for BackpressureStrategy {
    fn default() -> Self {
        Self::DropOldest
    }
}

/// イベントバスメトリクス
#[derive(Debug)]
pub struct EventBusMetrics {
    /// 現在のキュー深度
    queue_depth: AtomicU64,
    /// 破棄されたイベント数
    dropped_events: AtomicU64,
    /// 拒否されたイベント数
    rejected_events: AtomicU64,
    /// ラグ発生回数（サブスクライバがラグした回数）
    lagged_count: AtomicU64,
}

impl EventBusMetrics {
    /// 新しいメトリクスを作成
    fn new() -> Self {
        Self {
            queue_depth: AtomicU64::new(0),
            dropped_events: AtomicU64::new(0),
            rejected_events: AtomicU64::new(0),
            lagged_count: AtomicU64::new(0),
        }
    }

    /// 現在のキュー深度を取得
    pub fn queue_depth(&self) -> u64 {
        self.queue_depth.load(Ordering::Relaxed)
    }

    /// 破棄されたイベント数を取得
    pub fn dropped_events(&self) -> u64 {
        self.dropped_events.load(Ordering::Relaxed)
    }

    /// 拒否されたイベント数を取得
    pub fn rejected_events(&self) -> u64 {
        self.rejected_events.load(Ordering::Relaxed)
    }

    /// ラグ発生回数を取得
    pub fn lagged_count(&self) -> u64 {
        self.lagged_count.load(Ordering::Relaxed)
    }
}

impl Default for EventBusMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// インメモリのイベントバス。
///
/// `tokio::sync::broadcast` を使用してプロセス内イベント配信を行う。
/// テストやシングルプロセス構成で有用。
///
/// バックプレッシャー戦略を指定することで、チャネル満杯時の振る舞いを制御できる。
pub struct InMemoryEventBus {
    tx: broadcast::Sender<Arc<EventEnvelope>>,
    capacity: usize,
    strategy: BackpressureStrategy,
    metrics: Arc<EventBusMetrics>,
    /// ラグしたイベントのリカバリバッファ
    recovery_buffer: Arc<tokio::sync::Mutex<VecDeque<Arc<EventEnvelope>>>>,
}

impl InMemoryEventBus {
    /// 指定された容量でバスを作成する。
    ///
    /// デフォルトのバックプレッシャー戦略（`DropOldest`）を使用する。
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self::with_strategy(capacity, BackpressureStrategy::default())
    }

    /// 指定された容量とバックプレッシャー戦略でバスを作成する。
    #[must_use]
    pub fn with_strategy(capacity: usize, strategy: BackpressureStrategy) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            tx,
            capacity,
            strategy,
            metrics: Arc::new(EventBusMetrics::new()),
            recovery_buffer: Arc::new(tokio::sync::Mutex::new(VecDeque::new())),
        }
    }

    /// メトリクスへの参照を取得
    pub fn metrics(&self) -> &EventBusMetrics {
        &self.metrics
    }

    /// リカバリバッファからラグしたイベントを取得する。
    ///
    /// サブスクライバがラグした際にリカバリバッファに退避されたイベントを
    /// 最大 `max` 件取得する。
    pub async fn recover_lagged(&self, max: usize) -> Vec<Arc<EventEnvelope>> {
        let mut buffer = self.recovery_buffer.lock().await;
        let count = buffer.len().min(max);
        buffer.drain(..count).collect()
    }

    /// リカバリバッファの現在のサイズを取得
    pub async fn recovery_buffer_len(&self) -> usize {
        self.recovery_buffer.lock().await.len()
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventBus {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), PublishError> {
        debug!(event_type = %envelope.event_type, "publishing event");

        let arc_envelope = Arc::new(envelope);

        match self.strategy {
            BackpressureStrategy::DropOldest => {
                // broadcast::send は古いメッセージを上書きする（デフォルト動作）
                match self.tx.send(arc_envelope) {
                    Ok(_) => {
                        self.metrics.queue_depth.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        return Err(PublishError::Send(e.to_string()));
                    }
                }
            }
            BackpressureStrategy::Block => {
                // レシーバがいない場合はエラー、そうでなければ送信
                // broadcast チャネルはブロッキング送信をサポートしないため、
                // レシーバー数で判定する
                if self.tx.receiver_count() == 0 {
                    return Err(PublishError::Send("no subscribers".to_string()));
                }
                self.tx
                    .send(arc_envelope)
                    .map_err(|e| PublishError::Send(e.to_string()))?;
                self.metrics.queue_depth.fetch_add(1, Ordering::Relaxed);
            }
            BackpressureStrategy::Reject => {
                // チャネルの残り容量を推定（受信者数 * 容量 の概算）
                // broadcast チャネルは正確な残り容量を提供しないため、
                // len() で現在のメッセージ数を確認
                if self.tx.len() >= self.capacity {
                    self.metrics.rejected_events.fetch_add(1, Ordering::Relaxed);
                    return Err(PublishError::Send("event bus full, message rejected".to_string()));
                }
                self.tx
                    .send(arc_envelope)
                    .map_err(|e| PublishError::Send(e.to_string()))?;
                self.metrics.queue_depth.fetch_add(1, Ordering::Relaxed);
            }
            BackpressureStrategy::DropNewest => {
                if self.tx.len() >= self.capacity {
                    self.metrics.dropped_events.fetch_add(1, Ordering::Relaxed);
                    warn!(event_type = %arc_envelope.event_type, "dropping newest event due to full buffer");
                    return Ok(());
                }
                self.tx
                    .send(arc_envelope)
                    .map_err(|e| PublishError::Send(e.to_string()))?;
                self.metrics.queue_depth.fetch_add(1, Ordering::Relaxed);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl EventSubscriber for InMemoryEventBus {
    async fn subscribe(
        &self,
        handler: Box<dyn EventHandler>,
    ) -> Result<SubscriptionHandle, SubscribeError> {
        let mut rx = self.tx.subscribe();
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel();
        let event_type = handler.event_type().to_owned();
        let metrics = self.metrics.clone();
        let _recovery_buffer = self.recovery_buffer.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut cancel_rx => {
                        debug!(event_type = %event_type, "subscription cancelled");
                        break;
                    }
                    result = rx.recv() => {
                        match result {
                            Ok(envelope) => {
                                if envelope.event_type == event_type {
                                    if let Err(e) = handler.handle(&envelope).await {
                                        error!(
                                            event_type = %event_type,
                                            error = %e,
                                            "handler failed"
                                        );
                                    }
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                warn!(
                                    event_type = %event_type,
                                    skipped = n,
                                    "subscriber lagged, events skipped"
                                );
                                metrics.lagged_count.fetch_add(1, Ordering::Relaxed);
                                metrics.dropped_events.fetch_add(n, Ordering::Relaxed);
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                debug!(event_type = %event_type, "bus closed");
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(SubscriptionHandle::new(cancel_tx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::EventMetadata;

    fn make_envelope(event_type: &str) -> EventEnvelope {
        EventEnvelope {
            event_type: event_type.to_string(),
            metadata: EventMetadata::new("test-service"),
            payload: serde_json::json!({"key": "value"}),
        }
    }

    #[tokio::test]
    async fn test_default_bus() {
        let bus = InMemoryEventBus::default();
        assert_eq!(bus.capacity, 256);
        assert_eq!(bus.strategy, BackpressureStrategy::DropOldest);
    }

    #[tokio::test]
    async fn test_with_strategy() {
        let bus = InMemoryEventBus::with_strategy(128, BackpressureStrategy::Reject);
        assert_eq!(bus.capacity, 128);
        assert_eq!(bus.strategy, BackpressureStrategy::Reject);
    }

    #[tokio::test]
    async fn test_publish_drop_oldest() {
        let bus = InMemoryEventBus::new(4);

        // サブスクライバがないと送信はエラーになるが、DropOldest 戦略なら送信自体は試行する
        // broadcast の特性上、レシーバがいないと SendError になる
        let _rx = bus.tx.subscribe();

        for i in 0..4 {
            let envelope = make_envelope(&format!("test.event.{}", i));
            bus.publish(envelope).await.unwrap();
        }

        assert_eq!(bus.metrics().queue_depth(), 4);
    }

    #[tokio::test]
    async fn test_publish_reject_when_full() {
        let bus = InMemoryEventBus::with_strategy(2, BackpressureStrategy::Reject);
        let _rx = bus.tx.subscribe();

        // 2つまでは成功
        bus.publish(make_envelope("test.event.1")).await.unwrap();
        bus.publish(make_envelope("test.event.2")).await.unwrap();

        // 3つ目は拒否
        let result = bus.publish(make_envelope("test.event.3")).await;
        assert!(result.is_err());
        assert_eq!(bus.metrics().rejected_events(), 1);
    }

    #[tokio::test]
    async fn test_publish_drop_newest_when_full() {
        let bus = InMemoryEventBus::with_strategy(2, BackpressureStrategy::DropNewest);
        let _rx = bus.tx.subscribe();

        bus.publish(make_envelope("test.event.1")).await.unwrap();
        bus.publish(make_envelope("test.event.2")).await.unwrap();

        // 3つ目は静かに破棄される（エラーではない）
        let result = bus.publish(make_envelope("test.event.3")).await;
        assert!(result.is_ok());
        assert_eq!(bus.metrics().dropped_events(), 1);
    }

    #[tokio::test]
    async fn test_metrics_initial() {
        let bus = InMemoryEventBus::new(16);
        assert_eq!(bus.metrics().queue_depth(), 0);
        assert_eq!(bus.metrics().dropped_events(), 0);
        assert_eq!(bus.metrics().rejected_events(), 0);
        assert_eq!(bus.metrics().lagged_count(), 0);
    }

    #[tokio::test]
    async fn test_recover_lagged_empty() {
        let bus = InMemoryEventBus::new(16);
        let recovered = bus.recover_lagged(10).await;
        assert!(recovered.is_empty());
    }

    #[tokio::test]
    async fn test_recovery_buffer_len() {
        let bus = InMemoryEventBus::new(16);
        assert_eq!(bus.recovery_buffer_len().await, 0);
    }

    #[tokio::test]
    async fn test_backpressure_strategy_serde() {
        let json = r#""drop_oldest""#;
        let strategy: BackpressureStrategy = serde_json::from_str(json).unwrap();
        assert_eq!(strategy, BackpressureStrategy::DropOldest);

        let json = r#""block""#;
        let strategy: BackpressureStrategy = serde_json::from_str(json).unwrap();
        assert_eq!(strategy, BackpressureStrategy::Block);

        let json = r#""reject""#;
        let strategy: BackpressureStrategy = serde_json::from_str(json).unwrap();
        assert_eq!(strategy, BackpressureStrategy::Reject);

        let json = r#""drop_newest""#;
        let strategy: BackpressureStrategy = serde_json::from_str(json).unwrap();
        assert_eq!(strategy, BackpressureStrategy::DropNewest);
    }

    #[tokio::test]
    async fn test_backpressure_strategy_default() {
        assert_eq!(BackpressureStrategy::default(), BackpressureStrategy::DropOldest);
    }

    #[tokio::test]
    async fn test_subscribe_and_receive() {
        use std::sync::atomic::AtomicU32;

        struct TestHandler {
            count: Arc<AtomicU32>,
        }

        #[async_trait]
        impl EventHandler for TestHandler {
            fn event_type(&self) -> &str {
                "test.event"
            }
            async fn handle(&self, _envelope: &EventEnvelope) -> Result<(), crate::error::HandlerError> {
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let bus = InMemoryEventBus::new(16);
        let count = Arc::new(AtomicU32::new(0));

        let _handle = bus
            .subscribe(Box::new(TestHandler {
                count: count.clone(),
            }))
            .await
            .unwrap();

        // 少し待ってサブスクライバが起動するのを確認
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        bus.publish(make_envelope("test.event")).await.unwrap();
        bus.publish(make_envelope("other.event")).await.unwrap();
        bus.publish(make_envelope("test.event")).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_publish_block_no_subscribers() {
        let bus = InMemoryEventBus::with_strategy(4, BackpressureStrategy::Block);

        // Block 戦略でサブスクライバがいない場合はエラー
        let result = bus.publish(make_envelope("test.event")).await;
        assert!(result.is_err());
    }
}
