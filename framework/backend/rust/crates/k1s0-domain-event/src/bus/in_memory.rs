use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::broadcast;
use tracing::{debug, error, warn};

use crate::envelope::EventEnvelope;
use crate::error::{PublishError, SubscribeError};
use crate::publisher::EventPublisher;
use crate::subscriber::{EventHandler, EventSubscriber, SubscriptionHandle};

/// インメモリのイベントバス。
///
/// `tokio::sync::broadcast` を使用してプロセス内イベント配信を行う。
/// テストやシングルプロセス構成で有用。
pub struct InMemoryEventBus {
    tx: broadcast::Sender<Arc<EventEnvelope>>,
}

impl InMemoryEventBus {
    /// 指定された容量でバスを作成する。
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
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
        self.tx
            .send(Arc::new(envelope))
            .map_err(|e| PublishError::Send(e.to_string()))?;
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
