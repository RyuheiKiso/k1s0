// Saga: Order 作成イベントに応じて決済を開始するユースケース（C-001）。
// order.created → InitiatePaymentUseCase を呼び出して決済レコードを作成する。
//
// 冪等性: InitiatePaymentUseCase は同一 order_id に対する重複リクエストを検出し、
// 既存の決済レコードを返すことで二重課金を防ぐ（InitiatePaymentUseCase 参照）。

use crate::domain::entity::payment::InitiatePayment;
use crate::usecase::initiate_payment::InitiatePaymentUseCase;
use crate::proto::k1s0::event::service::order::v1::OrderCreatedEvent;
use std::sync::Arc;
use tracing::info;

/// HandleOrderEventUseCase は order イベントに応じた決済操作を担う。
pub struct HandleOrderEventUseCase {
    initiate_payment_uc: Arc<InitiatePaymentUseCase>,
}

impl HandleOrderEventUseCase {
    pub fn new(initiate_payment_uc: Arc<InitiatePaymentUseCase>) -> Self {
        Self {
            initiate_payment_uc,
        }
    }

    /// order.created イベントに応じて決済を開始する。
    /// OrderCreatedEvent から必要な支払い情報（order_id, customer_id, total_amount, currency）を抽出する。
    pub async fn handle_created(&self, event: &OrderCreatedEvent) -> anyhow::Result<()> {
        let input = InitiatePayment {
            order_id: event.order_id.clone(),
            customer_id: event.customer_id.clone(),
            amount: event.total_amount,
            currency: event.currency.clone(),
            // デフォルト決済手段: 注文作成時に指定がない場合は None（後続で選択）
            payment_method: None,
        };

        let payment = self.initiate_payment_uc.execute(&input).await?;
        info!(
            order_id = %event.order_id,
            payment_id = %payment.id,
            amount = payment.amount,
            "payment initiated by saga order consumer"
        );
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::payment::{Payment, PaymentStatus};
    use crate::domain::repository::payment_repository::MockPaymentRepository;
    use crate::proto::k1s0::event::service::order::v1::OrderItem;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_payment(order_id: &str) -> Payment {
        Payment {
            id: Uuid::new_v4(),
            order_id: order_id.to_string(),
            customer_id: "CUST-001".to_string(),
            amount: 5000,
            currency: "JPY".to_string(),
            status: PaymentStatus::Initiated,
            payment_method: None,
            transaction_id: None,
            error_code: None,
            error_message: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // order.created イベントで決済が開始されることを確認する
    #[tokio::test]
    async fn test_handle_created_initiates_payment() {
        let order_id = "ORD-001";
        let payment = sample_payment(order_id);
        let payment_clone = payment.clone();

        let mut mock_repo = MockPaymentRepository::new();
        // 冪等性チェック: 既存決済なし
        mock_repo
            .expect_find_by_order_id()
            .times(1)
            .returning(|_| Ok(None));
        mock_repo
            .expect_create()
            .times(1)
            .returning(move |_| Ok(payment_clone.clone()));

        let initiate_uc = Arc::new(InitiatePaymentUseCase::new(Arc::new(mock_repo)));
        let uc = HandleOrderEventUseCase::new(initiate_uc);

        let event = OrderCreatedEvent {
            metadata: None,
            order_id: order_id.to_string(),
            customer_id: "CUST-001".to_string(),
            items: vec![OrderItem {
                product_id: "PROD-001".to_string(),
                quantity: 5,
                unit_price: 1000,
            }],
            total_amount: 5000,
            currency: "JPY".to_string(),
        };
        let result = uc.handle_created(&event).await;
        assert!(result.is_ok());
    }

    // 同一 order_id への重複リクエストで既存決済が返されることを確認する（冪等性）
    #[tokio::test]
    async fn test_handle_created_idempotent_existing_payment() {
        let order_id = "ORD-002";
        let existing = sample_payment(order_id);
        let existing_clone = existing.clone();

        let mut mock_repo = MockPaymentRepository::new();
        // 冪等性チェック: 既存決済あり → create は呼ばれない
        mock_repo
            .expect_find_by_order_id()
            .times(1)
            .returning(move |_| Ok(Some(existing_clone.clone())));
        mock_repo.expect_create().times(0);

        let initiate_uc = Arc::new(InitiatePaymentUseCase::new(Arc::new(mock_repo)));
        let uc = HandleOrderEventUseCase::new(initiate_uc);

        let event = OrderCreatedEvent {
            metadata: None,
            order_id: order_id.to_string(),
            customer_id: "CUST-001".to_string(),
            items: vec![],
            total_amount: 3000,
            currency: "JPY".to_string(),
        };
        let result = uc.handle_created(&event).await;
        assert!(result.is_ok());
    }
}
