use serde::{Deserialize, Serialize};

use crate::error::KafkaError;

/// KafkaConfig は Kafka クライアントの基本設定を表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    /// Kafka ブローカーアドレスのリスト
    pub brokers: Vec<String>,
    /// セキュリティプロトコル（PLAINTEXT / SSL / SASL_PLAINTEXT / SASL_SSL）
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    /// コンシューマーグループ ID
    #[serde(default)]
    pub consumer_group: Option<String>,
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
    /// ビルダーを取得する。
    pub fn builder() -> KafkaConfigBuilder {
        KafkaConfigBuilder::default()
    }

    /// ブローカーアドレスをカンマ区切り文字列で返す（rdkafka の bootstrap.servers 用）。
    pub fn bootstrap_servers(&self) -> String {
        self.brokers.join(",")
    }

    /// セキュリティプロトコルが TLS を使用するか判定する。
    pub fn uses_tls(&self) -> bool {
        self.security_protocol.contains("SSL")
    }
}

/// KafkaConfigBuilder は KafkaConfig のビルダー。
#[derive(Default)]
pub struct KafkaConfigBuilder {
    brokers: Vec<String>,
    security_protocol: Option<String>,
    consumer_group: Option<String>,
    connection_timeout_ms: Option<u64>,
    request_timeout_ms: Option<u64>,
    max_message_bytes: Option<usize>,
}

impl KafkaConfigBuilder {
    /// ブローカーアドレスを設定する。
    pub fn brokers(mut self, brokers: Vec<String>) -> Self {
        self.brokers = brokers;
        self
    }

    /// セキュリティプロトコルを設定する。
    pub fn security_protocol(mut self, protocol: &str) -> Self {
        self.security_protocol = Some(protocol.to_string());
        self
    }

    /// コンシューマーグループ ID を設定する。
    pub fn consumer_group(mut self, group: &str) -> Self {
        self.consumer_group = Some(group.to_string());
        self
    }

    /// 接続タイムアウト（ミリ秒）を設定する。
    pub fn connection_timeout_ms(mut self, ms: u64) -> Self {
        self.connection_timeout_ms = Some(ms);
        self
    }

    /// リクエストタイムアウト（ミリ秒）を設定する。
    pub fn request_timeout_ms(mut self, ms: u64) -> Self {
        self.request_timeout_ms = Some(ms);
        self
    }

    /// 最大メッセージサイズ（バイト）を設定する。
    pub fn max_message_bytes(mut self, bytes: usize) -> Self {
        self.max_message_bytes = Some(bytes);
        self
    }

    /// KafkaConfig を構築する。ブローカーが未設定の場合はエラーを返す。
    pub fn build(self) -> Result<KafkaConfig, KafkaError> {
        if self.brokers.is_empty() {
            return Err(KafkaError::ConfigurationError(
                "at least one broker must be specified".to_string(),
            ));
        }
        Ok(KafkaConfig {
            brokers: self.brokers,
            security_protocol: self
                .security_protocol
                .unwrap_or_else(default_security_protocol),
            consumer_group: self.consumer_group,
            connection_timeout_ms: self.connection_timeout_ms.unwrap_or_else(default_timeout_ms),
            request_timeout_ms: self
                .request_timeout_ms
                .unwrap_or_else(default_request_timeout_ms),
            max_message_bytes: self
                .max_message_bytes
                .unwrap_or_else(default_max_message_bytes),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_servers_single() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9092".to_string()])
            .build()
            .unwrap();
        assert_eq!(cfg.bootstrap_servers(), "kafka:9092");
    }

    #[test]
    fn test_bootstrap_servers_multiple() {
        let cfg = KafkaConfig {
            brokers: vec!["kafka-0:9092".to_string(), "kafka-1:9092".to_string()],
            security_protocol: "PLAINTEXT".to_string(),
            consumer_group: None,
            connection_timeout_ms: 5000,
            request_timeout_ms: 30000,
            max_message_bytes: 1_000_000,
        };
        assert_eq!(cfg.bootstrap_servers(), "kafka-0:9092,kafka-1:9092");
    }

    #[test]
    fn test_uses_tls_plaintext() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9092".to_string()])
            .security_protocol("PLAINTEXT")
            .build()
            .unwrap();
        assert!(!cfg.uses_tls());
    }

    #[test]
    fn test_uses_tls_ssl() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9093".to_string()])
            .security_protocol("SSL")
            .build()
            .unwrap();
        assert!(cfg.uses_tls());
    }

    #[test]
    fn test_uses_tls_sasl_ssl() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9094".to_string()])
            .security_protocol("SASL_SSL")
            .build()
            .unwrap();
        assert!(cfg.uses_tls());
    }

    #[test]
    fn test_uses_tls_sasl_plaintext() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9092".to_string()])
            .security_protocol("SASL_PLAINTEXT")
            .build()
            .unwrap();
        assert!(!cfg.uses_tls());
    }

    #[test]
    fn test_deserialize_defaults() {
        let json = r#"{"brokers": ["kafka:9092"]}"#;
        let cfg: KafkaConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.security_protocol, "PLAINTEXT");
        assert_eq!(cfg.connection_timeout_ms, 5000);
    }

    #[test]
    fn test_deserialize_request_timeout_default() {
        let json = r#"{"brokers": ["kafka:9092"]}"#;
        let cfg: KafkaConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.request_timeout_ms, 30000);
        assert_eq!(cfg.max_message_bytes, 1_000_000);
    }

    #[test]
    fn test_bootstrap_servers_empty_via_builder() {
        let err = KafkaConfig::builder().build().unwrap_err();
        assert!(matches!(err, KafkaError::ConfigurationError(_)));
    }

    #[test]
    fn test_builder_with_consumer_group() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9092".to_string()])
            .consumer_group("auth-service-group")
            .security_protocol("SASL_SSL")
            .build()
            .unwrap();
        assert_eq!(cfg.consumer_group.as_deref(), Some("auth-service-group"));
        assert!(cfg.uses_tls());
    }

    #[test]
    fn test_builder_custom_timeouts() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9092".to_string()])
            .connection_timeout_ms(10000)
            .request_timeout_ms(60000)
            .build()
            .unwrap();
        assert_eq!(cfg.connection_timeout_ms, 10000);
        assert_eq!(cfg.request_timeout_ms, 60000);
    }
}
