// messaging の外部結合テスト。
// NoOpEventProducer、EventEnvelope 構築、ConsumerConfig、DLQ ヘルパーを検証する。

use std::collections::HashMap;

use k1s0_messaging::{
    ConsumerConfig, EventEnvelope, EventMetadata, EventProducer, MessagingConfig, MessagingError,
    NoOpEventProducer,
};

// --- NoOpEventProducer テスト ---

// NoOpEventProducer の publish が Ok を返すことを確認する。
#[tokio::test]
async fn test_noop_producer_publish() {
    let producer = NoOpEventProducer;
    let envelope = EventEnvelope {
        topic: "test.topic".to_string(),
        key: "key-1".to_string(),
        payload: b"test payload".to_vec(),
        headers: vec![],
        metadata: HashMap::new(),
    };
    let result = producer.publish(envelope).await;
    assert!(result.is_ok());
}

// NoOpEventProducer の publish_batch が Ok を返すことを確認する。
#[tokio::test]
async fn test_noop_producer_publish_batch() {
    let producer = NoOpEventProducer;
    let envelopes = vec![
        EventEnvelope {
            topic: "topic-a".to_string(),
            key: "key-1".to_string(),
            payload: b"payload1".to_vec(),
            headers: vec![],
            metadata: HashMap::new(),
        },
        EventEnvelope {
            topic: "topic-b".to_string(),
            key: "key-2".to_string(),
            payload: b"payload2".to_vec(),
            headers: vec![],
            metadata: HashMap::new(),
        },
    ];
    let result = producer.publish_batch(envelopes).await;
    assert!(result.is_ok());
}

// NoOpEventProducer が空のバッチでも正常に動作することを確認する。
#[tokio::test]
async fn test_noop_producer_empty_batch() {
    let producer = NoOpEventProducer;
    let result = producer.publish_batch(vec![]).await;
    assert!(result.is_ok());
}

// NoOpEventProducer が Send + Sync を満たすことを確認する。
#[test]
fn test_noop_producer_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<NoOpEventProducer>();
}

// --- EventEnvelope 構築テスト ---

// EventEnvelope::json が JSON ペイロードを正しくシリアライズすることを確認する。
#[test]
fn test_event_envelope_json() {
    let payload = serde_json::json!({"user_id": "u-1", "action": "login"});
    let envelope = EventEnvelope::json("k1s0.system.auth.login.v1", "u-1", &payload).unwrap();

    assert_eq!(envelope.topic, "k1s0.system.auth.login.v1");
    assert_eq!(envelope.key, "u-1");
    assert!(!envelope.payload.is_empty());
    assert!(envelope.headers.is_empty());
    assert!(envelope.metadata.is_empty());

    // ペイロードをデシリアライズして元のデータと一致することを確認する
    let deserialized: serde_json::Value = serde_json::from_slice(&envelope.payload).unwrap();
    assert_eq!(deserialized["user_id"], "u-1");
    assert_eq!(deserialized["action"], "login");
}

// EventEnvelope のフルフィールド構築が正しく動作することを確認する。
#[test]
fn test_event_envelope_full_fields() {
    let mut metadata = HashMap::new();
    metadata.insert("trace_id".to_string(), "trace-abc".to_string());

    let headers = vec![("content-type".to_string(), b"application/json".to_vec())];

    let envelope = EventEnvelope {
        topic: "my.topic".to_string(),
        key: "my-key".to_string(),
        payload: b"data".to_vec(),
        headers,
        metadata,
    };

    assert_eq!(envelope.topic, "my.topic");
    assert_eq!(envelope.key, "my-key");
    assert_eq!(envelope.headers.len(), 1);
    assert_eq!(envelope.metadata.get("trace_id").unwrap(), "trace-abc");
}

// --- EventMetadata テスト ---

// EventMetadata::new が正しいデフォルト値で生成されることを確認する。
#[test]
fn test_event_metadata_new() {
    let meta = EventMetadata::new("user.created", "user-service");
    assert_eq!(meta.event_type, "user.created");
    assert_eq!(meta.source, "user-service");
    assert_eq!(meta.schema_version, 1);
    assert!(!meta.event_id.is_empty());
    assert!(meta.trace_id.is_none());
    assert!(meta.correlation_id.is_none());
}

// EventMetadata の event_id が毎回異なることを確認する。
#[test]
fn test_event_metadata_unique_ids() {
    let m1 = EventMetadata::new("test", "svc");
    let m2 = EventMetadata::new("test", "svc");
    assert_ne!(m1.event_id, m2.event_id);
}

// with_trace_id と with_correlation_id が正しく設定されることを確認する。
#[test]
fn test_event_metadata_with_ids() {
    let meta = EventMetadata::new("test", "svc")
        .with_trace_id("trace-123")
        .with_correlation_id("corr-456");
    assert_eq!(meta.trace_id.as_deref(), Some("trace-123"));
    assert_eq!(meta.correlation_id.as_deref(), Some("corr-456"));
}

// to_unix_millis と from_unix_millis の変換が正しく動作することを確認する。
#[test]
fn test_event_metadata_unix_millis() {
    let meta = EventMetadata::new("test", "svc");
    let millis = meta.to_unix_millis();
    let restored = EventMetadata::from_unix_millis(millis).unwrap();
    assert_eq!(restored.timestamp_millis(), millis);
}

// EventMetadata の JSON ラウンドトリップが正しく動作することを確認する。
#[test]
fn test_event_metadata_serde_roundtrip() {
    let meta = EventMetadata::new("test.event", "test-service")
        .with_trace_id("trace-abc")
        .with_correlation_id("corr-xyz");
    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: EventMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(meta, deserialized);
}

// --- ConsumerConfig テスト ---

// ConsumerConfig のデフォルト値が正しいことを確認する。
#[test]
fn test_consumer_config_defaults() {
    let json = r#"{"group_id": "my-group", "topics": ["my-topic"]}"#;
    let cfg: ConsumerConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.group_id, "my-group");
    assert_eq!(cfg.topics, vec!["my-topic"]);
    assert!(!cfg.auto_commit);
    assert_eq!(cfg.session_timeout_ms, 30000);
}

// ConsumerConfig の全フィールド指定が正しく動作することを確認する。
#[test]
fn test_consumer_config_all_fields() {
    let json = r#"{
        "group_id": "consumer-group-1",
        "topics": ["topic-a", "topic-b"],
        "auto_commit": true,
        "session_timeout_ms": 60000
    }"#;
    let cfg: ConsumerConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.group_id, "consumer-group-1");
    assert_eq!(cfg.topics.len(), 2);
    assert!(cfg.auto_commit);
    assert_eq!(cfg.session_timeout_ms, 60000);
}

// ConsumerConfig の JSON ラウンドトリップが正しく動作することを確認する。
#[test]
fn test_consumer_config_roundtrip() {
    let cfg = ConsumerConfig {
        group_id: "test-group".to_string(),
        topics: vec!["t1".to_string(), "t2".to_string()],
        auto_commit: true,
        session_timeout_ms: 45000,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let restored: ConsumerConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.group_id, cfg.group_id);
    assert_eq!(restored.topics, cfg.topics);
    assert_eq!(restored.auto_commit, cfg.auto_commit);
    assert_eq!(restored.session_timeout_ms, cfg.session_timeout_ms);
}

// --- MessagingConfig テスト ---

// MessagingConfig のデフォルト値が正しいことを確認する。
#[test]
fn test_messaging_config_defaults() {
    let json = r#"{"brokers": ["kafka:9092"]}"#;
    let cfg: MessagingConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.security_protocol, "PLAINTEXT");
    assert_eq!(cfg.timeout_ms, 5000);
    assert_eq!(cfg.batch_size, 100);
}

// brokers_string が複数ブローカーをカンマ区切りで連結することを確認する。
#[test]
fn test_messaging_config_brokers_string() {
    let cfg = MessagingConfig {
        brokers: vec!["kafka-0:9092".to_string(), "kafka-1:9092".to_string()],
        security_protocol: "PLAINTEXT".to_string(),
        timeout_ms: 5000,
        batch_size: 100,
    };
    assert_eq!(cfg.brokers_string(), "kafka-0:9092,kafka-1:9092");
}

// --- MessagingError テスト ---

// MessagingError の各バリアントが正しい表示文字列を持つことを確認する。
#[test]
fn test_messaging_error_display() {
    let err = MessagingError::ProducerError("send failed".to_string());
    assert_eq!(err.to_string(), "producer error: send failed");

    let err = MessagingError::ConsumerError("poll failed".to_string());
    assert_eq!(err.to_string(), "consumer error: poll failed");

    let err = MessagingError::TimeoutError("30s".to_string());
    assert_eq!(err.to_string(), "timeout error: 30s");
}
