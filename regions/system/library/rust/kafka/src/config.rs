use serde::{Deserialize, Serialize};

/// KafkaConfig は Kafka クライアントの基本設定を表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    /// Kafka ブローカーアドレスのリスト
    pub brokers: Vec<String>,
    /// セキュリティプロトコル（PLAINTEXT / SSL / SASL_PLAINTEXT / SASL_SSL）
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    /// 接続タイムアウト（ミリ秒）
    #[serde(default = "default_timeout_ms")]
    pub connection_timeout_ms: u64,
    /// リクエストタイムアウト（ミリ秒）
    #[serde(default = "default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    /// メッセージの最大サイズ（バイト）
    #[serde(default = "default_max_message_bytes")]
    pub max_message_bytes: usize,
}

fn default_security_protocol() -> String {
    "PLAINTEXT".to_string()
}

fn default_timeout_ms() -> u64 {
    5000
}

fn default_request_timeout_ms() -> u64 {
    30000
}

fn default_max_message_bytes() -> usize {
    1_000_000
}

impl KafkaConfig {
    /// ブローカーアドレスをカンマ区切り文字列で返す（rdkafka の bootstrap.servers 用）。
    pub fn bootstrap_servers(&self) -> String {
        self.brokers.join(",")
    }

    /// セキュリティプロトコルが TLS を使用するか判定する。
    pub fn uses_tls(&self) -> bool {
        self.security_protocol.contains("SSL")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_servers_single() {
        let cfg = KafkaConfig {
            brokers: vec!["kafka:9092".to_string()],
            security_protocol: "PLAINTEXT".to_string(),
            connection_timeout_ms: 5000,
            request_timeout_ms: 30000,
            max_message_bytes: 1_000_000,
        };
        assert_eq!(cfg.bootstrap_servers(), "kafka:9092");
    }

    #[test]
    fn test_bootstrap_servers_multiple() {
        let cfg = KafkaConfig {
            brokers: vec!["kafka-0:9092".to_string(), "kafka-1:9092".to_string()],
            security_protocol: "PLAINTEXT".to_string(),
            connection_timeout_ms: 5000,
            request_timeout_ms: 30000,
            max_message_bytes: 1_000_000,
        };
        assert_eq!(cfg.bootstrap_servers(), "kafka-0:9092,kafka-1:9092");
    }

    #[test]
    fn test_uses_tls_plaintext() {
        let cfg = KafkaConfig {
            brokers: vec!["kafka:9092".to_string()],
            security_protocol: "PLAINTEXT".to_string(),
            connection_timeout_ms: 5000,
            request_timeout_ms: 30000,
            max_message_bytes: 1_000_000,
        };
        assert!(!cfg.uses_tls());
    }

    #[test]
    fn test_uses_tls_ssl() {
        let cfg = KafkaConfig {
            brokers: vec!["kafka:9093".to_string()],
            security_protocol: "SSL".to_string(),
            connection_timeout_ms: 5000,
            request_timeout_ms: 30000,
            max_message_bytes: 1_000_000,
        };
        assert!(cfg.uses_tls());
    }

    #[test]
    fn test_uses_tls_sasl_ssl() {
        let cfg = KafkaConfig {
            brokers: vec!["kafka:9094".to_string()],
            security_protocol: "SASL_SSL".to_string(),
            connection_timeout_ms: 5000,
            request_timeout_ms: 30000,
            max_message_bytes: 1_000_000,
        };
        assert!(cfg.uses_tls());
    }

    #[test]
    fn test_deserialize_defaults() {
        let json = r#"{"brokers": ["kafka:9092"]}"#;
        let cfg: KafkaConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.security_protocol, "PLAINTEXT");
        assert_eq!(cfg.connection_timeout_ms, 5000);
    }
}
