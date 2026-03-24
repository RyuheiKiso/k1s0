// ボードイベント Kafka プロデューサー。outbox_poller から呼ばれる。
// board_column_updated_topic へイベントを送信する。
// partition_key により同一ボード/カラムのイベント順序を保証し、Kafka パーティション間で負荷分散する。
use crate::infrastructure::config::KafkaConfig;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

pub struct BoardKafkaProducer {
    producer: FutureProducer,
    // ボードカラム更新イベント送信先トピック
    board_column_updated_topic: String,
}

impl BoardKafkaProducer {
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("acks", "all");
        // 冪等プロデューサーを有効化し、メッセージの重複送信を防止する
        client_config.set("enable.idempotence", "true");
        let producer = client_config.create()?;
        Ok(Self {
            producer,
            board_column_updated_topic: config.board_column_updated_topic.clone(),
        })
    }

    /// イベントを Kafka へ発行する。
    /// partition_key をパーティションキーとして使用し、同一ボード/カラムのイベント順序を保証する。
    /// 固定文字列ではなく動的キーを使用することで、Kafka パーティション間の負荷分散を実現する。
    pub async fn publish(&self, _event_type: &str, payload: &[u8], partition_key: &str) -> anyhow::Result<()> {
        // partition_key をパーティションキーとして使用し、同一エンティティのイベントを同一パーティションへ送信する
        let record = FutureRecord::to(&self.board_column_updated_topic).payload(payload).key(partition_key);
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| anyhow::anyhow!("kafka send error: {}", e))?;
        Ok(())
    }
}
