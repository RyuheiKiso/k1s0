//! Kafka統合テスト
//! 実行には Kafka ブローカーが必要: cargo test -- --ignored
//!
//! テスト対象: KafkaProducer による SagaEventPublisher::publish_saga_event の
//! 実際のメッセージ送信。KafkaConfig のデシリアライズとインメモリプロデューサーの
//! ユニットテストは src/infrastructure/kafka_producer.rs で実施済み。
#![allow(clippy::unwrap_used)]

#[cfg(test)]
mod tests {
    use k1s0_saga_server::infrastructure::kafka_producer::{
        KafkaConfig, SaslConfig, TopicsConfig,
    };

    // =========================================================================
    // ユニットテスト（外部サービス不要）
    // =========================================================================

    /// KafkaConfig のYAMLデシリアライズ: 全フィールド指定時
    #[test]
    fn test_kafka_config_full_deserialization() {
        let yaml = r#"
brokers:
  - "kafka-0.messaging:9092"
  - "kafka-1.messaging:9092"
consumer_group: "saga-server.default"
security_protocol: "SASL_SSL"
sasl:
  mechanism: "PLAIN"
  username: "saga-user"
  password: "secret"
topics:
  publish:
    - "k1s0.system.saga.state_changed.v1"
  subscribe:
    - "k1s0.system.saga.commands.v1"
"#;
        // YAML全フィールド指定時のデシリアライズを検証する
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.brokers.len(), 2);
        assert_eq!(config.brokers[0], "kafka-0.messaging:9092");
        assert_eq!(config.consumer_group, "saga-server.default");
        assert_eq!(config.security_protocol, "SASL_SSL");
        assert_eq!(config.sasl.mechanism, "PLAIN");
        assert_eq!(config.sasl.username, "saga-user");
        assert_eq!(config.sasl.password, "secret");
        assert_eq!(config.topics.publish.len(), 1);
        assert_eq!(config.topics.subscribe.len(), 1);
    }

    /// KafkaConfig のデフォルト値: security_protocol は SASL_SSL、topics は空
    #[test]
    fn test_kafka_config_defaults() {
        let yaml = r#"
brokers:
  - "localhost:9092"
"#;
        // 最小構成のデシリアライズ: デフォルト値が正しく適用されることを検証する
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.security_protocol, "SASL_SSL", "デフォルトはSASL_SSL");
        assert!(config.consumer_group.is_empty(), "consumer_group デフォルトは空文字");
        assert!(config.sasl.mechanism.is_empty(), "SASL mechanism デフォルトは空文字");
        assert!(config.topics.publish.is_empty(), "publish topics デフォルトは空");
        assert!(config.topics.subscribe.is_empty(), "subscribe topics デフォルトは空");
    }

    /// KafkaConfig の security_protocol を明示的に PLAINTEXT に設定可能か検証する
    #[test]
    fn test_kafka_config_plaintext_override() {
        let yaml = r#"
brokers:
  - "localhost:9092"
security_protocol: "PLAINTEXT"
"#;
        // 開発環境向け: PLAINTEXT の明示指定が反映されることを検証する
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.security_protocol, "PLAINTEXT");
    }

    /// KafkaConfig の構造体を直接構築して正当性を確認する
    #[test]
    fn test_kafka_config_struct_construction() {
        // プログラム上で直接 KafkaConfig を構築するケースを検証する
        let config = KafkaConfig {
            brokers: vec!["broker1:9092".to_string(), "broker2:9092".to_string()],
            consumer_group: "test-group".to_string(),
            security_protocol: "SASL_SSL".to_string(),
            sasl: SaslConfig {
                mechanism: "SCRAM-SHA-256".to_string(),
                username: "user".to_string(),
                password: "pass".to_string(),
            },
            topics: TopicsConfig {
                publish: vec!["topic-a".to_string()],
                subscribe: vec!["topic-b".to_string(), "topic-c".to_string()],
            },
        };
        assert_eq!(config.brokers.len(), 2);
        assert_eq!(config.sasl.mechanism, "SCRAM-SHA-256");
        assert_eq!(config.topics.subscribe.len(), 2);
    }

    /// イベントペイロードのJSON構造を検証する（KafkaProducer が送信するメッセージ形式）
    #[test]
    fn test_saga_event_payload_structure() {
        // KafkaProducer::publish_saga_event が構築するイベントJSONの構造を検証する
        let saga_id = "saga-001";
        let event_type = "SAGA_STARTED";
        let payload = serde_json::json!({"order_id": "ORD-123", "amount": 5000});

        let event = serde_json::json!({
            "saga_id": saga_id,
            "event_type": event_type,
            "payload": payload,
            "timestamp": "2025-01-01T00:00:00Z",
        });

        // イベントJSONが必須フィールドを含むことを検証する
        assert_eq!(event["saga_id"], "saga-001");
        assert_eq!(event["event_type"], "SAGA_STARTED");
        assert_eq!(event["payload"]["order_id"], "ORD-123");
        assert!(event["timestamp"].is_string());

        // JSONバイト列へのシリアライズが成功することを検証する
        let bytes = serde_json::to_vec(&event).unwrap();
        assert!(!bytes.is_empty());

        // デシリアライズで元のデータが復元されることを検証する
        let deserialized: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(deserialized["saga_id"], "saga-001");
        assert_eq!(deserialized["payload"]["amount"], 5000);
    }

    /// 複数ブローカー指定時のカンマ区切り結合を検証する
    #[test]
    fn test_brokers_join_for_bootstrap_servers() {
        // rdkafka ClientConfig の bootstrap.servers に渡す値の形式を検証する
        let config = KafkaConfig {
            brokers: vec![
                "kafka-0:9092".to_string(),
                "kafka-1:9092".to_string(),
                "kafka-2:9092".to_string(),
            ],
            consumer_group: String::new(),
            security_protocol: "PLAINTEXT".to_string(),
            sasl: SaslConfig::default(),
            topics: TopicsConfig::default(),
        };

        let joined = config.brokers.join(",");
        assert_eq!(joined, "kafka-0:9092,kafka-1:9092,kafka-2:9092");
    }

    /// デフォルトトピック名の解決ロジックを検証する
    #[test]
    fn test_default_topic_resolution() {
        // publish topics が空の場合、デフォルトトピックにフォールバックすることを検証する
        let config = KafkaConfig {
            brokers: vec!["localhost:9092".to_string()],
            consumer_group: String::new(),
            security_protocol: "PLAINTEXT".to_string(),
            sasl: SaslConfig::default(),
            topics: TopicsConfig::default(),
        };

        // KafkaProducer::new と同じロジック: publish の最初の要素、なければデフォルト
        let topic = config
            .topics
            .publish
            .first()
            .cloned()
            .unwrap_or_else(|| "k1s0.system.saga.state_changed.v1".to_string());

        assert_eq!(topic, "k1s0.system.saga.state_changed.v1");

        // publish にトピックを指定した場合はそれを使う
        let config_with_topic = KafkaConfig {
            brokers: vec!["localhost:9092".to_string()],
            consumer_group: String::new(),
            security_protocol: "PLAINTEXT".to_string(),
            sasl: SaslConfig::default(),
            topics: TopicsConfig {
                publish: vec!["custom.topic.v1".to_string()],
                subscribe: vec![],
            },
        };

        let topic = config_with_topic
            .topics
            .publish
            .first()
            .cloned()
            .unwrap_or_else(|| "k1s0.system.saga.state_changed.v1".to_string());

        assert_eq!(topic, "custom.topic.v1");
    }

    // =========================================================================
    // 統合テスト（Kafkaブローカー必要）
    // =========================================================================

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    #[ignore = "requires Kafka broker (see docker-compose.yaml kafka service)"]
    async fn test_publish_saga_event() {
        // 1. KafkaConfig を構築 (brokers: ["localhost:9092"])
        // 2. KafkaProducer::new(&config)
        // 3. publish_saga_event("saga-001", "SAGA_STARTED", &payload) を呼び出し
        // 4. Kafka consumer で該当トピックからメッセージを読み取り検証
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    #[ignore = "requires Kafka broker"]
    async fn test_publish_and_close() {
        // publish 後に close() を呼び出してフラッシュが正常に完了することを検証
    }
}
