//! Kafka統合テスト
//! 実行には Kafka ブローカーが必要: cargo test -- --ignored
//!
//! テスト対象: KafkaProducer による SagaEventPublisher::publish_saga_event の
//! 実際のメッセージ送信。KafkaConfig のデシリアライズとインメモリプロデューサーの
//! ユニットテストは src/infrastructure/kafka_producer.rs で実施済み。

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "requires Kafka broker (see docker-compose.yaml kafka service)"]
    async fn test_publish_saga_event() {
        // 1. KafkaConfig を構築 (brokers: ["localhost:9092"])
        // 2. KafkaProducer::new(&config)
        // 3. publish_saga_event("saga-001", "SAGA_STARTED", &payload) を呼び出し
        // 4. Kafka consumer で該当トピックからメッセージを読み取り検証
    }

    #[tokio::test]
    #[ignore = "requires Kafka broker"]
    async fn test_publish_and_close() {
        // publish 後に close() を呼び出してフラッシュが正常に完了することを検証
    }
}
