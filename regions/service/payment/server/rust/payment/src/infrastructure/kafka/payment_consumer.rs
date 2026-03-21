// Saga: Order 作成イベントを受信して決済を開始する Kafka Consumer（C-001）。
// order.created → InitiatePaymentUseCase を呼び出して決済レコードを作成する。
// 決済失敗時は PaymentFailedEvent（Outbox 経由）が order.failed として発行され、
// Order Consumer が注文をキャンセルする補償フローが起動する。
//
// 設計注: Choreography-based Saga パターンを採用。
// inventory と payment は共に order.created を購読して並行して処理を開始する。
// 決済失敗 → 注文キャンセル → 在庫解放、という補償チェーンが確立される。

use crate::infrastructure::config::KafkaConfig;
use crate::usecase::handle_order_event::HandleOrderEventUseCase;
use crate::proto::k1s0::event::service::order::v1::OrderCreatedEvent;
use anyhow::Context;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message as KafkaMessage;
use std::sync::Arc;
use tracing::{info, warn};

/// Payment Kafka Consumer。
/// order.created トピックを購読し、決済を開始する。
pub struct PaymentKafkaConsumer {
    consumer: StreamConsumer,
    handle_order_event_uc: Arc<HandleOrderEventUseCase>,
    order_created_topic: String,
}

impl PaymentKafkaConsumer {
    /// 新しい PaymentKafkaConsumer を生成する。
    pub fn new(
        kafka_cfg: &KafkaConfig,
        handle_order_event_uc: Arc<HandleOrderEventUseCase>,
    ) -> anyhow::Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", kafka_cfg.brokers.join(","))
            .set("group.id", &kafka_cfg.consumer_group_id)
            .set("security.protocol", &kafka_cfg.security_protocol)
            .set("enable.auto.commit", "false")
            .set("session.timeout.ms", "30000")
            // サービス再起動後は最古の未処理オフセットから再開し、決済開始を取りこぼさない
            .set("auto.offset.reset", "earliest")
            .create()
            .context("Payment Kafka consumer の生成に失敗")?;

        let topics = [kafka_cfg.order_created_topic.as_str()];
        consumer
            .subscribe(&topics)
            .context("order.created トピックへのサブスクライブに失敗")?;

        info!(
            topics = ?topics,
            group_id = %kafka_cfg.consumer_group_id,
            "payment kafka consumer subscribed"
        );

        Ok(Self {
            consumer,
            handle_order_event_uc,
            order_created_topic: kafka_cfg.order_created_topic.clone(),
        })
    }

    /// Kafka メッセージをポーリングして処理する。shutdown_rx が true になると停止する。
    pub async fn run(self, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) {
        info!("payment kafka consumer started");
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("payment kafka consumer shutting down");
                        break;
                    }
                }
                result = self.consumer.recv() => {
                    match result {
                        Err(e) => warn!("payment kafka consumer recv error: {}", e),
                        Ok(msg) => {
                            let payload = msg.payload().unwrap_or_default();
                            let topic = msg.topic();
                            if let Err(e) = self.process_message(topic, payload).await {
                                warn!(topic, "payment kafka consumer message processing failed: {}", e);
                            }
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
            &self.handle_order_event_uc,
            &self.order_created_topic,
            topic,
            payload,
        )
        .await
    }
}

/// topic と payload を受け取ってディスパッチするコアロジック。
/// テスタビリティのため PaymentKafkaConsumer 構造体から分離する。
async fn dispatch_message(
    uc: &HandleOrderEventUseCase,
    order_created_topic: &str,
    topic: &str,
    payload: &[u8],
) -> anyhow::Result<()> {
    if topic == order_created_topic {
        // order.created: 決済を開始する（Saga 正常フロー）
        let event = OrderCreatedEvent::decode(payload)
            .context("OrderCreatedEvent のデシリアライズに失敗")?;
        info!(order_id = %event.order_id, "order created received, initiating payment");
        uc.handle_created(&event)
            .await
            .context("order created 決済開始処理に失敗")?;
    } else {
        warn!(topic, "payment consumer: unknown topic, skipping");
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::payment::{Payment, PaymentStatus};
    use crate::domain::repository::payment_repository::MockPaymentRepository;
    use crate::proto::k1s0::event::service::order::v1::OrderItem;
    use crate::usecase::initiate_payment::InitiatePaymentUseCase;
    use chrono::Utc;
    use prost::Message;
    use uuid::Uuid;

    // テスト用の決済エンティティを生成するヘルパー
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

    // テスト用の HandleOrderEventUseCase をモックリポジトリから構築するヘルパー
    fn make_uc(mock_repo: MockPaymentRepository) -> HandleOrderEventUseCase {
        let initiate_uc = Arc::new(InitiatePaymentUseCase::new(Arc::new(mock_repo)));
        HandleOrderEventUseCase::new(initiate_uc)
    }

    // テスト用の OrderCreatedEvent を Protobuf バイト列にエンコードするヘルパー
    fn encode_order_created(order_id: &str) -> Vec<u8> {
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
        event.encode_to_vec()
    }

    // order.created を受信して決済が開始されることを確認する
    #[tokio::test]
    async fn test_dispatch_order_created_initiates_payment() {
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

        let uc = make_uc(mock_repo);
        let payload = encode_order_created(order_id);
        let result = dispatch_message(&uc, "order.created", "order.created", &payload).await;
        assert!(result.is_ok());
    }

    // 未知のトピックを受信した場合はスキップして Ok を返すことを確認する
    #[tokio::test]
    async fn test_dispatch_unknown_topic_returns_ok() {
        let mock_repo = MockPaymentRepository::new();
        let uc = make_uc(mock_repo);
        let result = dispatch_message(&uc, "order.created", "unknown.topic", b"dummy").await;
        assert!(result.is_ok());
    }

    // order.created トピックに不正なバイト列を受信した場合はデシリアライズエラーを返すことを確認する
    #[tokio::test]
    async fn test_dispatch_order_created_invalid_protobuf() {
        let mock_repo = MockPaymentRepository::new();
        let uc = make_uc(mock_repo);
        let result = dispatch_message(&uc, "order.created", "order.created", b"\xff\xfe").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("デシリアライズに失敗"));
    }

    // UseCase がエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_dispatch_order_created_usecase_error_propagates() {
        let mut mock_repo = MockPaymentRepository::new();
        mock_repo
            .expect_find_by_order_id()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("DB接続エラー")));

        let uc = make_uc(mock_repo);
        let payload = encode_order_created("ORD-001");
        let result = dispatch_message(&uc, "order.created", "order.created", &payload).await;
        assert!(result.is_err());
    }

    // 空の payload を受信した場合、Protobuf のデフォルト値（空フィールド）で決済バリデーションが失敗することを確認する
    #[tokio::test]
    async fn test_dispatch_empty_payload_order_created() {
        // 空バイト列は Protobuf デコード成功（全フィールドがデフォルト値）
        // order_id = "" などが domain バリデーションで ValidationFailed になることを確認する
        let mock_repo = MockPaymentRepository::new();
        let uc = make_uc(mock_repo);
        let result = dispatch_message(&uc, "order.created", "order.created", &[]).await;
        // Protobuf デコードは成功するが、空 order_id が決済ドメインバリデーションで Err になる
        assert!(result.is_err());
    }
}
