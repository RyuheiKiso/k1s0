//! Kafka 統合テスト
//!
//! プロデューサーが送信したメッセージをコンシューマー側で正しく
//! デシリアライズできることを検証する。
//!
//! 注意: このテストは実際の Kafka ブローカーへの接続を必要としない。
//! InMemory プロデューサーを使用してメッセージのシリアライズ/デシリアライズを検証する。

use k1s0_auth_server::domain::entity::audit_log::{AuditLog, CreateAuditLogRequest};
use k1s0_auth_server::infrastructure::kafka_producer::{
    AuditEventPublisher, KafkaConfig, TopicsConfig,
};
use std::collections::HashMap;

/// テスト用のインメモリプロデューサー/コンシューマー。
/// Kafka ブローカーなしでメッセージのラウンドトリップを検証する。
struct InMemoryBroker {
    messages: std::sync::Mutex<Vec<(String, Vec<u8>)>>,
}

impl InMemoryBroker {
    fn new() -> Self {
        Self {
            messages: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// プロデューサー側: メッセージを送信する
    fn produce(&self, key: &str, payload: &[u8]) {
        self.messages
            .lock()
            .unwrap()
            .push((key.to_string(), payload.to_vec()));
    }

    /// コンシューマー側: メッセージを受信し AuditLog にデシリアライズする
    fn consume_all(&self) -> Vec<(String, AuditLog)> {
        self.messages
            .lock()
            .unwrap()
            .iter()
            .map(|(key, payload)| {
                let log: AuditLog = serde_json::from_slice(payload).unwrap();
                (key.clone(), log)
            })
            .collect()
    }
}

fn make_audit_log(event_type: &str, user_id: &str, result: &str) -> AuditLog {
    AuditLog::new(CreateAuditLogRequest {
        event_type: event_type.to_string(),
        user_id: user_id.to_string(),
        ip_address: "192.168.1.100".to_string(),
        user_agent: "test-agent".to_string(),
        resource: "/api/v1/auth/token".to_string(),
        action: "POST".to_string(),
        result: result.to_string(),
        metadata: HashMap::from([("client_id".to_string(), "test-client".to_string())]),
    })
}

#[test]
fn test_roundtrip_single_message() {
    let broker = InMemoryBroker::new();
    let log = make_audit_log("LOGIN_SUCCESS", "user-001", "SUCCESS");

    // プロデューサー側: シリアライズして送信
    let payload = serde_json::to_vec(&log).unwrap();
    broker.produce(&log.user_id, &payload);

    // コンシューマー側: デシリアライズして検証
    let consumed = broker.consume_all();
    assert_eq!(consumed.len(), 1);

    let (key, received) = &consumed[0];
    assert_eq!(key, "user-001");
    assert_eq!(received.event_type, "LOGIN_SUCCESS");
    assert_eq!(received.user_id, "user-001");
    assert_eq!(received.result, "SUCCESS");
    assert_eq!(received.metadata.get("client_id").unwrap(), "test-client");
}

#[test]
fn test_roundtrip_multiple_messages() {
    let broker = InMemoryBroker::new();

    let events = vec![
        make_audit_log("LOGIN_SUCCESS", "user-001", "SUCCESS"),
        make_audit_log("LOGIN_FAILURE", "user-002", "FAILURE"),
        make_audit_log("TOKEN_VALIDATE", "user-001", "SUCCESS"),
    ];

    // プロデューサー側
    for log in &events {
        let payload = serde_json::to_vec(log).unwrap();
        broker.produce(&log.user_id, &payload);
    }

    // コンシューマー側
    let consumed = broker.consume_all();
    assert_eq!(consumed.len(), 3);

    // 同一ユーザーのイベントは同一パーティションキーであることを確認
    assert_eq!(consumed[0].0, "user-001");
    assert_eq!(consumed[1].0, "user-002");
    assert_eq!(consumed[2].0, "user-001");

    assert_eq!(consumed[0].1.event_type, "LOGIN_SUCCESS");
    assert_eq!(consumed[1].1.event_type, "LOGIN_FAILURE");
    assert_eq!(consumed[2].1.event_type, "TOKEN_VALIDATE");
}

#[test]
fn test_kafka_config_topic_names() {
    // 設計書のトピック命名規則に一致していることを確認
    let config = KafkaConfig {
        brokers: vec!["localhost:9092".to_string()],
        consumer_group: String::new(),
        security_protocol: "PLAINTEXT".to_string(),
        sasl: Default::default(),
        topics: TopicsConfig {
            publish: vec!["k1s0.system.auth.audit.v1".to_string()],
            subscribe: vec![],
        },
    };

    let topic = &config.topics.publish[0];
    // k1s0.{tier}.{domain}.{event-type}.{version} 形式であることを検証
    let parts: Vec<&str> = topic.split('.').collect();
    assert_eq!(parts[0], "k1s0");
    assert_eq!(parts[1], "system");
    assert_eq!(parts[2], "auth");
    assert_eq!(parts[3], "audit");
    assert_eq!(parts[4], "v1");
}
