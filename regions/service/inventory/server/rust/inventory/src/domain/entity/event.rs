// 在庫ドメインイベント型。
// usecase層はProto型に依存せず、このドメインイベント型を使用する。
// インフラ層（Kafka producer）でProto型への変換を行う。

use chrono::{DateTime, Utc};

/// イベントメタデータ。全イベント共通の追跡情報を保持する。
#[derive(Debug, Clone)]
pub struct EventMetadata {
    /// イベント一意識別子
    pub event_id: String,
    /// イベント種別（例: "inventory.reserved"）
    pub event_type: String,
    /// イベント発生元サービス名
    pub source: String,
    /// イベント発生タイムスタンプ（ミリ秒エポック）
    pub timestamp: i64,
    /// 分散トレーシング用トレースID
    pub trace_id: String,
    /// リクエスト相関ID
    pub correlation_id: String,
    /// スキーマバージョン
    pub schema_version: i32,
    /// 因果関係ID
    pub causation_id: String,
}

/// 在庫予約ドメインイベント
#[derive(Debug, Clone)]
pub struct InventoryReservedDomainEvent {
    /// イベントメタデータ
    pub metadata: Option<EventMetadata>,
    /// 注文ID
    pub order_id: String,
    /// 商品ID
    pub product_id: String,
    /// 予約数量
    pub quantity: i32,
    /// 倉庫ID
    pub warehouse_id: String,
    /// 予約日時
    pub reserved_at: Option<DateTime<Utc>>,
}

/// 在庫解放ドメインイベント
#[derive(Debug, Clone)]
pub struct InventoryReleasedDomainEvent {
    /// イベントメタデータ
    pub metadata: Option<EventMetadata>,
    /// 注文ID
    pub order_id: String,
    /// 商品ID
    pub product_id: String,
    /// 解放数量
    pub quantity: i32,
    /// 倉庫ID
    pub warehouse_id: String,
    /// 解放理由
    pub reason: String,
    /// 解放日時
    pub released_at: Option<DateTime<Utc>>,
}

/// ドメインイベントのデフォルト値を生成するヘルパー（テスト用）
impl Default for InventoryReservedDomainEvent {
    fn default() -> Self {
        Self {
            metadata: None,
            order_id: String::new(),
            product_id: String::new(),
            quantity: 0,
            warehouse_id: String::new(),
            reserved_at: None,
        }
    }
}

/// ドメインイベントのデフォルト値を生成するヘルパー（テスト用）
impl Default for InventoryReleasedDomainEvent {
    fn default() -> Self {
        Self {
            metadata: None,
            order_id: String::new(),
            product_id: String::new(),
            quantity: 0,
            warehouse_id: String::new(),
            reason: String::new(),
            released_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_reserved_domain_event_default() {
        // デフォルト値でドメインイベントが正しく生成されることを検証
        let event = InventoryReservedDomainEvent::default();
        assert!(event.order_id.is_empty());
        assert_eq!(event.quantity, 0);
    }

    #[test]
    fn test_inventory_released_domain_event_default() {
        // デフォルト値でドメインイベントが正しく生成されることを検証
        let event = InventoryReleasedDomainEvent::default();
        assert!(event.order_id.is_empty());
        assert!(event.reason.is_empty());
    }

    #[test]
    fn test_inventory_reserved_domain_event_with_fields() {
        // フィールド指定でドメインイベントが正しく構築されることを検証
        let event = InventoryReservedDomainEvent {
            metadata: Some(EventMetadata {
                event_id: "evt-001".to_string(),
                event_type: "inventory.reserved".to_string(),
                source: "inventory-server".to_string(),
                timestamp: 1700000000000,
                trace_id: "".to_string(),
                correlation_id: "order-001".to_string(),
                schema_version: 1,
                causation_id: "".to_string(),
            }),
            order_id: "order-001".to_string(),
            product_id: "prod-001".to_string(),
            quantity: 5,
            warehouse_id: "wh-001".to_string(),
            reserved_at: None,
        };
        assert_eq!(event.order_id, "order-001");
        assert_eq!(event.quantity, 5);
    }
}
