use serde::{Deserialize, Serialize};

/// MessagingConfig は Kafka 接続設定を表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingConfig {
    /// Kafka ブローカーアドレスのリスト（例: ["kafka:9092"]）
    pub brokers: Vec<String>,
    /// セキュリティプロトコル（PLAINTEXT / SSL / SASL_PLAINTEXT / SASL_SSL）
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    /// 接続タイムアウト（ミリ秒）
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    /// プロデューサーのバッチサイズ
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_security_protocol() -> String {
    "PLAINTEXT".to_string()
}

fn default_timeout_ms() -> u64 {
    5000
}

fn default_batch_size() -> usize {
    100
}

impl MessagingConfig {
    /// ブローカーアドレスをカンマ区切り文字列で返す（rdkafka 用）。
    pub fn brokers_string(&self) -> String {
        self.brokers.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brokers_string_single() {
        let cfg = MessagingConfig {
            brokers: vec!["kafka:9092".to_string()],
            security_protocol: "PLAINTEXT".to_string(),
            timeout_ms: 5000,
            batch_size: 100,
        };
        assert_eq!(cfg.brokers_string(), "kafka:9092");
    }

    #[test]
    fn test_brokers_string_multiple() {
        let cfg = MessagingConfig {
            brokers: vec!["kafka-0:9092".to_string(), "kafka-1:9092".to_string()],
            security_protocol: "PLAINTEXT".to_string(),
            timeout_ms: 5000,
            batch_size: 100,
        };
        assert_eq!(cfg.brokers_string(), "kafka-0:9092,kafka-1:9092");
    }

    #[test]
    fn test_deserialize_defaults() {
        let json = r#"{"brokers": ["kafka:9092"]}"#;
        let cfg: MessagingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.security_protocol, "PLAINTEXT");
        assert_eq!(cfg.timeout_ms, 5000);
        assert_eq!(cfg.batch_size, 100);
    }
}
