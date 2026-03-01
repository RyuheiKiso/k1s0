use async_trait::async_trait;

use crate::error::EventBusError;
use crate::event::{DomainEvent, Event};

/// レガシーイベントハンドラートレイト（後方互換性のため維持）。
#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait EventHandler: Send + Sync {
    fn event_type(&self) -> &str;
    async fn handle(&self, event: Event) -> Result<(), EventBusError>;
}

/// DDD ドメインイベントハンドラートレイト。
/// ジェネリックなドメインイベントを処理する。
#[async_trait]
pub trait DomainEventHandler<T: DomainEvent>: Send + Sync {
    /// イベントを処理する。
    async fn handle(&self, event: &T) -> Result<(), EventBusError>;
}
