use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// OutboxEvent はサービス層のアウトボックステーブルに格納されるイベントを表す。
///
/// 各サービス（order, inventory, payment）で共通の構造体。
/// DB トランザクション内でビジネス操作と同時に書き込まれ、
/// OutboxEventPoller が定期的にポーリングして Kafka へパブリッシュする。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    /// イベントの一意識別子
    pub id: Uuid,
    /// 集約タイプ（例: "order", "inventory", "payment"）
    pub aggregate_type: String,
    /// 集約 ID（例: 注文ID、商品ID）
    pub aggregate_id: String,
    /// イベント種別（例: "order.created", "inventory.reserved"）
    pub event_type: String,
    /// イベントペイロード（JSON）
    pub payload: serde_json::Value,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// パブリッシュ完了日時（未パブリッシュの場合は None）
    pub published_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // OutboxEvent が正しく生成・シリアライズできることを確認する。
    #[test]
    fn test_outbox_event_creation() {
        let event = OutboxEvent {
            id: Uuid::new_v4(),
            aggregate_type: "order".to_string(),
            aggregate_id: Uuid::new_v4().to_string(),
            event_type: "order.created".to_string(),
            payload: serde_json::json!({"order_id": "test"}),
            created_at: Utc::now(),
            published_at: None,
        };
        assert!(event.published_at.is_none());
        assert_eq!(event.aggregate_type, "order");
    }

    // OutboxEvent のシリアライズ・デシリアライズが双方向で正しく動作することを確認する。
    #[test]
    fn test_outbox_event_serialization_roundtrip() {
        let event = OutboxEvent {
            id: Uuid::new_v4(),
            aggregate_type: "order".to_string(),
            aggregate_id: "ORD-001".to_string(),
            event_type: "order.created".to_string(),
            payload: serde_json::json!({"key": "value"}),
            created_at: Utc::now(),
            published_at: None,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: OutboxEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.aggregate_id, "ORD-001");
        assert_eq!(deserialized.event_type, "order.created");
    }
}
