use crate::domain::entity::event::{
    EventMetadata, OrderCancelledDomainEvent, OrderCreatedDomainEvent, OrderItemEvent,
    OrderUpdatedDomainEvent,
};
use crate::domain::repository::order_repository::OrderRepository;
use crate::usecase::event_publisher::OrderEventPublisher;
use k1s0_outbox::util::{json_i32, json_i64, json_str};
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
        causation_id: json_str(metadata, "causation_id"),
    }
}

/// JSON の items 配列からドメインイベント用 OrderItemEvent のベクタに変換する
fn json_to_order_items(items: Option<&serde_json::Value>) -> Vec<OrderItemEvent> {
    items
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|item| OrderItemEvent {
                    product_id: json_str(item, "product_id"),
                    quantity: json_i32(item, "quantity"),
                    unit_price: json_i64(item, "unit_price"),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// OrderRepository を OutboxEventFetcher として使えるようにするアダプタ実装。
/// ドメイン OutboxEvent は k1s0_outbox::OutboxEvent の再エクスポートのため、
/// 型変換なしで直接返せる。
#[async_trait::async_trait]
impl k1s0_outbox::OutboxEventFetcher for dyn OrderRepository {
    async fn fetch_unpublished_events(
        &self,
        limit: i64,
    ) -> anyhow::Result<Vec<k1s0_outbox::OutboxEvent>> {
        OrderRepository::fetch_unpublished_events(self, limit).await
    }

    async fn mark_events_published(&self, ids: &[uuid::Uuid]) -> anyhow::Result<()> {
        OrderRepository::mark_events_published(self, ids).await
    }
}

/// OutboxEventHandler の実装 — イベント種別ごとに JSON を ドメインイベントに変換して publish する
struct OrderOutboxHandler {
    publisher: Arc<dyn OrderEventPublisher>,
}

#[async_trait::async_trait]
impl k1s0_outbox::OutboxEventHandler for OrderOutboxHandler {
    /// イベント種別に応じて JSON payload をドメインイベントに変換し publish する
    async fn handle_event(&self, event: &k1s0_outbox::OutboxEvent) -> anyhow::Result<bool> {
        let payload = &event.payload;

        match event.event_type.as_str() {
            // JSON payload から OrderCreatedDomainEvent に変換して publish する
            "order.created" => {
                let metadata = payload.get("metadata").cloned().unwrap_or_default();
                let domain_event = OrderCreatedDomainEvent {
                    metadata: Some(json_to_event_metadata(&metadata)),
                    order_id: json_str(payload, "order_id"),
                    customer_id: json_str(payload, "customer_id"),
                    items: json_to_order_items(payload.get("items")),
                    total_amount: json_i64(payload, "total_amount"),
                    currency: json_str(payload, "currency"),
                };
                self.publisher.publish_order_created(&domain_event).await?;
                Ok(true)
            }
            // JSON payload から OrderUpdatedDomainEvent に変換して publish する
            "order.updated" => {
                let metadata = payload.get("metadata").cloned().unwrap_or_default();
                let domain_event = OrderUpdatedDomainEvent {
                    metadata: Some(json_to_event_metadata(&metadata)),
                    order_id: json_str(payload, "order_id"),
                    user_id: json_str(payload, "user_id"),
                    items: json_to_order_items(payload.get("items")),
                    total_amount: json_i64(payload, "total_amount"),
                    status: json_str(payload, "status"),
                };
                self.publisher.publish_order_updated(&domain_event).await?;
                Ok(true)
            }
            // JSON payload から OrderCancelledDomainEvent に変換して publish する
            "order.cancelled" => {
                let metadata = payload.get("metadata").cloned().unwrap_or_default();
                let domain_event = OrderCancelledDomainEvent {
                    metadata: Some(json_to_event_metadata(&metadata)),
                    order_id: json_str(payload, "order_id"),
                    user_id: json_str(payload, "user_id"),
                    reason: json_str(payload, "reason"),
                };
                self.publisher
                    .publish_order_cancelled(&domain_event)
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
    order_repo: Arc<dyn OrderRepository>,
    event_publisher: Arc<dyn OrderEventPublisher>,
    poll_interval: std::time::Duration,
    batch_size: i64,
) -> k1s0_outbox::OutboxEventPoller {
    let handler = Arc::new(OrderOutboxHandler {
        publisher: event_publisher,
    });
    k1s0_outbox::poller::new_poller(order_repo, handler, poll_interval, batch_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::outbox::OutboxEvent;
    use crate::domain::repository::order_repository::MockOrderRepository;
    use crate::usecase::event_publisher::MockOrderEventPublisher;
    use chrono::Utc;
    use std::time::Duration;
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

    // ファクトリ関数でポーラーが正常に構築されることを確認する
    #[tokio::test]
    async fn test_new_outbox_poller_creation() {
        let mut mock_repo = MockOrderRepository::new();
        let mock_publisher = MockOrderEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .returning(|_| Ok(vec![]));

        let _poller = new_outbox_poller(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
            Duration::from_secs(1),
            10,
        );
    }

    // シャットダウンシグナルを受信すると run が正常終了することを確認する
    #[tokio::test]
    async fn test_run_shutdown() {
        let mut mock_repo = MockOrderRepository::new();
        let mock_publisher = MockOrderEventPublisher::new();

        // シャットダウンまでの間、任意回数のポーリングを許可する
        mock_repo
            .expect_fetch_unpublished_events()
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

        // 少なくとも1回ポーリングさせる
        tokio::time::sleep(Duration::from_millis(100)).await;

        // シャットダウンシグナルを送信
        shutdown_tx.send(true).unwrap();
        handle.await.unwrap();
    }

    // order.created イベントが正常にパブリッシュされることを確認する
    #[tokio::test]
    async fn test_poll_and_publish_created_event() {
        let event = sample_outbox_event("order.created");
        let events = vec![event];

        let mut mock_repo = MockOrderRepository::new();
        let mut mock_publisher = MockOrderEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        // at-least-once: publish 成功後に mark が呼ばれることを確認する
        mock_repo
            .expect_mark_events_published()
            .times(1)
            .returning(|_| Ok(()));

        mock_publisher
            .expect_publish_order_created()
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

    // order.cancelled イベントが正常にパブリッシュされることを確認する
    #[tokio::test]
    async fn test_poll_and_publish_cancelled_event() {
        let event = sample_outbox_event("order.cancelled");
        let events = vec![event];

        let mut mock_repo = MockOrderRepository::new();
        let mut mock_publisher = MockOrderEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        // at-least-once: publish 成功後に mark が呼ばれることを確認する
        mock_repo
            .expect_mark_events_published()
            .times(1)
            .returning(|_| Ok(()));

        mock_publisher
            .expect_publish_order_cancelled()
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

    // publish 失敗時もポーラー自体はエラーにならず、mark も呼ばれないことを確認する
    // at-least-once: 失敗したイベントは mark されず、次回ポーリングでリトライされる
    #[tokio::test]
    async fn test_poll_and_publish_failure_logs_warning() {
        let event = sample_outbox_event("order.created");
        let events = vec![event];

        let mut mock_repo = MockOrderRepository::new();
        let mut mock_publisher = MockOrderEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        // publish 失敗時は mark_events_published が呼ばれない（at-least-once 保証）

        mock_publisher
            .expect_publish_order_created()
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
}
