//! Integration tests for k1s0-kafka topic module.
//!
//! Validates topic naming conventions and TopicConfig defaults
//! without requiring a running Kafka broker.

use k1s0_kafka::TopicConfig;

// ===========================================================================
// TopicConfig naming validation
// ===========================================================================

#[test]
fn valid_topic_name_full_5_segments() {
    let cfg = TopicConfig {
        name: "k1s0.system.auth.audit.v1".to_string(),
        partitions: 3,
        replication_factor: 3,
        retention_ms: 604_800_000,
    };
    assert!(cfg.validate_name());
}

#[test]
fn valid_topic_name_exactly_4_segments() {
    let cfg = TopicConfig {
        name: "k1s0.system.auth.login".to_string(),
        partitions: 3,
        replication_factor: 3,
        retention_ms: 604_800_000,
    };
    assert!(cfg.validate_name());
}

#[test]
fn valid_topic_name_business_tier() {
    let cfg = TopicConfig {
        name: "k1s0.business.order.created.v1".to_string(),
        partitions: 3,
        replication_factor: 3,
        retention_ms: 604_800_000,
    };
    assert!(cfg.validate_name());
}

#[test]
fn valid_topic_name_service_tier() {
    let cfg = TopicConfig {
        name: "k1s0.service.payment.completed.v2".to_string(),
        partitions: 3,
        replication_factor: 3,
        retention_ms: 604_800_000,
    };
    assert!(cfg.validate_name());
}

#[test]
fn valid_topic_name_many_segments() {
    let cfg = TopicConfig {
        name: "k1s0.system.auth.audit.login.detailed.v3".to_string(),
        partitions: 3,
        replication_factor: 3,
        retention_ms: 604_800_000,
    };
    assert!(cfg.validate_name());
}

#[test]
fn invalid_topic_name_wrong_prefix() {
    let cfg = TopicConfig {
        name: "other.system.auth.login.v1".to_string(),
        partitions: 3,
        replication_factor: 3,
        retention_ms: 604_800_000,
    };
    assert!(!cfg.validate_name());
}

#[test]
fn invalid_topic_name_too_few_segments_3() {
    let cfg = TopicConfig {
        name: "k1s0.system.auth".to_string(),
        partitions: 3,
        replication_factor: 3,
        retention_ms: 604_800_000,
    };
    assert!(!cfg.validate_name());
}

#[test]
fn invalid_topic_name_too_few_segments_2() {
    let cfg = TopicConfig {
        name: "k1s0.system".to_string(),
        partitions: 3,
        replication_factor: 3,
        retention_ms: 604_800_000,
    };
    assert!(!cfg.validate_name());
}

#[test]
fn invalid_topic_name_single_segment() {
    let cfg = TopicConfig {
        name: "k1s0".to_string(),
        partitions: 3,
        replication_factor: 3,
        retention_ms: 604_800_000,
    };
    assert!(!cfg.validate_name());
}

#[test]
fn invalid_topic_name_empty() {
    let cfg = TopicConfig {
        name: String::new(),
        partitions: 3,
        replication_factor: 3,
        retention_ms: 604_800_000,
    };
    assert!(!cfg.validate_name());
}

// ===========================================================================
// TopicConfig serde defaults via JSON deserialization
// ===========================================================================

#[test]
fn deserialize_defaults_partitions() {
    let json = r#"{"name": "k1s0.system.auth.login.v1"}"#;
    let cfg: TopicConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.partitions, 3);
}

#[test]
fn deserialize_defaults_replication_factor() {
    let json = r#"{"name": "k1s0.system.auth.login.v1"}"#;
    let cfg: TopicConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.replication_factor, 3);
}

#[test]
fn deserialize_defaults_retention_ms_is_7_days() {
    let json = r#"{"name": "k1s0.system.auth.login.v1"}"#;
    let cfg: TopicConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.retention_ms, 7 * 24 * 60 * 60 * 1000);
}

#[test]
fn deserialize_explicit_values_override_defaults() {
    let json = r#"{
        "name": "k1s0.system.auth.login.v1",
        "partitions": 12,
        "replication_factor": 2,
        "retention_ms": 86400000
    }"#;
    let cfg: TopicConfig = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.partitions, 12);
    assert_eq!(cfg.replication_factor, 2);
    assert_eq!(cfg.retention_ms, 86_400_000);
}

// ===========================================================================
// TopicConfig serialization roundtrip
// ===========================================================================

#[test]
fn serialization_roundtrip() {
    let original = TopicConfig {
        name: "k1s0.system.auth.audit.v1".to_string(),
        partitions: 6,
        replication_factor: 3,
        retention_ms: 604_800_000,
    };
    let json = serde_json::to_string(&original).unwrap();
    let restored: TopicConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.name, original.name);
    assert_eq!(restored.partitions, original.partitions);
    assert_eq!(restored.replication_factor, original.replication_factor);
    assert_eq!(restored.retention_ms, original.retention_ms);
}

// ===========================================================================
// TopicPartitionInfo
// ===========================================================================

#[test]
fn topic_partition_info_construction() {
    let info = k1s0_kafka::TopicPartitionInfo {
        topic: "k1s0.system.auth.login.v1".to_string(),
        partition: 0,
        leader: 1,
        replicas: vec![1, 2, 3],
        in_sync_replicas: vec![1, 2, 3],
    };
    assert_eq!(info.topic, "k1s0.system.auth.login.v1");
    assert_eq!(info.partition, 0);
    assert_eq!(info.leader, 1);
    assert_eq!(info.replicas.len(), 3);
    assert_eq!(info.in_sync_replicas, info.replicas);
}

#[test]
fn topic_partition_info_partial_isr() {
    let info = k1s0_kafka::TopicPartitionInfo {
        topic: "k1s0.system.auth.login.v1".to_string(),
        partition: 2,
        leader: 1,
        replicas: vec![1, 2, 3],
        in_sync_replicas: vec![1, 3],
    };
    assert_eq!(info.in_sync_replicas.len(), 2);
    assert!(info.in_sync_replicas.len() < info.replicas.len());
}
