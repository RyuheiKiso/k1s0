//! Kafka 統合テスト
//!
//! プロデューサーが送信したメッセージをコンシューマー側で正しく
//! デシリアライズできることを検証する。

use k1s0_config_server::domain::entity::config_change_log::{
    ConfigChangeLog, CreateChangeLogRequest,
};
use k1s0_config_server::infrastructure::kafka_producer::{KafkaConfig, TopicsConfig};
use uuid::Uuid;

/// テスト用のインメモリプロデューサー/コンシューマー。
struct InMemoryBroker {
    messages: std::sync::Mutex<Vec<(String, Vec<u8>)>>,
}

impl InMemoryBroker {
    fn new() -> Self {
        Self {
            messages: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn produce(&self, key: &str, payload: &[u8]) {
        self.messages
            .lock()
            .unwrap()
            .push((key.to_string(), payload.to_vec()));
    }

    fn consume_all(&self) -> Vec<(String, ConfigChangeLog)> {
        self.messages
            .lock()
            .unwrap()
            .iter()
            .map(|(key, payload)| {
                let log: ConfigChangeLog = serde_json::from_slice(payload).unwrap();
                (key.clone(), log)
            })
            .collect()
    }
}

fn make_change_log(
    namespace: &str,
    key: &str,
    change_type: &str,
    old_val: Option<serde_json::Value>,
    new_val: Option<serde_json::Value>,
) -> ConfigChangeLog {
    ConfigChangeLog::new(CreateChangeLogRequest {
        config_entry_id: Uuid::new_v4(),
        namespace: namespace.to_string(),
        key: key.to_string(),
        old_value: old_val,
        new_value: new_val,
        old_version: 1,
        new_version: 2,
        change_type: change_type.to_string(),
        changed_by: "admin@example.com".to_string(),
    })
}

#[test]
fn test_roundtrip_single_message() {
    let broker = InMemoryBroker::new();
    let log = make_change_log(
        "system.auth.database",
        "max_connections",
        "UPDATED",
        Some(serde_json::json!(25)),
        Some(serde_json::json!(50)),
    );

    let payload = serde_json::to_vec(&log).unwrap();
    let key = format!("{}/{}", log.namespace, log.key);
    broker.produce(&key, &payload);

    let consumed = broker.consume_all();
    assert_eq!(consumed.len(), 1);

    let (received_key, received) = &consumed[0];
    assert_eq!(received_key, "system.auth.database/max_connections");
    assert_eq!(received.namespace, "system.auth.database");
    assert_eq!(received.key, "max_connections");
    assert_eq!(received.change_type, "UPDATED");
    assert_eq!(received.old_value, Some(serde_json::json!(25)));
    assert_eq!(received.new_value, Some(serde_json::json!(50)));
}

#[test]
fn test_roundtrip_multiple_messages() {
    let broker = InMemoryBroker::new();

    let events = vec![
        make_change_log(
            "system.auth.database",
            "max_connections",
            "UPDATED",
            Some(serde_json::json!(25)),
            Some(serde_json::json!(50)),
        ),
        make_change_log(
            "system.auth.jwt",
            "issuer",
            "CREATED",
            None,
            Some(serde_json::json!("https://auth.example.com")),
        ),
        make_change_log(
            "system.auth.database",
            "deprecated_flag",
            "DELETED",
            Some(serde_json::json!(true)),
            None,
        ),
    ];

    for log in &events {
        let payload = serde_json::to_vec(log).unwrap();
        let key = format!("{}/{}", log.namespace, log.key);
        broker.produce(&key, &payload);
    }

    let consumed = broker.consume_all();
    assert_eq!(consumed.len(), 3);

    // パーティションキーが namespace/key であることを確認
    assert_eq!(consumed[0].0, "system.auth.database/max_connections");
    assert_eq!(consumed[1].0, "system.auth.jwt/issuer");
    assert_eq!(consumed[2].0, "system.auth.database/deprecated_flag");

    assert_eq!(consumed[0].1.change_type, "UPDATED");
    assert_eq!(consumed[1].1.change_type, "CREATED");
    assert_eq!(consumed[2].1.change_type, "DELETED");
}

#[test]
fn test_kafka_config_topic_names() {
    let config = KafkaConfig {
        brokers: vec!["localhost:9092".to_string()],
        consumer_group: String::new(),
        security_protocol: "PLAINTEXT".to_string(),
        sasl: Default::default(),
        topics: TopicsConfig {
            publish: vec!["k1s0.system.config.changed.v1".to_string()],
            subscribe: vec![],
        },
    };

    let topic = &config.topics.publish[0];
    // k1s0.{tier}.{domain}.{event-type}.{version} 形式であることを検証
    let parts: Vec<&str> = topic.split('.').collect();
    assert_eq!(parts[0], "k1s0");
    assert_eq!(parts[1], "system");
    assert_eq!(parts[2], "config");
    assert_eq!(parts[3], "changed");
    assert_eq!(parts[4], "v1");
}
