use crate::domain::repository::payment_repository::PaymentRepository;
use crate::usecase::event_publisher::PaymentEventPublisher;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

/// OutboxPoller -- 未パブリッシュの Outbox イベントを定期的にポーリングし、
/// Kafka へパブリッシュ後にパブリッシュ済みとしてマークする。
pub struct OutboxPoller {
    payment_repo: Arc<dyn PaymentRepository>,
    event_publisher: Arc<dyn PaymentEventPublisher>,
    poll_interval: Duration,
    batch_size: i64,
}

impl OutboxPoller {
    pub fn new(
        payment_repo: Arc<dyn PaymentRepository>,
        event_publisher: Arc<dyn PaymentEventPublisher>,
        poll_interval: Duration,
        batch_size: i64,
    ) -> Self {
        Self {
            payment_repo,
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
            .payment_repo
            .fetch_unpublished_events(self.batch_size)
            .await?;

        if events.is_empty() {
            return Ok(());
        }

        tracing::debug!(count = events.len(), "processing outbox events");

        for event in &events {
            let publish_result = match event.event_type.as_str() {
                "payment.initiated" => {
                    self.event_publisher
                        .publish_payment_initiated(&event.payload)
                        .await
                }
                "payment.completed" => {
                    self.event_publisher
                        .publish_payment_completed(&event.payload)
                        .await
                }
                "payment.failed" => {
                    self.event_publisher
                        .publish_payment_failed(&event.payload)
                        .await
                }
                "payment.refunded" => {
                    self.event_publisher
                        .publish_payment_refunded(&event.payload)
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
                    if let Err(err) = self.payment_repo.mark_event_published(event.id).await {
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
    use crate::domain::repository::payment_repository::MockPaymentRepository;
    use crate::usecase::event_publisher::MockPaymentEventPublisher;
    use chrono::Utc;
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

    #[tokio::test]
    async fn test_poll_and_publish_empty() {
        let mut mock_repo = MockPaymentRepository::new();
        let mock_publisher = MockPaymentEventPublisher::new();

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
    async fn test_poll_and_publish_initiated_event() {
        let event = sample_outbox_event("payment.initiated");
        let event_id = event.id;
        let events = vec![event];

        let mut mock_repo = MockPaymentRepository::new();
        let mut mock_publisher = MockPaymentEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_payment_initiated()
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
    async fn test_poll_and_publish_refunded_event() {
        let event = sample_outbox_event("payment.refunded");
        let event_id = event.id;
        let events = vec![event];

        let mut mock_repo = MockPaymentRepository::new();
        let mut mock_publisher = MockPaymentEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_payment_refunded()
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
        let event = sample_outbox_event("payment.initiated");
        let events = vec![event];

        let mut mock_repo = MockPaymentRepository::new();
        let mut mock_publisher = MockPaymentEventPublisher::new();

        mock_repo
            .expect_fetch_unpublished_events()
            .times(1)
            .returning(move |_| Ok(events.clone()));

        mock_publisher
            .expect_publish_payment_initiated()
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
        let mut mock_repo = MockPaymentRepository::new();
        let mock_publisher = MockPaymentEventPublisher::new();

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
