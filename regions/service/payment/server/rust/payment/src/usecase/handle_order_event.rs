// Saga: Order イベントに応じて決済操作を行うユースケース（C-001 / M-20）。
// order.created → InitiatePaymentUseCase を呼び出して決済レコードを作成する。
// order.cancelled → FailPaymentUseCase を呼び出して進行中の決済を失敗ステータスに遷移させる。
//
// 冪等性: InitiatePaymentUseCase は同一 order_id に対する重複リクエストを検出し、
// 既存の決済レコードを返すことで二重課金を防ぐ（InitiatePaymentUseCase 参照）。
//
// Saga 補償フロー（order.cancelled）:
// 注文がキャンセルされた場合、対応する決済が Initiated 状態であれば Failed に遷移させる。
// Completed/Refunded など終端ステータスの決済は変更しない（無視して Ok を返す）。

use crate::domain::entity::payment::InitiatePayment;
use crate::domain::entity::payment::PaymentStatus;
use crate::usecase::fail_payment::FailPaymentUseCase;
use crate::usecase::initiate_payment::InitiatePaymentUseCase;
use crate::domain::repository::payment_repository::PaymentRepository;
use crate::proto::k1s0::event::service::order::v1::{OrderCancelledEvent, OrderCreatedEvent};
use std::sync::Arc;
use tracing::{info, warn};

/// HandleOrderEventUseCase は order イベントに応じた決済操作を担う。
pub struct HandleOrderEventUseCase {
    /// 注文作成イベントに応じて決済を開始するユースケース
    initiate_payment_uc: Arc<InitiatePaymentUseCase>,
    /// 注文キャンセルイベントに応じて決済を失敗ステータスに遷移させるユースケース
    fail_payment_uc: Arc<FailPaymentUseCase>,
    /// 注文 ID から決済を検索するリポジトリ（キャンセル時の決済特定に使用）
    payment_repo: Arc<dyn PaymentRepository>,
}

impl HandleOrderEventUseCase {
    pub fn new(
        initiate_payment_uc: Arc<InitiatePaymentUseCase>,
        fail_payment_uc: Arc<FailPaymentUseCase>,
        payment_repo: Arc<dyn PaymentRepository>,
    ) -> Self {
        Self {
            initiate_payment_uc,
            fail_payment_uc,
            payment_repo,
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

    /// order.cancelled イベントに応じて進行中の決済を失敗ステータスに遷移させる（M-20）。
    /// Saga 補償フロー: 注文キャンセル → 決済中断。
    ///
    /// 冪等性:
    /// - 対応する決済が存在しない場合は Ok を返す（決済開始前にキャンセルされたケース）。
    /// - Initiated 以外のステータス（Completed/Failed/Refunded）の決済は変更しない。
    ///   FailPaymentUseCase がステータス遷移バリデーションエラーを返す場合は警告ログを出して無視する。
    pub async fn handle_cancelled(&self, event: &OrderCancelledEvent) -> anyhow::Result<()> {
        // 注文 ID から対応する決済を検索する
        let payment = match self.payment_repo.find_by_order_id(&event.order_id).await? {
            Some(p) => p,
            None => {
                // 決済が存在しない場合は補償不要（注文作成前にキャンセルされたケース）
                info!(
                    order_id = %event.order_id,
                    "order.cancelled received but no payment found, skipping"
                );
                return Ok(());
            }
        };

        // Initiated 状態でない決済は遷移不可のため警告ログを出してスキップする
        if payment.status != PaymentStatus::Initiated {
            warn!(
                order_id = %event.order_id,
                payment_id = %payment.id,
                status = %payment.status,
                "order.cancelled received but payment is not in Initiated state, skipping"
            );
            return Ok(());
        }

        // Initiated → Failed に遷移させる（Saga 補償）
        self.fail_payment_uc
            .execute(payment.id, "ORDER_CANCELLED", &event.reason)
            .await?;

        info!(
            order_id = %event.order_id,
            payment_id = %payment.id,
            reason = %event.reason,
            "payment failed by saga order cancellation"
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
