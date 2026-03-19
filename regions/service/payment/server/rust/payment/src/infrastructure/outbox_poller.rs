use crate::domain::entity::event::{
    EventMetadata, PaymentCompletedDomainEvent, PaymentFailedDomainEvent,
    PaymentInitiatedDomainEvent, PaymentRefundedDomainEvent,
};
use crate::domain::repository::payment_repository::PaymentRepository;
use crate::usecase::event_publisher::PaymentEventPublisher;
use k1s0_outbox::util::{json_datetime, json_i64, json_str};
use std::sync::Arc;

/// JSON の metadata オブジェクトからドメインイベント用 EventMetadata に変換する。
/// metadata キーが存在しない場合は None を返す。
fn json_to_event_metadata(payload: &serde_json::Value) -> Option<EventMetadata> {
    let metadata = payload.get("metadata")?;
    Some(EventMetadata {
        event_id: json_str(metadata, "event_id"),
        event_type: json_str(metadata, "event_type"),
        source: json_str(metadata, "source"),
        timestamp: json_i64(metadata, "timestamp"),
        trace_id: json_str(metadata, "trace_id"),
        correlation_id: json_str(metadata, "correlation_id"),
        schema_version: json_i64(metadata, "schema_version").max(1) as i32,
        // 因果関係追跡用 ID
        causation_id: json_str(metadata, "causation_id"),
    })
}

/// PaymentRepository を OutboxEventFetcher として使えるようにするアダプタ実装。
/// ドメイン OutboxEvent は k1s0_outbox::OutboxEvent の再エクスポートのため、
/// 型変換なしで直接返せる。
#[async_trait::async_trait]
impl k1s0_outbox::OutboxEventFetcher for dyn PaymentRepository {
    async fn fetch_and_mark_events_published(
        &self,
        limit: i64,
    ) -> anyhow::Result<Vec<k1s0_outbox::OutboxEvent>> {
        PaymentRepository::fetch_and_mark_events_published(self, limit).await
    }
}

/// OutboxEventHandler の実装 — イベント種別ごとに JSON をドメインイベントに変換して publish する
struct PaymentOutboxHandler {
    publisher: Arc<dyn PaymentEventPublisher>,
}

#[async_trait::async_trait]
impl k1s0_outbox::OutboxEventHandler for PaymentOutboxHandler {
    /// JSON ペイロードをドメインイベントに変換してイベント種別ごとに publish する
    async fn handle_event(&self, event: &k1s0_outbox::OutboxEvent) -> anyhow::Result<bool> {
        let payload = &event.payload;

        match event.event_type.as_str() {
            // JSON payload を PaymentInitiatedDomainEvent に変換して publish する
            "payment.initiated" => {
                let domain_event = PaymentInitiatedDomainEvent {
                    metadata: json_to_event_metadata(payload),
                    payment_id: json_str(payload, "payment_id"),
                    order_id: json_str(payload, "order_id"),
                    customer_id: json_str(payload, "customer_id"),
                    amount: json_i64(payload, "amount"),
                    currency: json_str(payload, "currency"),
                    payment_method: json_str(payload, "payment_method"),
                    initiated_at: json_datetime(payload, "initiated_at"),
                };
                self.publisher
                    .publish_payment_initiated(&domain_event)
                    .await?;
                Ok(true)
            }
            // JSON payload を PaymentCompletedDomainEvent に変換して publish する
            "payment.completed" => {
                let domain_event = PaymentCompletedDomainEvent {
                    metadata: json_to_event_metadata(payload),
                    payment_id: json_str(payload, "payment_id"),
                    order_id: json_str(payload, "order_id"),
                    amount: json_i64(payload, "amount"),
                    currency: json_str(payload, "currency"),
                    transaction_id: json_str(payload, "transaction_id"),
                    completed_at: json_datetime(payload, "completed_at"),
                };
                self.publisher
                    .publish_payment_completed(&domain_event)
                    .await?;
                Ok(true)
            }
            // JSON payload を PaymentFailedDomainEvent に変換して publish する
            "payment.failed" => {
                let domain_event = PaymentFailedDomainEvent {
                    metadata: json_to_event_metadata(payload),
                    payment_id: json_str(payload, "payment_id"),
                    order_id: json_str(payload, "order_id"),
                    reason: json_str(payload, "reason"),
                    error_code: json_str(payload, "error_code"),
                    failed_at: json_datetime(payload, "failed_at"),
                };
                self.publisher.publish_payment_failed(&domain_event).await?;
                Ok(true)
            }
            // JSON payload を PaymentRefundedDomainEvent に変換して publish する
            "payment.refunded" => {
                let domain_event = PaymentRefundedDomainEvent {
                    metadata: json_to_event_metadata(payload),
                    payment_id: json_str(payload, "payment_id"),
                    order_id: json_str(payload, "order_id"),
                    refund_amount: json_i64(payload, "refund_amount"),
                    currency: json_str(payload, "currency"),
                    reason: json_str(payload, "reason"),
                    refunded_at: json_datetime(payload, "refunded_at"),
                };
                self.publisher
                    .publish_payment_refunded(&domain_event)
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
    payment_repo: Arc<dyn PaymentRepository>,
    event_publisher: Arc<dyn PaymentEventPublisher>,
    poll_interval: std::time::Duration,
    batch_size: i64,
) -> k1s0_outbox::OutboxEventPoller {
    let handler = Arc::new(PaymentOutboxHandler {
        publisher: event_publisher,
    });
    k1s0_outbox::poller::new_poller(payment_repo, handler, poll_interval, batch_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::outbox::OutboxEvent;
    use crate::domain::repository::payment_repository::MockPaymentRepository;
    use crate::usecase::event_publisher::MockPaymentEventPublisher;
    use chrono::Utc;
    use std::time::Duration;
    use uuid::Uuid;

    fn sample_outbox_event(event_type: &str) -> OutboxEvent {
        OutboxEvent {
            id: Uuid::new_v4(),
            aggregate_type: "payment".to_string(),
            aggregate_id: Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            payload: serde_json::json!({"payment_id": "test"}),
            created_at: Utc::now(),
            published_at: None,
        }
    }

    // ファクトリ関数でポーラーが正常に構築されることを確認する
    #[tokio::test]
    async fn test_new_outbox_poller_creation() {
        let mut mock_repo = MockPaymentRepository::new();
        let mock_publisher = MockPaymentEventPublisher::new();

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

    // payment.initiated イベントが正常にパブリッシュされることを確認する
    #[tokio::test]
    async fn test_poll_and_publish_initiated_event() {
        let event = sample_outbox_event("payment.initiated");
        let events = vec![event];

        let mut mock_repo = MockPaymentRepository::new();
        let mut mock_publisher = MockPaymentEventPublisher::new();

        mock_repo
            .expect_fetch_and_mark_events_published()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_payment_initiated()
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

    // payment.refunded イベントが正常にパブリッシュされることを確認する
    #[tokio::test]
    async fn test_poll_and_publish_refunded_event() {
        let event = sample_outbox_event("payment.refunded");
        let events = vec![event];

        let mut mock_repo = MockPaymentRepository::new();
        let mut mock_publisher = MockPaymentEventPublisher::new();

        mock_repo
            .expect_fetch_and_mark_events_published()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_payment_refunded()
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
        let event = sample_outbox_event("payment.initiated");
        let events = vec![event];

        let mut mock_repo = MockPaymentRepository::new();
        let mut mock_publisher = MockPaymentEventPublisher::new();

        mock_repo
            .expect_fetch_and_mark_events_published()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_payment_initiated()
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
        let mut mock_repo = MockPaymentRepository::new();
        let mock_publisher = MockPaymentEventPublisher::new();

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
