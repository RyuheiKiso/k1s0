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
        if topic == self.order_created_topic {
            // order.created: 決済を開始する（Saga 正常フロー）
            let event = OrderCreatedEvent::decode(payload)
                .context("OrderCreatedEvent のデシリアライズに失敗")?;
            info!(order_id = %event.order_id, "order created received, initiating payment");
            self.handle_order_event_uc
                .handle_created(&event)
                .await
                .context("order created 決済開始処理に失敗")?;
        } else {
            warn!(topic, "payment consumer: unknown topic, skipping");
        }
        Ok(())
    }
}
