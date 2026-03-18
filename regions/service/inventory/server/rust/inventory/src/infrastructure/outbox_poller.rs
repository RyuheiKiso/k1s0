use crate::domain::entity::event::{
    EventMetadata, InventoryReleasedDomainEvent, InventoryReservedDomainEvent,
};
use crate::domain::repository::inventory_repository::InventoryRepository;
use crate::usecase::event_publisher::InventoryEventPublisher;
use k1s0_outbox::util::{json_datetime, json_i32, json_i64, json_str};
use std::sync::Arc;

/// JSON の metadata オブジェクトからドメインイベント用 EventMetadata に変換する
fn json_to_event_metadata(metadata: &serde_json::Value) -> EventMetadata {
    EventMetadata {
        event_id: json_str(metadata, "event_id"),
        event_type: json_str(metadata, "event_type"),
        source: json_str(metadata, "source"),
        timestamp: json_i64(metadata, "timestamp"),
        trace_id: json_str(metadata, "trace_id"),
        correlation_id: json_str(metadata, "correlation_id"),
        schema_version: json_i32(metadata, "schema_version").max(1),
        // 因果関係追跡用 ID
        causation_id: json_str(metadata, "causation_id"),
    }
}

/// InventoryRepository を OutboxEventFetcher として使えるようにするアダプタ実装。
/// ドメイン OutboxEvent は k1s0_outbox::OutboxEvent の再エクスポートのため、
/// 型変換なしで直接返せる。
#[async_trait::async_trait]
impl k1s0_outbox::OutboxEventFetcher for dyn InventoryRepository {
    async fn fetch_and_mark_events_published(
        &self,
        limit: i64,
    ) -> anyhow::Result<Vec<k1s0_outbox::OutboxEvent>> {
        InventoryRepository::fetch_and_mark_events_published(self, limit).await
    }
}

/// OutboxEventHandler の実装 — イベント種別ごとに JSON をドメインイベントに変換して publish する
struct InventoryOutboxHandler {
    publisher: Arc<dyn InventoryEventPublisher>,
}

#[async_trait::async_trait]
impl k1s0_outbox::OutboxEventHandler for InventoryOutboxHandler {
    /// イベント種別に応じて JSON payload をドメインイベントに変換し publish する
    async fn handle_event(&self, event: &k1s0_outbox::OutboxEvent) -> anyhow::Result<bool> {
        let payload = &event.payload;

        match event.event_type.as_str() {
            // JSON payload を InventoryReservedDomainEvent に変換して publish する
            "inventory.reserved" => {
                let metadata = payload.get("metadata").cloned().unwrap_or_default();
                let domain_event = InventoryReservedDomainEvent {
                    metadata: Some(json_to_event_metadata(&metadata)),
                    order_id: json_str(payload, "order_id"),
                    product_id: json_str(payload, "product_id"),
                    quantity: json_i32(payload, "quantity"),
                    warehouse_id: json_str(payload, "warehouse_id"),
                    reserved_at: json_datetime(payload, "reserved_at"),
                };
                self.publisher
                    .publish_inventory_reserved(&domain_event)
                    .await?;
                Ok(true)
            }
            // JSON payload を InventoryReleasedDomainEvent に変換して publish する
            "inventory.released" => {
                let metadata = payload.get("metadata").cloned().unwrap_or_default();
                let domain_event = InventoryReleasedDomainEvent {
                    metadata: Some(json_to_event_metadata(&metadata)),
                    order_id: json_str(payload, "order_id"),
                    product_id: json_str(payload, "product_id"),
                    quantity: json_i32(payload, "quantity"),
                    warehouse_id: json_str(payload, "warehouse_id"),
                    reason: json_str(payload, "reason"),
                    released_at: json_datetime(payload, "released_at"),
                };
                self.publisher
                    .publish_inventory_released(&domain_event)
                    .await?;
                Ok(true)
            }
            // 未知のイベント種別はスキップする
            _ => Ok(false),
        }
    }
}

/// OutboxEventPoller のファクトリ関数。
/// startup.rs から呼び出して汎用ポーラーを構築する。
/// k1s0_outbox::poller::new_poller を使い、OutboxSource ボイラープレートを削除。
pub fn new_outbox_poller(
    inventory_repo: Arc<dyn InventoryRepository>,
    event_publisher: Arc<dyn InventoryEventPublisher>,
    poll_interval: std::time::Duration,
    batch_size: i64,
) -> k1s0_outbox::OutboxEventPoller {
    let handler = Arc::new(InventoryOutboxHandler {
        publisher: event_publisher,
    });
    k1s0_outbox::poller::new_poller(inventory_repo, handler, poll_interval, batch_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::outbox::OutboxEvent;
    use crate::domain::repository::inventory_repository::MockInventoryRepository;
    use crate::usecase::event_publisher::MockInventoryEventPublisher;
    use chrono::Utc;
    use std::time::Duration;
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

    // ファクトリ関数でポーラーが正常に構築されることを確認する
    #[tokio::test]
    async fn test_new_outbox_poller_creation() {
        let mut mock_repo = MockInventoryRepository::new();
        let mock_publisher = MockInventoryEventPublisher::new();

        mock_repo
            .expect_fetch_and_mark_events_published()
            .returning(|_| Ok(vec![]));

        let _poller = new_outbox_poller(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
            Duration::from_secs(1),
            10,
        );
    }

    // inventory.reserved イベントが正常にパブリッシュされることを確認する
    #[tokio::test]
    async fn test_poll_and_publish_reserved_event() {
        let event = sample_outbox_event("inventory.reserved");
        let events = vec![event];

        let mut mock_repo = MockInventoryRepository::new();
        let mut mock_publisher = MockInventoryEventPublisher::new();

        mock_repo
            .expect_fetch_and_mark_events_published()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_inventory_reserved()
            .times(1)
            .returning(|_| Ok(()));

        let poller = new_outbox_poller(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
            Duration::from_secs(1),
            10,
        );

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let poller = Arc::new(poller);
        let poller_clone = poller.clone();
        let handle = tokio::spawn(async move {
            poller_clone.run(shutdown_rx).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        shutdown_tx.send(true).unwrap();
        handle.await.unwrap();
    }

    // inventory.released イベントが正常にパブリッシュされることを確認する
    #[tokio::test]
    async fn test_poll_and_publish_released_event() {
        let event = sample_outbox_event("inventory.released");
        let events = vec![event];

        let mut mock_repo = MockInventoryRepository::new();
        let mut mock_publisher = MockInventoryEventPublisher::new();

        mock_repo
            .expect_fetch_and_mark_events_published()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_inventory_released()
            .times(1)
            .returning(|_| Ok(()));

        let poller = new_outbox_poller(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
            Duration::from_secs(1),
            10,
        );

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let poller = Arc::new(poller);
        let poller_clone = poller.clone();
        let handle = tokio::spawn(async move {
            poller_clone.run(shutdown_rx).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        shutdown_tx.send(true).unwrap();
        handle.await.unwrap();
    }

    // publish 失敗時もポーラー自体はエラーにならないことを確認する
    #[tokio::test]
    async fn test_poll_and_publish_failure_logs_warning() {
        let event = sample_outbox_event("inventory.reserved");
        let events = vec![event];

        let mut mock_repo = MockInventoryRepository::new();
        let mut mock_publisher = MockInventoryEventPublisher::new();

        mock_repo
            .expect_fetch_and_mark_events_published()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_inventory_reserved()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("kafka unavailable")));

        let poller = new_outbox_poller(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
            Duration::from_secs(1),
            10,
        );

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let poller = Arc::new(poller);
        let poller_clone = poller.clone();
        let handle = tokio::spawn(async move {
            poller_clone.run(shutdown_rx).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        shutdown_tx.send(true).unwrap();
        handle.await.unwrap();
    }

    // シャットダウンシグナルを受信すると run が正常終了することを確認する
    #[tokio::test]
    async fn test_run_shutdown() {
        let mut mock_repo = MockInventoryRepository::new();
        let mock_publisher = MockInventoryEventPublisher::new();

        mock_repo
            .expect_fetch_and_mark_events_published()
            .returning(|_| Ok(vec![]));

        let poller = Arc::new(new_outbox_poller(
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
