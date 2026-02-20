use serde::{Deserialize, Serialize};

/// TopicConfig はトピック作成・管理の設定を表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfig {
    /// トピック名（命名規則: k1s0.{tier}.{domain}.{event-type}.{version}）
    pub name: String,
    /// パーティション数
    #[serde(default = "default_partitions")]
    pub partitions: u32,
    /// レプリケーションファクター
    #[serde(default = "default_replication_factor")]
    pub replication_factor: i16,
    /// メッセージ保持期間（ミリ秒）。デフォルト: 7日
    #[serde(default = "default_retention_ms")]
    pub retention_ms: i64,
}

fn default_partitions() -> u32 {
    3
}

fn default_replication_factor() -> i16 {
    3
}

fn default_retention_ms() -> i64 {
    7 * 24 * 60 * 60 * 1000 // 7日
}

/// TopicPartitionInfo はトピックのパーティション情報を表す。
#[derive(Debug, Clone)]
pub struct TopicPartitionInfo {
    pub topic: String,
    pub partition: i32,
    pub leader: i32,
    pub replicas: Vec<i32>,
    pub in_sync_replicas: Vec<i32>,
}

impl TopicConfig {
    /// トピック名が k1s0 の命名規則に従っているか検証する。
    /// 形式: k1s0.{tier}.{domain}.{event-type}.{version}
    pub fn validate_name(&self) -> bool {
        let parts: Vec<&str> = self.name.split('.').collect();
        parts.len() >= 4 && parts[0] == "k1s0"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_config_defaults() {
        let json = r#"{"name": "k1s0.system.auth.login.v1"}"#;
        let cfg: TopicConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.partitions, 3);
        assert_eq!(cfg.replication_factor, 3);
    }

    #[test]
    fn test_validate_name_valid() {
        let cfg = TopicConfig {
            name: "k1s0.system.auth.login.v1".to_string(),
            partitions: 3,
            replication_factor: 3,
            retention_ms: 604_800_000,
        };
        assert!(cfg.validate_name());
    }

    #[test]
    fn test_validate_name_invalid_prefix() {
        let cfg = TopicConfig {
            name: "other.system.auth.login.v1".to_string(),
            partitions: 3,
            replication_factor: 3,
            retention_ms: 604_800_000,
        };
        assert!(!cfg.validate_name());
    }

    #[test]
    fn test_validate_name_too_short() {
        let cfg = TopicConfig {
            name: "k1s0.system".to_string(),
            partitions: 3,
            replication_factor: 3,
            retention_ms: 604_800_000,
        };
        assert!(!cfg.validate_name());
    }

    #[test]
    fn test_retention_ms_default_is_7_days() {
        let cfg = TopicConfig {
            name: "k1s0.test.topic.v1".to_string(),
            partitions: 3,
            replication_factor: 3,
            retention_ms: default_retention_ms(),
        };
        assert_eq!(cfg.retention_ms, 7 * 24 * 60 * 60 * 1000);
    }
}
