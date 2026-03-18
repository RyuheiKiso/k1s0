use crate::domain::repository::order_repository::OrderRepository;
use crate::proto::k1s0::event::service::order::v1::{
    OrderCancelledEvent, OrderCreatedEvent, OrderItem, OrderUpdatedEvent,
};
use crate::proto::k1s0::system::common::v1::EventMetadata;
use crate::usecase::event_publisher::OrderEventPublisher;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

/// JSON の metadata オブジェクトから EventMetadata Protobuf メッセージに変換する
fn json_to_event_metadata(metadata: &serde_json::Value) -> EventMetadata {
    EventMetadata {
        event_id: metadata
            .get("event_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        event_type: metadata
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        source: metadata
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        timestamp: metadata
            .get("timestamp")
            .and_then(|v| v.as_i64())
            .unwrap_or_default(),
        trace_id: metadata
            .get("trace_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        correlation_id: metadata
            .get("correlation_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        schema_version: metadata
            .get("schema_version")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as i32,
    }
}

/// JSON の items 配列から OrderItem Protobuf メッセージのベクタに変換する
fn json_to_order_items(items: Option<&serde_json::Value>) -> Vec<OrderItem> {
    items
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|item| OrderItem {
                    product_id: item
                        .get("product_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    quantity: item
                        .get("quantity")
                        .and_then(|v| v.as_i64())
                        .unwrap_or_default() as i32,
                    unit_price: item
                        .get("unit_price")
                        .and_then(|v| v.as_i64())
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// OutboxPoller — 未パブリッシュの Outbox イベントを定期的にポーリングし、
/// Kafka へ Protobuf 形式でパブリッシュ後にパブリッシュ済みとしてマークする。
pub struct OutboxPoller {
    order_repo: Arc<dyn OrderRepository>,
    event_publisher: Arc<dyn OrderEventPublisher>,
    poll_interval: Duration,
    batch_size: i64,
}

impl OutboxPoller {
    pub fn new(
        order_repo: Arc<dyn OrderRepository>,
        event_publisher: Arc<dyn OrderEventPublisher>,
        poll_interval: Duration,
        batch_size: i64,
    ) -> Self {
        Self {
            order_repo,
            event_publisher,
            poll_interval,
            batch_size,
        }
    }

    /// バックグラウンドタスクとしてポーリングを開始する。
    /// CancellationToken 等で停止制御する場合は shutdown_rx を使う。
    pub async fn run(&self, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) {
        tracing::info!(
            poll_interval_ms = self.poll_interval.as_millis() as u64,
            batch_size = self.batch_size,
            "outbox poller started"
        );

        let mut interval = time::interval(self.poll_interval);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(err) = self.poll_and_publish().await {
                        tracing::error!(error = %err, "outbox poller failed to process events");
                    }
                }
                _ = shutdown_rx.changed() => {
                    tracing::info!("outbox poller shutting down");
                    break;
                }
            }
        }
    }

    async fn poll_and_publish(&self) -> anyhow::Result<()> {
        let events = self
            .order_repo
            .fetch_unpublished_events(self.batch_size)
            .await?;

        if events.is_empty() {
            return Ok(());
        }

        tracing::debug!(count = events.len(), "processing outbox events");

        for event in &events {
            let payload = &event.payload;

            // イベント種別に応じて JSON payload を Protobuf メッセージに変換し publish する
            let publish_result = match event.event_type.as_str() {
                "order.created" => {
                    // JSON payload から OrderCreatedEvent Protobuf メッセージに変換
                    let metadata = payload.get("metadata").cloned().unwrap_or_default();
                    let proto_event = OrderCreatedEvent {
                        metadata: Some(json_to_event_metadata(&metadata)),
                        order_id: payload
                            .get("order_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        customer_id: payload
                            .get("customer_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        items: json_to_order_items(payload.get("items")),
                        total_amount: payload
                            .get("total_amount")
                            .and_then(|v| v.as_i64())
                            .unwrap_or_default(),
                        currency: payload
                            .get("currency")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                    };
                    self.event_publisher
                        .publish_order_created(&proto_event)
                        .await
                }
                "order.updated" => {
                    // JSON payload から OrderUpdatedEvent Protobuf メッセージに変換
                    let metadata = payload.get("metadata").cloned().unwrap_or_default();
                    let proto_event = OrderUpdatedEvent {
                        metadata: Some(json_to_event_metadata(&metadata)),
                        order_id: payload
                            .get("order_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        user_id: payload
                            .get("user_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        items: json_to_order_items(payload.get("items")),
                        total_amount: payload
                            .get("total_amount")
                            .and_then(|v| v.as_i64())
                            .unwrap_or_default(),
                        status: payload
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                    };
                    self.event_publisher
                        .publish_order_updated(&proto_event)
                        .await
                }
                "order.cancelled" => {
                    // JSON payload から OrderCancelledEvent Protobuf メッセージに変換
                    let metadata = payload.get("metadata").cloned().unwrap_or_default();
                    let proto_event = OrderCancelledEvent {
                        metadata: Some(json_to_event_metadata(&metadata)),
                        order_id: payload
                            .get("order_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        user_id: payload
                            .get("user_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        reason: payload
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                    };
                    self.event_publisher
                        .publish_order_cancelled(&proto_event)
                        .await
                }
                unknown => {
                    tracing::warn!(
                        event_type = unknown,
                        event_id = %event.id,
                        "unknown outbox event type, skipping"
                    );
                    continue;
                }
            };

            match publish_result {
                Ok(()) => {
                    if let Err(err) = self.order_repo.mark_event_published(event.id).await {
                        tracing::error!(
                            error = %err,
                            event_id = %event.id,
                            "failed to mark outbox event as published"
                        );
                    } else {
                        tracing::debug!(
                            event_id = %event.id,
                            event_type = %event.event_type,
                            "outbox event published and marked"
                        );
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        error = %err,
                        event_id = %event.id,
                        event_type = %event.event_type,
                        "failed to publish outbox event, will retry"
                    );
                    // 次のポーリングサイクルでリトライされる
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::outbox::OutboxEvent;
    use crate::domain::repository::order_repository::MockOrderRepository;
    use crate::usecase::event_publisher::MockOrderEventPublisher;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_outbox_event(event_type: &str) -> OutboxEvent {
        OutboxEvent {
            id: Uuid::new_v4(),
            aggregate_type: "order".to_string(),
            aggregate_id: Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            payload: serde_json::json!({"order_id": "test"}),
            created_at: Utc::now(),
            published_at: None,
        }
    }

    #[tokio::test]
    async fn test_poll_and_publish_empty() {
        let mut mock_repo = MockOrderRepository::new();
        let mock_publisher = MockOrderEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(|_| Ok(vec![]));

        let poller = OutboxPoller::new(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
            Duration::from_secs(1),
            10,
        );

        let result = poller.poll_and_publish().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_poll_and_publish_created_event() {
        let event = sample_outbox_event("order.created");
        let event_id = event.id;
        let events = vec![event];

        let mut mock_repo = MockOrderRepository::new();
        let mut mock_publisher = MockOrderEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_order_created()
            .times(1)
            .returning(|_| Ok(()));

        mock_repo
            .expect_mark_event_published()
            .withf(move |id| *id == event_id)
            .times(1)
            .returning(|_| Ok(()));

        let poller = OutboxPoller::new(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
            Duration::from_secs(1),
            10,
        );

        let result = poller.poll_and_publish().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_poll_and_publish_cancelled_event() {
        let event = sample_outbox_event("order.cancelled");
        let event_id = event.id;
        let events = vec![event];

        let mut mock_repo = MockOrderRepository::new();
        let mut mock_publisher = MockOrderEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_order_cancelled()
            .times(1)
            .returning(|_| Ok(()));

        mock_repo
            .expect_mark_event_published()
            .withf(move |id| *id == event_id)
            .times(1)
            .returning(|_| Ok(()));

        let poller = OutboxPoller::new(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
            Duration::from_secs(1),
            10,
        );

        let result = poller.poll_and_publish().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_poll_and_publish_failure_does_not_mark() {
        let event = sample_outbox_event("order.created");
        let events = vec![event];

        let mut mock_repo = MockOrderRepository::new();
        let mut mock_publisher = MockOrderEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_order_created()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("kafka unavailable")));

        // mark_event_published should NOT be called when publish fails
        mock_repo.expect_mark_event_published().times(0);

        let poller = OutboxPoller::new(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
            Duration::from_secs(1),
            10,
        );

        let result = poller.poll_and_publish().await;
        assert!(result.is_ok()); // poller itself should not fail
    }

    #[tokio::test]
    async fn test_run_shutdown() {
        let mut mock_repo = MockOrderRepository::new();
        let mock_publisher = MockOrderEventPublisher::new();

        // Allow any number of poll calls before shutdown
        mock_repo
            .expect_fetch_unpublished_events()
            .returning(|_| Ok(vec![]));

        let poller = Arc::new(OutboxPoller::new(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
            Duration::from_millis(50),
            10,
        ));

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        let poller_clone = poller.clone();
        let handle = tokio::spawn(async move {
            poller_clone.run(shutdown_rx).await;
        });

        // Let it poll at least once
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Signal shutdown
        shutdown_tx.send(true).unwrap();
        handle.await.unwrap();
    }
}
