// Saga 補償: Payment イベントに応じて注文ステータスを更新するユースケース（C-001）。
// payment.completed → OrderStatus::Confirmed に遷移し、Saga を正常完了させる。
// payment.failed → OrderStatus::Cancelled に遷移し、在庫解放の補償フローを開始させる。
//
// 冪等性: UpdateOrderStatusUseCase 内のステータス遷移バリデーションにより、
// 既に同じステータスへの遷移が実行済みの場合は OrderError::InvalidStatusTransition となる。
// これを "既に処理済み" として扱いスキップすることで重複処理を防ぐ。

use crate::domain::entity::order::OrderStatus;
use crate::domain::error::OrderError;
use crate::domain::repository::order_repository::OrderRepository;
use crate::usecase::update_order_status::UpdateOrderStatusUseCase;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// HandlePaymentEventUseCase は payment イベントに応じた注文ステータス更新を担う。
pub struct HandlePaymentEventUseCase {
    update_order_status_uc: Arc<UpdateOrderStatusUseCase>,
}

impl HandlePaymentEventUseCase {
    pub fn new(order_repo: Arc<dyn OrderRepository>) -> Self {
        Self {
            update_order_status_uc: Arc::new(UpdateOrderStatusUseCase::new(order_repo)),
        }
    }

    /// payment.completed イベントに応じて注文を Confirmed ステータスに遷移させる。
    pub async fn handle_completed(&self, order_id: &str) -> anyhow::Result<()> {
        let id = Uuid::parse_str(order_id)
            .map_err(|_| anyhow::anyhow!("invalid order_id format: {}", order_id))?;

        match self
            .update_order_status_uc
            .execute(id, &OrderStatus::Confirmed, "saga-payment-consumer")
            .await
        {
            Ok(_) => {
                info!(order_id, "order confirmed by saga payment completion");
                Ok(())
            }
            Err(e) => {
                // 冪等性: 既に Confirmed または他の終端ステータスの場合は InvalidStatusTransition になる。
                // これは重複メッセージ処理であり、エラーとして扱わない。
                if e.downcast_ref::<OrderError>()
                    .map(|oe| matches!(oe, OrderError::InvalidStatusTransition { .. }))
                    .unwrap_or(false)
                {
                    info!(order_id, "order already in terminal status, skipping (idempotent)");
                    return Ok(());
                }
                Err(e)
            }
        }
    }

    /// payment.failed イベントに応じて注文を Cancelled ステータスに遷移させる。
    /// これにより inventory.cancelled イベントが Outbox 経由で発行され、
    /// 在庫解放の補償トランザクションが起動される。
    pub async fn handle_failed(&self, order_id: &str, reason: &str) -> anyhow::Result<()> {
        let id = Uuid::parse_str(order_id)
            .map_err(|_| anyhow::anyhow!("invalid order_id format: {}", order_id))?;

        match self
            .update_order_status_uc
            .execute(id, &OrderStatus::Cancelled, "saga-payment-consumer")
            .await
        {
            Ok(_) => {
                info!(order_id, reason, "order cancelled by saga payment failure");
                Ok(())
            }
            Err(e) => {
                // 冪等性: 既に Cancelled 等の終端ステータスの場合はスキップ
                if e.downcast_ref::<OrderError>()
                    .map(|oe| matches!(oe, OrderError::InvalidStatusTransition { .. }))
                    .unwrap_or(false)
                {
                    info!(order_id, "order already in terminal status, skipping (idempotent)");
                    return Ok(());
                }
                Err(e)
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::order::Order;
    use crate::domain::repository::order_repository::MockOrderRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_order(id: Uuid, status: OrderStatus) -> Order {
        Order {
            id,
            customer_id: "CUST-001".to_string(),
            status,
            total_amount: 2000,
            currency: "JPY".to_string(),
            notes: None,
            created_by: "admin".to_string(),
            updated_by: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // payment.completed を受信して注文が Confirmed に遷移することを確認する
    #[tokio::test]
    async fn test_handle_completed_confirms_order() {
        let order_id = Uuid::new_v4();
        let order = sample_order(order_id, OrderStatus::Pending);
        let mut confirmed = order.clone();
        confirmed.status = OrderStatus::Confirmed;
        let order_clone = order.clone();
        let confirmed_clone = confirmed.clone();

        let mut mock_repo = MockOrderRepository::new();
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(order_clone.clone())));
        mock_repo
            .expect_update_status()
            .times(1)
            .returning(move |_, _, _, _| Ok(confirmed_clone.clone()));

        let uc = HandlePaymentEventUseCase::new(Arc::new(mock_repo));
        let result = uc.handle_completed(&order_id.to_string()).await;
        assert!(result.is_ok());
    }

    // payment.failed を受信して注文が Cancelled に遷移することを確認する
    #[tokio::test]
    async fn test_handle_failed_cancels_order() {
        let order_id = Uuid::new_v4();
        let order = sample_order(order_id, OrderStatus::Pending);
        let mut cancelled = order.clone();
        cancelled.status = OrderStatus::Cancelled;
        let order_clone = order.clone();
        let cancelled_clone = cancelled.clone();

        let mut mock_repo = MockOrderRepository::new();
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(order_clone.clone())));
        mock_repo
            .expect_update_status()
            .times(1)
            .returning(move |_, _, _, _| Ok(cancelled_clone.clone()));

        let uc = HandlePaymentEventUseCase::new(Arc::new(mock_repo));
        let result = uc.handle_failed(&order_id.to_string(), "insufficient funds").await;
        assert!(result.is_ok());
    }

    // 既に終端ステータスの注文に対して冪等処理が機能することを確認する
    #[tokio::test]
    async fn test_handle_completed_idempotent_when_already_confirmed() {
        let order_id = Uuid::new_v4();
        // 既に Confirmed の注文（再処理ケース）
        let order = sample_order(order_id, OrderStatus::Confirmed);
        let order_clone = order.clone();

        let mut mock_repo = MockOrderRepository::new();
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(order_clone.clone())));
        // update_status は呼ばれない（InvalidStatusTransition でスキップ）
        mock_repo.expect_update_status().times(0);

        let uc = HandlePaymentEventUseCase::new(Arc::new(mock_repo));
        let result = uc.handle_completed(&order_id.to_string()).await;
        // 冪等処理: エラーにならずに Ok が返る
        assert!(result.is_ok());
    }
}
