// 注文ドメインイベント型。
// usecase層はProto型に依存せず、このドメインイベント型を使用する。
// インフラ層（Kafka producer）でProto型への変換を行う。

/// イベントメタデータ。全イベント共通の追跡情報を保持する。
#[derive(Debug, Clone)]
pub struct EventMetadata {
    /// イベント一意識別子
    pub event_id: String,
    /// イベント種別（例: "order.created"）
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

/// 注文明細のドメインイベント表現
#[derive(Debug, Clone)]
pub struct OrderItemEvent {
    /// 商品ID
    pub product_id: String,
    /// 数量
    pub quantity: i32,
    /// 単価（最小通貨単位）
    pub unit_price: i64,
}

/// 注文作成ドメインイベント
#[derive(Debug, Clone)]
pub struct OrderCreatedDomainEvent {
    /// イベントメタデータ
    pub metadata: Option<EventMetadata>,
    /// 注文ID
    pub order_id: String,
    /// 顧客ID
    pub customer_id: String,
    /// 注文明細
    pub items: Vec<OrderItemEvent>,
    /// 合計金額（最小通貨単位）
    pub total_amount: i64,
    /// 通貨コード
    pub currency: String,
}

/// 注文更新ドメインイベント
#[derive(Debug, Clone)]
pub struct OrderUpdatedDomainEvent {
    /// イベントメタデータ
    pub metadata: Option<EventMetadata>,
    /// 注文ID
    pub order_id: String,
    /// 更新実行ユーザーID
    pub user_id: String,
    /// 更新後の注文明細
    pub items: Vec<OrderItemEvent>,
    /// 更新後の合計金額
    pub total_amount: i64,
    /// 更新後のステータス
    pub status: String,
}

/// 注文キャンセルドメインイベント
#[derive(Debug, Clone)]
pub struct OrderCancelledDomainEvent {
    /// イベントメタデータ
    pub metadata: Option<EventMetadata>,
    /// 注文ID
    pub order_id: String,
    /// キャンセル実行ユーザーID
    pub user_id: String,
    /// キャンセル理由
    pub reason: String,
}

/// ドメインイベントのデフォルト値を生成するヘルパー（テスト用）
impl Default for OrderCreatedDomainEvent {
    fn default() -> Self {
        Self {
            metadata: None,
            order_id: String::new(),
            customer_id: String::new(),
            items: Vec::new(),
            total_amount: 0,
            currency: String::new(),
        }
    }
}

/// ドメインイベントのデフォルト値を生成するヘルパー（テスト用）
impl Default for OrderUpdatedDomainEvent {
    fn default() -> Self {
        Self {
            metadata: None,
            order_id: String::new(),
            user_id: String::new(),
            items: Vec::new(),
            total_amount: 0,
            status: String::new(),
        }
    }
}

/// ドメインイベントのデフォルト値を生成するヘルパー（テスト用）
impl Default for OrderCancelledDomainEvent {
    fn default() -> Self {
        Self {
            metadata: None,
            order_id: String::new(),
            user_id: String::new(),
            reason: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_created_domain_event_default() {
        // デフォルト値でドメインイベントが正しく生成されることを検証
        let event = OrderCreatedDomainEvent::default();
        assert!(event.order_id.is_empty());
        assert!(event.items.is_empty());
    }

    #[test]
    fn test_order_updated_domain_event_default() {
        // デフォルト値でドメインイベントが正しく生成されることを検証
        let event = OrderUpdatedDomainEvent::default();
        assert!(event.order_id.is_empty());
        assert!(event.status.is_empty());
    }

    #[test]
    fn test_order_cancelled_domain_event_default() {
        // デフォルト値でドメインイベントが正しく生成されることを検証
        let event = OrderCancelledDomainEvent::default();
        assert!(event.order_id.is_empty());
        assert!(event.reason.is_empty());
    }

    #[test]
    fn test_order_created_domain_event_with_items() {
        // 明細付きドメインイベントが正しく構築されることを検証
        let event = OrderCreatedDomainEvent {
            metadata: Some(EventMetadata {
                event_id: "evt-001".to_string(),
                event_type: "order.created".to_string(),
                source: "order-server".to_string(),
                timestamp: 1700000000000,
                trace_id: "".to_string(),
                correlation_id: "order-001".to_string(),
                schema_version: 1,
                causation_id: "".to_string(),
            }),
            order_id: "order-001".to_string(),
            customer_id: "cust-001".to_string(),
            items: vec![OrderItemEvent {
                product_id: "prod-001".to_string(),
                quantity: 2,
                unit_price: 1000,
            }],
            total_amount: 2000,
            currency: "JPY".to_string(),
        };
        assert_eq!(event.order_id, "order-001");
        assert_eq!(event.items.len(), 1);
        assert_eq!(event.items[0].quantity, 2);
    }
}
