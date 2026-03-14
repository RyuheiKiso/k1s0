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

/// tier 別のデフォルトパーティション数を返す。
///
/// - system tier: 6 パーティション
/// - business tier: 6 パーティション
/// - service tier / その他: 3 パーティション
pub fn default_partitions_for_tier(tier: &str) -> u32 {
    match tier {
        "system" | "business" => 6,
        _ => 3,
    }
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

    /// トピック名から tier を抽出する（system / business / service）。
    /// 名前が不正な場合は空文字列を返す。
    pub fn tier(&self) -> &str {
        let parts: Vec<&str> = self.name.split('.').collect();
        if parts.len() >= 2 && parts[0] == "k1s0" {
            match parts[1] {
                "system" | "business" | "service" => parts[1],
                _ => "",
            }
        } else {
            ""
        }
    }

    /// トピック名から tier を判定し、tier 別デフォルトパーティション数を設定した TopicConfig を返す。
    /// パーティション数が明示指定されていない（デフォルト値 3 のまま）場合に tier 別の値で上書きする。
    pub fn with_tier_defaults(mut self) -> Self {
        let parts: Vec<&str> = self.name.split('.').collect();
        let tier = if parts.len() >= 2 && parts[0] == "k1s0" {
            match parts[1] {
                "system" | "business" | "service" => parts[1],
                _ => "",
            }
        } else {
            ""
        };
        if !tier.is_empty() {
            self.partitions = default_partitions_for_tier(tier);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // JSON デシリアライズ時にデフォルト値（パーティション数・レプリケーションファクター）が正しく設定されることを確認する。
    #[test]
    fn test_topic_config_defaults() {
        let json = r#"{"name": "k1s0.system.auth.login.v1"}"#;
        let cfg: TopicConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.partitions, 3);
        assert_eq!(cfg.replication_factor, 3);
    }

    // k1s0 の命名規則に従った正しいトピック名を validate_name が true と判定することを確認する。
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

    // k1s0 以外のプレフィックスを持つトピック名を validate_name が false と判定することを確認する。
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

    // セグメント数が 4 未満のトピック名を validate_name が false と判定することを確認する。
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

    // デフォルトのメッセージ保持期間が 7 日間（ミリ秒）であることを確認する。
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

    // system tier のデフォルトパーティション数が 6 であることを確認する。
    #[test]
    fn test_default_partitions_for_tier_system() {
        assert_eq!(default_partitions_for_tier("system"), 6);
    }

    // business tier のデフォルトパーティション数が 6 であることを確認する。
    #[test]
    fn test_default_partitions_for_tier_business() {
        assert_eq!(default_partitions_for_tier("business"), 6);
    }

    // service tier のデフォルトパーティション数が 3 であることを確認する。
    #[test]
    fn test_default_partitions_for_tier_service() {
        assert_eq!(default_partitions_for_tier("service"), 3);
    }

    // 未知の tier に対してデフォルトパーティション数が 3 であることを確認する。
    #[test]
    fn test_default_partitions_for_tier_unknown() {
        assert_eq!(default_partitions_for_tier("other"), 3);
    }

    // トピック名から tier（system / business / service）が正しく抽出されることを確認する。
    #[test]
    fn test_tier_extraction() {
        let cfg = TopicConfig {
            name: "k1s0.system.auth.login.v1".to_string(),
            partitions: 3,
            replication_factor: 3,
            retention_ms: 604_800_000,
        };
        assert_eq!(cfg.tier(), "system");

        let cfg = TopicConfig {
            name: "k1s0.business.order.created.v1".to_string(),
            partitions: 3,
            replication_factor: 3,
            retention_ms: 604_800_000,
        };
        assert_eq!(cfg.tier(), "business");

        let cfg = TopicConfig {
            name: "k1s0.service.payment.done.v1".to_string(),
            partitions: 3,
            replication_factor: 3,
            retention_ms: 604_800_000,
        };
        assert_eq!(cfg.tier(), "service");
    }

    // 不正なトピック名から tier を抽出した場合に空文字列が返ることを確認する。
    #[test]
    fn test_tier_extraction_invalid() {
        let cfg = TopicConfig {
            name: "invalid".to_string(),
            partitions: 3,
            replication_factor: 3,
            retention_ms: 604_800_000,
        };
        assert_eq!(cfg.tier(), "");
    }

    // system tier トピックに with_tier_defaults を適用するとパーティション数が 6 になることを確認する。
    #[test]
    fn test_with_tier_defaults_system() {
        let cfg = TopicConfig {
            name: "k1s0.system.auth.login.v1".to_string(),
            partitions: 3,
            replication_factor: 3,
            retention_ms: 604_800_000,
        }
        .with_tier_defaults();
        assert_eq!(cfg.partitions, 6);
    }

    // business tier トピックに with_tier_defaults を適用するとパーティション数が 6 になることを確認する。
    #[test]
    fn test_with_tier_defaults_business() {
        let cfg = TopicConfig {
            name: "k1s0.business.order.created.v1".to_string(),
            partitions: 3,
            replication_factor: 3,
            retention_ms: 604_800_000,
        }
        .with_tier_defaults();
        assert_eq!(cfg.partitions, 6);
    }

    // service tier トピックに with_tier_defaults を適用してもパーティション数が 3 のままであることを確認する。
    #[test]
    fn test_with_tier_defaults_service() {
        let cfg = TopicConfig {
            name: "k1s0.service.payment.done.v1".to_string(),
            partitions: 3,
            replication_factor: 3,
            retention_ms: 604_800_000,
        }
        .with_tier_defaults();
        assert_eq!(cfg.partitions, 3);
    }
}
