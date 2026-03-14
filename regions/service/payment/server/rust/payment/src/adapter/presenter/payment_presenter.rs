//! Presenter レイヤー — ドメインエンティティを API レスポンス形式に変換する。

use crate::domain::entity::payment::Payment;
use serde::Serialize;

/// 決済詳細 API レスポンス。
#[derive(Debug, Serialize)]
pub struct PaymentDetailResponse {
    pub id: String,
    pub order_id: String,
    pub customer_id: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub version: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// 決済一覧 API レスポンス。
#[derive(Debug, Serialize)]
pub struct PaymentListResponse {
    pub payments: Vec<PaymentSummaryResponse>,
    pub total: i64,
}

/// 決済サマリ（一覧表示用）。
#[derive(Debug, Serialize)]
pub struct PaymentSummaryResponse {
    pub id: String,
    pub order_id: String,
    pub customer_id: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl PaymentDetailResponse {
    pub fn from_entity(payment: &Payment) -> Self {
        Self {
            id: payment.id.to_string(),
            order_id: payment.order_id.clone(),
            customer_id: payment.customer_id.clone(),
            amount: payment.amount,
            currency: payment.currency.clone(),
            status: payment.status.as_str().to_string(),
            payment_method: payment.payment_method.clone(),
            transaction_id: payment.transaction_id.clone(),
            error_code: payment.error_code.clone(),
            error_message: payment.error_message.clone(),
            version: payment.version,
            created_at: payment.created_at.to_rfc3339(),
            updated_at: payment.updated_at.to_rfc3339(),
        }
    }
}

impl From<&Payment> for PaymentSummaryResponse {
    fn from(payment: &Payment) -> Self {
        Self {
            id: payment.id.to_string(),
            order_id: payment.order_id.clone(),
            customer_id: payment.customer_id.clone(),
            amount: payment.amount,
            currency: payment.currency.clone(),
            status: payment.status.as_str().to_string(),
            created_at: payment.created_at.to_rfc3339(),
            updated_at: payment.updated_at.to_rfc3339(),
        }
    }
}

impl PaymentListResponse {
    pub fn from_entities(payments: &[Payment], total: i64) -> Self {
        Self {
            payments: payments.iter().map(PaymentSummaryResponse::from).collect(),
            total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::payment::PaymentStatus;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_payment() -> Payment {
        Payment {
            id: Uuid::new_v4(),
            order_id: "ORD-001".to_string(),
            customer_id: "CUST-001".to_string(),
            amount: 5000,
            currency: "JPY".to_string(),
            status: PaymentStatus::Initiated,
            payment_method: Some("credit_card".to_string()),
            transaction_id: None,
            error_code: None,
            error_message: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_payment_detail_response() {
        let payment = sample_payment();
        let resp = PaymentDetailResponse::from_entity(&payment);
        assert_eq!(resp.order_id, "ORD-001");
        assert_eq!(resp.status, "initiated");
        assert_eq!(resp.amount, 5000);
    }

    #[test]
    fn test_payment_list_response() {
        let payments = vec![sample_payment()];
        let resp = PaymentListResponse::from_entities(&payments, 1);
        assert_eq!(resp.total, 1);
        assert_eq!(resp.payments.len(), 1);
        assert_eq!(resp.payments[0].status, "initiated");
    }

    #[test]
    fn test_payment_summary_response() {
        let payment = sample_payment();
        let resp = PaymentSummaryResponse::from(&payment);
        assert_eq!(resp.customer_id, "CUST-001");
        assert_eq!(resp.amount, 5000);
    }
}
