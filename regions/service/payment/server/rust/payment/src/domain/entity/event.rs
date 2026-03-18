// 決済ドメインイベント型。
// usecase層はProto型に依存せず、このドメインイベント型を使用する。
// インフラ層（Kafka producer）でProto型への変換を行う。

use chrono::{DateTime, Utc};

/// イベントメタデータ。全イベント共通の追跡情報を保持する。
#[derive(Debug, Clone)]
pub struct EventMetadata {
    /// イベント一意識別子
    pub event_id: String,
    /// イベント種別（例: "payment.initiated"）
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

/// 決済開始ドメインイベント
#[derive(Debug, Clone)]
pub struct PaymentInitiatedDomainEvent {
    /// イベントメタデータ
    pub metadata: Option<EventMetadata>,
    /// 決済ID
    pub payment_id: String,
    /// 注文ID
    pub order_id: String,
    /// 顧客ID
    pub customer_id: String,
    /// 金額（最小通貨単位）
    pub amount: i64,
    /// 通貨コード
    pub currency: String,
    /// 決済方法
    pub payment_method: String,
    /// 開始日時
    pub initiated_at: Option<DateTime<Utc>>,
}

/// 決済完了ドメインイベント
#[derive(Debug, Clone)]
pub struct PaymentCompletedDomainEvent {
    /// イベントメタデータ
    pub metadata: Option<EventMetadata>,
    /// 決済ID
    pub payment_id: String,
    /// 注文ID
    pub order_id: String,
    /// 金額（最小通貨単位）
    pub amount: i64,
    /// 通貨コード
    pub currency: String,
    /// トランザクションID
    pub transaction_id: String,
    /// 完了日時
    pub completed_at: Option<DateTime<Utc>>,
}

/// 決済失敗ドメインイベント
#[derive(Debug, Clone)]
pub struct PaymentFailedDomainEvent {
    /// イベントメタデータ
    pub metadata: Option<EventMetadata>,
    /// 決済ID
    pub payment_id: String,
    /// 注文ID
    pub order_id: String,
    /// 失敗理由
    pub reason: String,
    /// エラーコード
    pub error_code: String,
    /// 失敗日時
    pub failed_at: Option<DateTime<Utc>>,
}

/// 返金ドメインイベント
#[derive(Debug, Clone)]
pub struct PaymentRefundedDomainEvent {
    /// イベントメタデータ
    pub metadata: Option<EventMetadata>,
    /// 決済ID
    pub payment_id: String,
    /// 注文ID
    pub order_id: String,
    /// 返金金額（最小通貨単位）
    pub refund_amount: i64,
    /// 通貨コード
    pub currency: String,
    /// 返金理由
    pub reason: String,
    /// 返金日時
    pub refunded_at: Option<DateTime<Utc>>,
}

/// ドメインイベントのデフォルト値を生成するヘルパー（テスト用）
impl Default for PaymentInitiatedDomainEvent {
    fn default() -> Self {
        Self {
            metadata: None,
            payment_id: String::new(),
            order_id: String::new(),
            customer_id: String::new(),
            amount: 0,
            currency: String::new(),
            payment_method: String::new(),
            initiated_at: None,
        }
    }
}

/// ドメインイベントのデフォルト値を生成するヘルパー（テスト用）
impl Default for PaymentCompletedDomainEvent {
    fn default() -> Self {
        Self {
            metadata: None,
            payment_id: String::new(),
            order_id: String::new(),
            amount: 0,
            currency: String::new(),
            transaction_id: String::new(),
            completed_at: None,
        }
    }
}

/// ドメインイベントのデフォルト値を生成するヘルパー（テスト用）
impl Default for PaymentFailedDomainEvent {
    fn default() -> Self {
        Self {
            metadata: None,
            payment_id: String::new(),
            order_id: String::new(),
            reason: String::new(),
            error_code: String::new(),
            failed_at: None,
        }
    }
}

/// ドメインイベントのデフォルト値を生成するヘルパー（テスト用）
impl Default for PaymentRefundedDomainEvent {
    fn default() -> Self {
        Self {
            metadata: None,
            payment_id: String::new(),
            order_id: String::new(),
            refund_amount: 0,
            currency: String::new(),
            reason: String::new(),
            refunded_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_initiated_domain_event_default() {
        // デフォルト値でドメインイベントが正しく生成されることを検証
        let event = PaymentInitiatedDomainEvent::default();
        assert!(event.payment_id.is_empty());
        assert_eq!(event.amount, 0);
    }

    #[test]
    fn test_payment_completed_domain_event_default() {
        // デフォルト値でドメインイベントが正しく生成されることを検証
        let event = PaymentCompletedDomainEvent::default();
        assert!(event.transaction_id.is_empty());
    }

    #[test]
    fn test_payment_failed_domain_event_default() {
        // デフォルト値でドメインイベントが正しく生成されることを検証
        let event = PaymentFailedDomainEvent::default();
        assert!(event.reason.is_empty());
    }

    #[test]
    fn test_payment_refunded_domain_event_default() {
        // デフォルト値でドメインイベントが正しく生成されることを検証
        let event = PaymentRefundedDomainEvent::default();
        assert_eq!(event.refund_amount, 0);
    }

    #[test]
    fn test_payment_initiated_domain_event_with_fields() {
        // フィールド指定でドメインイベントが正しく構築されることを検証
        let event = PaymentInitiatedDomainEvent {
            metadata: Some(EventMetadata {
                event_id: "evt-001".to_string(),
                event_type: "payment.initiated".to_string(),
                source: "payment-server".to_string(),
                timestamp: 1700000000000,
                trace_id: "".to_string(),
                correlation_id: "pay-001".to_string(),
                schema_version: 1,
                causation_id: "".to_string(),
            }),
            payment_id: "pay-001".to_string(),
            order_id: "order-001".to_string(),
            customer_id: "cust-001".to_string(),
            amount: 5000,
            currency: "JPY".to_string(),
            payment_method: "credit_card".to_string(),
            initiated_at: None,
        };
        assert_eq!(event.payment_id, "pay-001");
        assert_eq!(event.amount, 5000);
    }
}
