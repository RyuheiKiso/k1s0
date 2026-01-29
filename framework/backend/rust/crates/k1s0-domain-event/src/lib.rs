//! k1s0-domain-event
//!
//! ドメインイベントの発行・購読・Outbox パターンを提供するライブラリ。
//!
//! # 概要
//!
//! - `DomainEvent` trait: すべてのドメインイベントが実装する基底 trait
//! - `EventEnvelope`: イベント本体 + メタデータの格納構造
//! - `EventPublisher` / `EventSubscriber`: 発行・購読の抽象
//! - `InMemoryEventBus`: テスト・シングルプロセス向けインメモリ実装
//! - `outbox`: Outbox パターン（`outbox` feature で有効化）
//!
//! # 使用例
//!
//! ```
//! use k1s0_domain_event::{DomainEvent, EventEnvelope, EventMetadata};
//! use serde::Serialize;
//!
//! #[derive(Debug, Serialize)]
//! struct OrderCreated {
//!     order_id: String,
//! }
//!
//! impl DomainEvent for OrderCreated {
//!     fn event_type(&self) -> &str {
//!         "order.created"
//!     }
//!
//!     fn aggregate_id(&self) -> Option<&str> {
//!         Some(&self.order_id)
//!     }
//! }
//!
//! let event = OrderCreated { order_id: "ord-123".into() };
//! let envelope = EventEnvelope::from_event(&event, "order-service").unwrap();
//! assert_eq!(envelope.event_type, "order.created");
//! ```

pub mod bus;
mod envelope;
mod error;
mod event;
#[cfg(feature = "outbox")]
pub mod outbox;
mod publisher;
mod subscriber;

pub use envelope::{EventEnvelope, EventMetadata};
pub use error::{HandlerError, OutboxError, PublishError, SubscribeError};
pub use event::DomainEvent;
pub use publisher::EventPublisher;
pub use subscriber::{EventHandler, EventSubscriber, SubscriptionHandle};
