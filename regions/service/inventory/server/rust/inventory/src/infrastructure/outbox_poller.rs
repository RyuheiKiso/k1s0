use crate::domain::repository::inventory_repository::InventoryRepository;
use crate::proto::k1s0::event::service::inventory::v1::{
    InventoryReleasedEvent, InventoryReservedEvent,
};
use crate::proto::k1s0::system::common::v1::EventMetadata;
use crate::usecase::event_publisher::InventoryEventPublisher;
// カスタム Timestamp 型（k1s0.system.common.v1.Timestamp）を使用
use crate::proto::k1s0::system::common::v1::Timestamp;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

// ISO 8601 文字列を prost Timestamp に変換する
fn parse_timestamp(s: &str) -> Option<Timestamp> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        })
}

/// OutboxPoller — 未パブリッシュの Outbox イベントを定期的にポーリングし、
/// Kafka へパブリッシュ後にパブリッシュ済みとしてマークする。
pub struct OutboxPoller {
    inventory_repo: Arc<dyn InventoryRepository>,
    event_publisher: Arc<dyn InventoryEventPublisher>,
    poll_interval: Duration,
    batch_size: i64,
}

impl OutboxPoller {
    pub fn new(
        inventory_repo: Arc<dyn InventoryRepository>,
        event_publisher: Arc<dyn InventoryEventPublisher>,
        poll_interval: Duration,
        batch_size: i64,
    ) -> Self {
        Self {
            inventory_repo,
            event_publisher,
            poll_interval,
            batch_size,
        }
    }

    /// バックグラウンドタスクとしてポーリングを開始する。
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
            .inventory_repo
            .fetch_unpublished_events(self.batch_size)
            .await?;

        if events.is_empty() {
            return Ok(());
        }

        tracing::debug!(count = events.len(), "processing outbox events");

        for event in &events {
            let publish_result = match event.event_type.as_str() {
                // JSON payload を InventoryReservedEvent (Protobuf) に変換して publish する
                "inventory.reserved" => {
                    let payload = &event.payload;
                    let metadata = payload.get("metadata").cloned().unwrap_or_default();
                    let proto_event = InventoryReservedEvent {
                        metadata: Some(EventMetadata {
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
                            // 因果関係追跡用 ID
                            causation_id: metadata
                                .get("causation_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string(),
                        }),
                        order_id: payload
                            .get("order_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        product_id: payload
                            .get("product_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        quantity: payload
                            .get("quantity")
                            .and_then(|v| v.as_i64())
                            .unwrap_or_default() as i32,
                        warehouse_id: payload
                            .get("warehouse_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        reserved_at: payload
                            .get("reserved_at")
                            .and_then(|v| v.as_str())
                            .and_then(parse_timestamp),
                    };
                    self.event_publisher
                        .publish_inventory_reserved(&proto_event)
                        .await
                }
                // JSON payload を InventoryReleasedEvent (Protobuf) に変換して publish する
                "inventory.released" => {
                    let payload = &event.payload;
                    let metadata = payload.get("metadata").cloned().unwrap_or_default();
                    let proto_event = InventoryReleasedEvent {
                        metadata: Some(EventMetadata {
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
                            // 因果関係追跡用 ID
                            causation_id: metadata
                                .get("causation_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string(),
                        }),
                        order_id: payload
                            .get("order_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        product_id: payload
                            .get("product_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        quantity: payload
                            .get("quantity")
                            .and_then(|v| v.as_i64())
                            .unwrap_or_default() as i32,
                        warehouse_id: payload
                            .get("warehouse_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        reason: payload
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        released_at: payload
                            .get("released_at")
                            .and_then(|v| v.as_str())
                            .and_then(parse_timestamp),
                    };
                    self.event_publisher
                        .publish_inventory_released(&proto_event)
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
                    if let Err(err) = self.inventory_repo.mark_event_published(event.id).await {
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
    use crate::domain::repository::inventory_repository::MockInventoryRepository;
    use crate::usecase::event_publisher::MockInventoryEventPublisher;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_outbox_event(event_type: &str) -> OutboxEvent {
        OutboxEvent {
            id: Uuid::new_v4(),
            aggregate_type: "inventory".to_string(),
            aggregate_id: Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            payload: serde_json::json!({"order_id": "test"}),
            created_at: Utc::now(),
            published_at: None,
        }
    }

    #[tokio::test]
    async fn test_poll_and_publish_empty() {
        let mut mock_repo = MockInventoryRepository::new();
        let mock_publisher = MockInventoryEventPublisher::new();

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
    async fn test_poll_and_publish_reserved_event() {
        let event = sample_outbox_event("inventory.reserved");
        let event_id = event.id;
        let events = vec![event];

        let mut mock_repo = MockInventoryRepository::new();
        let mut mock_publisher = MockInventoryEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_inventory_reserved()
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
    async fn test_poll_and_publish_released_event() {
        let event = sample_outbox_event("inventory.released");
        let event_id = event.id;
        let events = vec![event];

        let mut mock_repo = MockInventoryRepository::new();
        let mut mock_publisher = MockInventoryEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_inventory_released()
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
        let event = sample_outbox_event("inventory.reserved");
        let events = vec![event];

        let mut mock_repo = MockInventoryRepository::new();
        let mut mock_publisher = MockInventoryEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_inventory_reserved()
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
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_shutdown() {
        let mut mock_repo = MockInventoryRepository::new();
        let mock_publisher = MockInventoryEventPublisher::new();

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

        tokio::time::sleep(Duration::from_millis(100)).await;

        shutdown_tx.send(true).unwrap();
        handle.await.unwrap();
    }
}
