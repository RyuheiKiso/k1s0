use std::sync::Arc;

use async_trait::async_trait;

use crate::error::EventBusError;
use crate::event::{DomainEvent, Event};

/// レガシーイベントハンドラートレイト（後方互換性のため維持）。
/// ハンドラー間で Event のデータを共有するため Arc<Event> を受け取る（SL-3 監査対応）。
/// これにより publish 時に `serde_json::Value` を含む Event 全体をハンドラー数分コピーせず済む。
#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait EventHandler: Send + Sync {
    fn event_type(&self) -> &str;
    async fn handle(&self, event: Arc<Event>) -> Result<(), EventBusError>;
}

/// DDD ドメインイベントハンドラートレイト。
/// ジェネリックなドメインイベントを処理する。
#[async_trait]
pub trait DomainEventHandler<T: DomainEvent>: Send + Sync {
    /// イベントを処理する。
    async fn handle(&self, event: &T) -> Result<(), EventBusError>;
}
