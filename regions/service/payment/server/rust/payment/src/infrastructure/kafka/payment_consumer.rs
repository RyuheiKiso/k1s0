// Saga: Order イベントを受信して決済操作を行う Kafka Consumer（C-001 / M-20）。
// order.created → InitiatePaymentUseCase を呼び出して決済レコードを作成する。
// order.cancelled → FailPaymentUseCase を呼び出して進行中の決済を中断する（M-20）。
//
// 設計注: Choreography-based Saga パターンを採用。
// inventory と payment は共に order.created を購読して並行して処理を開始する。
// 決済失敗 → 注文キャンセル → 在庫解放、という補償チェーンが確立される。
// また、注文キャンセル → 決済中断、という補償チェーンも同 Consumer で処理する。

use crate::infrastructure::config::KafkaConfig;
use crate::usecase::handle_order_event::HandleOrderEventUseCase;
use crate::proto::k1s0::event::service::order::v1::{OrderCancelledEvent, OrderCreatedEvent};
use anyhow::Context;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message as KafkaMessage;
use std::sync::Arc;
use tracing::{info, warn};

/// Payment Kafka Consumer。
/// order.created および order.cancelled トピックを購読し、決済を開始・中断する。
pub struct PaymentKafkaConsumer {
    consumer: StreamConsumer,
    handle_order_event_uc: Arc<HandleOrderEventUseCase>,
    /// order.created トピック名
    order_created_topic: String,
    /// order.cancelled トピック名（M-20: 注文キャンセル時の決済中断）
    order_cancelled_topic: String,
}

impl PaymentKafkaConsumer {
    /// 新しい PaymentKafkaConsumer を生成する。
    /// order.created と order.cancelled の両トピックを購読する。
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
            // サービス再起動後は最古の未処理オフセットから再開し、決済開始・中断を取りこぼさない
            .set("auto.offset.reset", "earliest")
            .create()
            .context("Payment Kafka consumer の生成に失敗")?;

        // order.created と order.cancelled の両トピックを購読する（M-20）
        let topics = [
            kafka_cfg.order_created_topic.as_str(),
            kafka_cfg.order_cancelled_topic.as_str(),
        ];
        consumer
            .subscribe(&topics)
            .context("order トピックへのサブスクライブに失敗")?;

        info!(
            topics = ?topics,
            group_id = %kafka_cfg.consumer_group_id,
            "payment kafka consumer subscribed"
        );

        Ok(Self {
            consumer,
            handle_order_event_uc,
            order_created_topic: kafka_cfg.order_created_topic.clone(),
            order_cancelled_topic: kafka_cfg.order_cancelled_topic.clone(),
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
            &self.order_cancelled_topic,
            topic,
            payload,
        )
        .await
    }
}

/// topic と payload を受け取ってディスパッチするコアロジック。
/// テスタビリティのため PaymentKafkaConsumer 構造体から分離する。
/// order.created → handle_created / order.cancelled → handle_cancelled（M-20）
async fn dispatch_message(
    uc: &HandleOrderEventUseCase,
    order_created_topic: &str,
    order_cancelled_topic: &str,
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
    } else if topic == order_cancelled_topic {
        // order.cancelled: 進行中の決済を中断する（Saga 補償フロー / M-20）
        let event = OrderCancelledEvent::decode(payload)
            .context("OrderCancelledEvent のデシリアライズに失敗")?;
        info!(order_id = %event.order_id, reason = %event.reason, "order cancelled received, failing payment");
        uc.handle_cancelled(&event)
            .await
            .context("order cancelled 決済中断処理に失敗")?;
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
    use crate::usecase::fail_payment::FailPaymentUseCase;
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
    // initiate 用、fail 用、search 用の 3 つのモックリポジトリを分けて渡す
    fn make_uc(
        initiate_repo: MockPaymentRepository,
        fail_repo: MockPaymentRepository,
        search_repo: MockPaymentRepository,
    ) -> HandleOrderEventUseCase {
        let initiate_uc = Arc::new(InitiatePaymentUseCase::new(Arc::new(initiate_repo)));
        let fail_uc = Arc::new(FailPaymentUseCase::new(Arc::new(fail_repo)));
        HandleOrderEventUseCase::new(initiate_uc, fail_uc, Arc::new(search_repo))
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

    // テスト用の OrderCancelledEvent を Protobuf バイト列にエンコードするヘルパー
    fn encode_order_cancelled(order_id: &str) -> Vec<u8> {
        let event = OrderCancelledEvent {
            metadata: None,
            order_id: order_id.to_string(),
            user_id: "USER-001".to_string(),
            reason: "customer request".to_string(),
        };
        event.encode_to_vec()
    }

    // order.created を受信して決済が開始されることを確認する
    #[tokio::test]
    async fn test_dispatch_order_created_initiates_payment() {
        let order_id = "ORD-001";
        let payment = sample_payment(order_id);
        let payment_clone = payment.clone();

        // initiate_uc 用モック: 冪等性チェックなし → create 呼び出し
        let mut initiate_repo = MockPaymentRepository::new();
        initiate_repo
            .expect_find_by_order_id()
            .times(1)
            .returning(|_| Ok(None));
        initiate_repo
            .expect_create()
            .times(1)
            .returning(move |_| Ok(payment_clone.clone()));

        let fail_repo = MockPaymentRepository::new();
        let search_repo = MockPaymentRepository::new();

        let uc = make_uc(initiate_repo, fail_repo, search_repo);
        let payload = encode_order_created(order_id);
        let result = dispatch_message(&uc, "order.created", "order.cancelled", "order.created", &payload).await;
        assert!(result.is_ok());
    }

    // 未知のトピックを受信した場合はスキップして Ok を返すことを確認する
    #[tokio::test]
    async fn test_dispatch_unknown_topic_returns_ok() {
        let uc = make_uc(MockPaymentRepository::new(), MockPaymentRepository::new(), MockPaymentRepository::new());
        let result = dispatch_message(&uc, "order.created", "order.cancelled", "unknown.topic", b"dummy").await;
        assert!(result.is_ok());
    }

    // order.created トピックに不正なバイト列を受信した場合はデシリアライズエラーを返すことを確認する
    #[tokio::test]
    async fn test_dispatch_order_created_invalid_protobuf() {
        let uc = make_uc(MockPaymentRepository::new(), MockPaymentRepository::new(), MockPaymentRepository::new());
        let result = dispatch_message(&uc, "order.created", "order.cancelled", "order.created", b"\xff\xfe").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("デシリアライズに失敗"));
    }

    // UseCase がエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_dispatch_order_created_usecase_error_propagates() {
        let mut initiate_repo = MockPaymentRepository::new();
        initiate_repo
            .expect_find_by_order_id()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("DB接続エラー")));

        let uc = make_uc(initiate_repo, MockPaymentRepository::new(), MockPaymentRepository::new());
        let payload = encode_order_created("ORD-001");
        let result = dispatch_message(&uc, "order.created", "order.cancelled", "order.created", &payload).await;
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
