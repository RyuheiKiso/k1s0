// アクティビティイベント Kafka プロデューサー。outbox_poller から呼ばれる。
// イベント種別ごとに専用トピックへルーティングし、topics.yaml との整合性を保つ。
// partition_key により同一アクティビティのイベント順序を保証し、Kafka パーティション間で負荷分散する。
use crate::infrastructure::config::KafkaConfig;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

pub struct ActivityKafkaProducer {
    producer: FutureProducer,
    // アクティビティ作成イベント送信先トピック
    activity_created_topic: String,
    // アクティビティ承認イベント送信先トピック（ActivityApproved 専用）
    activity_approved_topic: String,
}

impl ActivityKafkaProducer {
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("acks", "all");
        // 冪等プロデューサーを有効化し、メッセージの重複送信を防止する
        client_config.set("enable.idempotence", "true");
        let producer = client_config.create()?;
        Ok(Self {
            producer,
            activity_created_topic: config.activity_created_topic.clone(),
            activity_approved_topic: config.activity_approved_topic.clone(),
        })
    }

    /// イベントを Kafka へ発行する。
    /// partition_key をパーティションキーとして使用し、同一アクティビティのイベント順序を保証する。
    /// イベント種別ごとに専用トピックへルーティングし、topics.yaml との整合性を保つ。
    pub async fn publish(&self, event_type: &str, payload: &[u8], partition_key: &str) -> anyhow::Result<()> {
        let topic = match event_type {
            // アクティビティ承認イベントは activity_approved_topic へ送信する
            "ActivityApproved" => &self.activity_approved_topic,
            // その他のイベントはアクティビティ作成トピックへ送信する
            _ => &self.activity_created_topic,
        };
        // partition_key をパーティションキーとして使用し、同一アクティビティのイベントを同一パーティションへ送信する
        let record = FutureRecord::to(topic).payload(payload).key(partition_key);
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| anyhow::anyhow!("kafka send error: {}", e))?;
        Ok(())
    }
}
