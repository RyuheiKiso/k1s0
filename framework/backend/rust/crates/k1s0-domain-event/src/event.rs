use serde::Serialize;
use std::fmt::Debug;

/// ドメインイベントの基底 trait。
///
/// すべてのドメインイベントはこの trait を実装する。
/// `event_type` はイベントの識別子（例: `"order.created"`）を返す。
pub trait DomainEvent: Debug + Send + Sync + Serialize + 'static {
    /// イベント型の識別子を返す。
    fn event_type(&self) -> &str;

    /// イベントの集約ルート ID を返す（オプション）。
    fn aggregate_id(&self) -> Option<&str> {
        None
    }

    /// イベントの集約型を返す（オプション）。
    fn aggregate_type(&self) -> Option<&str> {
        None
    }
}
