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
/// テスタビリティのため InventoryKafkaConsumer 構造体から分離する。
async fn dispatch_message(
    uc: &HandleOrderEventUseCase,
    order_created_topic: &str,
    order_cancelled_topic: &str,
    topic: &str,
    payload: &[u8],
) -> anyhow::Result<()> {
    if topic == order_created_topic {
        // order.created: 注文明細ごとに在庫を確保する（Saga 正常フロー）
        let event = OrderCreatedEvent::decode(payload)
            .context("OrderCreatedEvent のデシリアライズに失敗")?;
        info!(order_id = %event.order_id, "order created received, reserving stock");
        uc.handle_created(&event)
            .await
            .context("order created 処理に失敗")?;
    } else if topic == order_cancelled_topic {
        // order.cancelled: 在庫を解放する（Saga 補償トランザクション）
        let event = OrderCancelledEvent::decode(payload)
            .context("OrderCancelledEvent のデシリアライズに失敗")?;
        warn!(
            order_id = %event.order_id,
            reason = %event.reason,
            "order cancelled received, releasing stock for saga compensation"
        );
        uc.handle_cancelled(&event)
            .await
            .context("order cancelled 処理に失敗")?;
    } else {
        warn!(topic, "inventory consumer: unknown topic, skipping");
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::inventory_item::InventoryItem;
    use crate::domain::repository::inventory_repository::MockInventoryRepository;
    use crate::proto::k1s0::event::service::order::v1::OrderItem;
    use chrono::Utc;
    use prost::Message;
    use uuid::Uuid;

    // テスト用の在庫エンティティを生成するヘルパー
    fn sample_inventory_item() -> InventoryItem {
        InventoryItem {
            id: Uuid::new_v4(),
            product_id: "PROD-001".to_string(),
            warehouse_id: "WH-DEFAULT".to_string(),
            qty_available: 100,
            qty_reserved: 0,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
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
            user_id: "saga-consumer".to_string(),
            reason: "payment failed".to_string(),
        };
        event.encode_to_vec()
    }

    // order.created を受信して在庫が確保されることを確認する
    #[tokio::test]
    async fn test_dispatch_order_created_reserves_stock() {
        let item = sample_inventory_item();
        let item_id = item.id;
        let item_clone = item.clone();
        let mut reserved = item.clone();
        reserved.qty_available = 95;
        reserved.qty_reserved = 5;
        reserved.version = 2;
        let reserved_clone = reserved.clone();

        let mut mock_repo = MockInventoryRepository::new();
        mock_repo
            .expect_find_by_product_and_warehouse()
            .times(1)
            .returning(move |_, _| Ok(Some(item_clone.clone())));
        mock_repo
            .expect_reserve_stock()
            .withf(move |id, qty, ver, _| *id == item_id && *qty == 5 && *ver == 1)
            .times(1)
            .returning(move |_, _, _, _| Ok(reserved_clone.clone()));

        let uc = HandleOrderEventUseCase::new(Arc::new(mock_repo));
        let payload = encode_order_created("ORD-001");
        let result = dispatch_message(&uc, "order.created", "order.cancelled", "order.created", &payload).await;
        assert!(result.is_ok());
    }

    // order.cancelled を受信して在庫解放処理（現在は警告ログのみ）が Ok を返すことを確認する
    #[tokio::test]
    async fn test_dispatch_order_cancelled_releases_stock() {
        // handle_cancelled は現在 NOP（TODO 実装待ち）のためリポジトリ呼び出しなし
        let mock_repo = MockInventoryRepository::new();
        let uc = HandleOrderEventUseCase::new(Arc::new(mock_repo));
        let payload = encode_order_cancelled("ORD-001");
        let result = dispatch_message(&uc, "order.created", "order.cancelled", "order.cancelled", &payload).await;
        assert!(result.is_ok());
    }

    // 未知のトピックを受信した場合はスキップして Ok を返すことを確認する
    #[tokio::test]
    async fn test_dispatch_unknown_topic_returns_ok() {
        let mock_repo = MockInventoryRepository::new();
        let uc = HandleOrderEventUseCase::new(Arc::new(mock_repo));
        let result = dispatch_message(&uc, "order.created", "order.cancelled", "unknown.topic", b"dummy").await;
        assert!(result.is_ok());
    }

    // order.created トピックに不正なバイト列を受信した場合はデシリアライズエラーを返すことを確認する
    #[tokio::test]
    async fn test_dispatch_order_created_invalid_protobuf() {
        let mock_repo = MockInventoryRepository::new();
        let uc = HandleOrderEventUseCase::new(Arc::new(mock_repo));
        let result = dispatch_message(&uc, "order.created", "order.cancelled", "order.created", b"\xff\xfe").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("デシリアライズに失敗"));
    }

    // order.cancelled トピックに不正なバイト列を受信した場合はデシリアライズエラーを返すことを確認する
    #[tokio::test]
    async fn test_dispatch_order_cancelled_invalid_protobuf() {
        let mock_repo = MockInventoryRepository::new();
        let uc = HandleOrderEventUseCase::new(Arc::new(mock_repo));
        let result = dispatch_message(&uc, "order.created", "order.cancelled", "order.cancelled", b"\xff\xfe").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("デシリアライズに失敗"));
    }

    // UseCase がエラーを返した場合にエラーが伝播することを確認する
    #[tokio::test]
    async fn test_dispatch_order_created_usecase_error_propagates() {
        let mut mock_repo = MockInventoryRepository::new();
        mock_repo
            .expect_find_by_product_and_warehouse()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("DB接続エラー")));

        let uc = HandleOrderEventUseCase::new(Arc::new(mock_repo));
        let payload = encode_order_created("ORD-001");
        let result = dispatch_message(&uc, "order.created", "order.cancelled", "order.created", &payload).await;
        assert!(result.is_err());
    }

    // 空の payload を受信した場合、Protobuf のデフォルト値（空 items）で handle_created が Ok を返すことを確認する
    #[tokio::test]
    async fn test_dispatch_empty_payload_order_created() {
        // 空バイト列は Protobuf デコード成功（全フィールドがデフォルト値、items = []）
        // handle_created はアイテムを反復しないため、リポジトリ呼び出しなしで Ok を返す
        let mock_repo = MockInventoryRepository::new();
        let uc = HandleOrderEventUseCase::new(Arc::new(mock_repo));
        let result = dispatch_message(&uc, "order.created", "order.cancelled", "order.created", &[]).await;
        assert!(result.is_ok());
    }
}
