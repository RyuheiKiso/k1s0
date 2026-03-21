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
        if topic == self.payment_completed_topic {
            // payment.completed: 注文を confirmed ステータスに遷移させる
            let event = PaymentCompletedEvent::decode(payload)
                .context("PaymentCompletedEvent のデシリアライズに失敗")?;
            info!(order_id = %event.order_id, "payment completed received, confirming order");
            self.handle_payment_event_uc
                .handle_completed(&event.order_id)
                .await
                .context("payment completed 処理に失敗")?;
        } else if topic == self.payment_failed_topic {
            // payment.failed: 注文を cancelled に遷移させ、在庫解放の補償トランザクションを起動
            let event = PaymentFailedEvent::decode(payload)
                .context("PaymentFailedEvent のデシリアライズに失敗")?;
            warn!(
                order_id = %event.order_id,
                reason = %event.reason,
                "payment failed received, cancelling order for saga compensation"
            );
            self.handle_payment_event_uc
                .handle_failed(&event.order_id, &event.reason)
                .await
                .context("payment failed 処理に失敗")?;
        } else {
            warn!(topic, "order consumer: unknown topic, skipping");
        }
        Ok(())
    }
}
