// Saga 正常/補償: Order イベントを受信して在庫操作を行う Kafka Consumer（C-001）。
// order.created → ReserveStockUseCase を呼び出して在庫を確保する。
// order.cancelled → ReleaseStockUseCase を呼び出して在庫を解放する（補償トランザクション）。

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

/// Inventory Kafka Consumer。
/// order イベントトピックを購読し、在庫を Saga に従い操作する。
pub struct InventoryKafkaConsumer {
    consumer: StreamConsumer,
    handle_order_event_uc: Arc<HandleOrderEventUseCase>,
    order_created_topic: String,
    order_cancelled_topic: String,
}

impl InventoryKafkaConsumer {
    /// 新しい InventoryKafkaConsumer を生成する。
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
            // サービス再起動後は最古の未処理オフセットから再開し、在庫操作を取りこぼさない
            .set("auto.offset.reset", "earliest")
            .create()
            .context("Inventory Kafka consumer の生成に失敗")?;

        let topics = [
            kafka_cfg.order_created_topic.as_str(),
            kafka_cfg.order_cancelled_topic.as_str(),
        ];
        consumer
            .subscribe(&topics)
            .context("order イベントトピックへのサブスクライブに失敗")?;

        info!(
            topics = ?topics,
            group_id = %kafka_cfg.consumer_group_id,
            "inventory kafka consumer subscribed"
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
        info!("inventory kafka consumer started");
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("inventory kafka consumer shutting down");
                        break;
                    }
                }
                result = self.consumer.recv() => {
                    match result {
                        Err(e) => warn!("inventory kafka consumer recv error: {}", e),
                        Ok(msg) => {
                            let payload = msg.payload().unwrap_or_default();
                            let topic = msg.topic();
                            if let Err(e) = self.process_message(topic, payload).await {
                                warn!(topic, "inventory kafka consumer message processing failed: {}", e);
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
            // order.created: 注文明細ごとに在庫を確保する（Saga 正常フロー）
            let event = OrderCreatedEvent::decode(payload)
                .context("OrderCreatedEvent のデシリアライズに失敗")?;
            info!(order_id = %event.order_id, "order created received, reserving stock");
            self.handle_order_event_uc
                .handle_created(&event)
                .await
                .context("order created 処理に失敗")?;
        } else if topic == self.order_cancelled_topic {
            // order.cancelled: 在庫を解放する（Saga 補償トランザクション）
            let event = OrderCancelledEvent::decode(payload)
                .context("OrderCancelledEvent のデシリアライズに失敗")?;
            warn!(
                order_id = %event.order_id,
                reason = %event.reason,
                "order cancelled received, releasing stock for saga compensation"
            );
            self.handle_order_event_uc
                .handle_cancelled(&event)
                .await
                .context("order cancelled 処理に失敗")?;
        } else {
            warn!(topic, "inventory consumer: unknown topic, skipping");
        }
        Ok(())
    }
}
