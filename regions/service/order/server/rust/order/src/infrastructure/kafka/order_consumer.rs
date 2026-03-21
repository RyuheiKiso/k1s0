// Saga 補償: Payment イベントを受信して注文ステータスを更新する Kafka Consumer（C-001）。
// payment.completed → 注文を confirmed に遷移させ、Saga を正常完了させる。
// payment.failed → 注文を cancelled に遷移させ、在庫解放の補償トランザクションを起動する。

use crate::domain::entity::order::OrderStatus;
use crate::infrastructure::config::KafkaConfig;
use crate::usecase::handle_payment_event::HandlePaymentEventUseCase;
use crate::proto::k1s0::event::service::payment::v1::{
    PaymentCompletedEvent, PaymentFailedEvent,
};
use anyhow::Context;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message as KafkaMessage;
use std::sync::Arc;
use tracing::{info, warn};

/// Order Kafka Consumer。
/// payment イベントトピックを購読し、注文ステータスを Saga に従い更新する。
pub struct OrderKafkaConsumer {
    consumer: StreamConsumer,
    handle_payment_event_uc: Arc<HandlePaymentEventUseCase>,
    payment_completed_topic: String,
    payment_failed_topic: String,
}

impl OrderKafkaConsumer {
    /// 新しい OrderKafkaConsumer を生成する。
    pub fn new(
        kafka_cfg: &KafkaConfig,
        handle_payment_event_uc: Arc<HandlePaymentEventUseCase>,
    ) -> anyhow::Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", kafka_cfg.brokers.join(","))
            .set("group.id", &kafka_cfg.consumer_group_id)
            .set("security.protocol", &kafka_cfg.security_protocol)
            .set("enable.auto.commit", "false")
            .set("session.timeout.ms", "30000")
            // サービス再起動後は最古の未処理オフセットから再開し、イベントを取りこぼさない
            .set("auto.offset.reset", "earliest")
            .create()
            .context("Order Kafka consumer の生成に失敗")?;

        let topics = [
            kafka_cfg.payment_completed_topic.as_str(),
            kafka_cfg.payment_failed_topic.as_str(),
        ];
        consumer
            .subscribe(&topics)
            .context("payment イベントトピックへのサブスクライブに失敗")?;

        info!(
            topics = ?topics,
            group_id = %kafka_cfg.consumer_group_id,
            "order kafka consumer subscribed"
        );

        Ok(Self {
            consumer,
            handle_payment_event_uc,
            payment_completed_topic: kafka_cfg.payment_completed_topic.clone(),
            payment_failed_topic: kafka_cfg.payment_failed_topic.clone(),
        })
    }

    /// Kafka メッセージをポーリングして処理する。shutdown_rx が true になると停止する。
    pub async fn run(self, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) {
        info!("order kafka consumer started");
        loop {
            tokio::select! {
                // シャットダウンシグナル受信時はループを抜ける
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("order kafka consumer shutting down");
                        break;
                    }
                }
                result = self.consumer.recv() => {
                    match result {
                        Err(e) => warn!("order kafka consumer recv error: {}", e),
                        Ok(msg) => {
                            let payload = msg.payload().unwrap_or_default();
                            let topic = msg.topic();
                            if let Err(e) = self.process_message(topic, payload).await {
                                warn!(topic, "order kafka consumer message processing failed: {}", e);
                                // エラーでもオフセットをコミットしてスキップ（DLQ 移送は将来対応）
                            }
                            // 手動オフセットコミット（at-least-once 保証）
                            if let Err(e) = self.consumer.store_offset_from_message(&msg) {
                                warn!("offset store failed: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    /// 受信したメッセージをトピックに応じてディスパッチする。
    async fn process_message(&self, topic: &str, payload: &[u8]) -> anyhow::Result<()> {
        dispatch_message(
            &self.handle_payment_event_uc,
            &self.payment_completed_topic,
            &self.payment_failed_topic,
            topic,
            payload,
        )
        .await
    }
}

/// topic と payload を受け取ってディスパッチするコアロジック。
/// テスタビリティのため OrderKafkaConsumer 構造体から分離する。
async fn dispatch_message(
    uc: &HandlePaymentEventUseCase,
    payment_completed_topic: &str,
    payment_failed_topic: &str,
    topic: &str,
    payload: &[u8],
) -> anyhow::Result<()> {
    if topic == payment_completed_topic {
        // payment.completed: 注文を confirmed ステータスに遷移させる
        let event = PaymentCompletedEvent::decode(payload)
            .context("PaymentCompletedEvent のデシリアライズに失敗")?;
        info!(order_id = %event.order_id, "payment completed received, confirming order");
        uc.handle_completed(&event.order_id)
            .await
            .context("payment completed 処理に失敗")?;
    } else if topic == payment_failed_topic {
        // payment.failed: 注文を cancelled に遷移させ、在庫解放の補償トランザクションを起動
        let event = PaymentFailedEvent::decode(payload)
            .context("PaymentFailedEvent のデシリアライズに失敗")?;
        warn!(
            order_id = %event.order_id,
            reason = %event.reason,
            "payment failed received, cancelling order for saga compensation"
        );
        uc.handle_failed(&event.order_id, &event.reason)
            .await
            .context("payment failed 処理に失敗")?;
    } else {
        warn!(topic, "order consumer: unknown topic, skipping");
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::order::{Order, OrderStatus};
    use crate::domain::repository::order_repository::MockOrderRepository;
    use chrono::Utc;
    use prost::Message;
    use uuid::Uuid;

    // テスト用の注文エンティティを生成するヘルパー
    fn sample_order(id: Uuid, status: OrderStatus) -> Order {
        Order {
            id,
            customer_id: "CUST-001".to_string(),
            status,
            total_amount: 5000,
            currency: "JPY".to_string(),
            notes: None,
            created_by: "admin".to_string(),
            updated_by: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // テスト用の PaymentCompletedEvent を Protobuf バイト列にエンコードするヘルパー
    fn encode_payment_completed(order_id: &str) -> Vec<u8> {
        let event = PaymentCompletedEvent {
            metadata: None,
            payment_id: "PAY-001".to_string(),
            order_id: order_id.to_string(),
            amount: 5000,
            currency: "JPY".to_string(),
            transaction_id: "TXN-001".to_string(),
            completed_at: None,
        };
        event.encode_to_vec()
    }

    // テスト用の PaymentFailedEvent を Protobuf バイト列にエンコードするヘルパー
    fn encode_payment_failed(order_id: &str) -> Vec<u8> {
        let event = PaymentFailedEvent {
            metadata: None,
            payment_id: "PAY-001".to_string(),
            order_id: order_id.to_string(),
            reason: "insufficient funds".to_string(),
            error_code: "INSUFFICIENT_FUNDS".to_string(),
            failed_at: None,
        };
        event.encode_to_vec()
    }

    // payment.completed を受信して注文が Confirmed に遷移することを確認する
    #[tokio::test]
    async fn test_dispatch_payment_completed_confirms_order() {
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
        let payload = encode_payment_completed(&order_id.to_string());
        let result = dispatch_message(&uc, "payment.completed", "payment.failed", "payment.completed", &payload).await;
        assert!(result.is_ok());
    }

    // payment.failed を受信して注文が Cancelled に遷移することを確認する
    #[tokio::test]
    async fn test_dispatch_payment_failed_cancels_order() {
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
        let payload = encode_payment_failed(&order_id.to_string());
        let result = dispatch_message(&uc, "payment.completed", "payment.failed", "payment.failed", &payload).await;
        assert!(result.is_ok());
    }

    // 未知のトピックを受信した場合はスキップして Ok を返すことを確認する
    #[tokio::test]
    async fn test_dispatch_unknown_topic_returns_ok() {
        let mock_repo = MockOrderRepository::new();
        let uc = HandlePaymentEventUseCase::new(Arc::new(mock_repo));
        let result = dispatch_message(&uc, "payment.completed", "payment.failed", "unknown.topic", b"dummy").await;
        assert!(result.is_ok());
    }

    // payment.completed トピックに不正なバイト列を受信した場合はデシリアライズエラーを返すことを確認する
    #[tokio::test]
    async fn test_dispatch_payment_completed_invalid_protobuf() {
        let mock_repo = MockOrderRepository::new();
        let uc = HandlePaymentEventUseCase::new(Arc::new(mock_repo));
        let result = dispatch_message(&uc, "payment.completed", "payment.failed", "payment.completed", b"\xff\xfe").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("デシリアライズに失敗"));
    }

    // payment.failed トピックに不正なバイト列を受信した場合はデシリアライズエラーを返すことを確認する
    #[tokio::test]
    async fn test_dispatch_payment_failed_invalid_protobuf() {
        let mock_repo = MockOrderRepository::new();
        let uc = HandlePaymentEventUseCase::new(Arc::new(mock_repo));
        let result = dispatch_message(&uc, "payment.completed", "payment.failed", "payment.failed", b"\xff\xfe").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("デシリアライズに失敗"));
    }

    // UseCase がエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_dispatch_payment_completed_usecase_error_propagates() {
        let order_id = Uuid::new_v4();
        let mut mock_repo = MockOrderRepository::new();
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("DB接続エラー")));
        mock_repo.expect_update_status().times(0);

        let uc = HandlePaymentEventUseCase::new(Arc::new(mock_repo));
        let payload = encode_payment_completed(&order_id.to_string());
        let result = dispatch_message(&uc, "payment.completed", "payment.failed", "payment.completed", &payload).await;
        assert!(result.is_err());
    }

    // 空の payload を受信した場合、Protobuf のデフォルト値（空 order_id）となり UUID 解析エラーになることを確認する
    #[tokio::test]
    async fn test_dispatch_empty_payload() {
        let mock_repo = MockOrderRepository::new();
        let uc = HandlePaymentEventUseCase::new(Arc::new(mock_repo));
        // 空バイト列は Protobuf デコード成功（全フィールドがデフォルト値）→ order_id = "" → UUID 解析失敗
        let result = dispatch_message(&uc, "payment.completed", "payment.failed", "payment.completed", &[]).await;
        assert!(result.is_err());
    }
}
