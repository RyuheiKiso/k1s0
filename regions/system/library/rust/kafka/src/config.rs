use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub consumer_group: String,
    /// SASL メカニズム（例: PLAIN / SCRAM-SHA-256 / SCRAM-SHA-512）
    #[serde(default)]
    pub sasl_mechanism: Option<String>,
    /// SASL ユーザー名
    #[serde(default)]
    pub sasl_username: Option<String>,
    /// SASL パスワード
    #[serde(default)]
    pub sasl_password: Option<String>,
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

/// セキュリティデフォルト: 本番環境では SASL_SSL を強制する。
/// 開発環境では config.dev.yaml / config.docker.yaml で明示的に PLAINTEXT を指定すること。
fn default_security_protocol() -> String {
    "SASL_SSL".to_string()
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

    /// Kafka クライアント初期化に使う設定マップを返す。
    /// producer/consumer の双方で同じ値を利用できる。
    pub fn client_properties(&self) -> HashMap<String, String> {
        let mut props = HashMap::new();
        props.insert("bootstrap.servers".to_string(), self.bootstrap_servers());
        props.insert(
            "security.protocol".to_string(),
            self.security_protocol.clone(),
        );

        if let Some(mech) = &self.sasl_mechanism {
            props.insert("sasl.mechanism".to_string(), mech.clone());
        }
        if let Some(username) = &self.sasl_username {
            props.insert("sasl.username".to_string(), username.clone());
        }
        if let Some(password) = &self.sasl_password {
            props.insert("sasl.password".to_string(), password.clone());
        }

        props
    }
}

/// KafkaConfigBuilder は KafkaConfig のビルダー。
#[derive(Default)]
pub struct KafkaConfigBuilder {
    brokers: Vec<String>,
    security_protocol: Option<String>,
    consumer_group: Option<String>,
    sasl_mechanism: Option<String>,
    sasl_username: Option<String>,
    sasl_password: Option<String>,
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

    /// SASL メカニズムを設定する。
    pub fn sasl_mechanism(mut self, mechanism: &str) -> Self {
        self.sasl_mechanism = Some(mechanism.to_string());
        self
    }

    /// SASL ユーザー名を設定する。
    pub fn sasl_username(mut self, username: &str) -> Self {
        self.sasl_username = Some(username.to_string());
        self
    }

    /// SASL パスワードを設定する。
    pub fn sasl_password(mut self, password: &str) -> Self {
        self.sasl_password = Some(password.to_string());
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
            consumer_group: self.consumer_group.ok_or_else(|| {
                KafkaError::ConfigurationError("consumer_group is required".to_string())
            })?,
            sasl_mechanism: self.sasl_mechanism,
            sasl_username: self.sasl_username,
            sasl_password: self.sasl_password,
            connection_timeout_ms: self
                .connection_timeout_ms
                .unwrap_or_else(default_timeout_ms),
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // 単一ブローカーの場合に bootstrap_servers がそのアドレスをそのまま返すことを確認する。
    #[test]
    fn test_bootstrap_servers_single() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9092".to_string()])
            .consumer_group("test-group")
            .build()
            .unwrap();
        assert_eq!(cfg.bootstrap_servers(), "kafka:9092");
    }

    // 複数ブローカーの場合に bootstrap_servers がカンマ区切りのアドレス文字列を返すことを確認する。
    #[test]
    fn test_bootstrap_servers_multiple() {
        let cfg = KafkaConfig {
            brokers: vec!["kafka-0:9092".to_string(), "kafka-1:9092".to_string()],
            security_protocol: "PLAINTEXT".to_string(),
            consumer_group: "test-group".to_string(),
            sasl_mechanism: None,
            sasl_username: None,
            sasl_password: None,
            connection_timeout_ms: 5000,
            request_timeout_ms: 30000,
            max_message_bytes: 1_000_000,
        };
        assert_eq!(cfg.bootstrap_servers(), "kafka-0:9092,kafka-1:9092");
    }

    // PLAINTEXT プロトコルの場合に uses_tls が false を返すことを確認する。
    #[test]
    fn test_uses_tls_plaintext() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9092".to_string()])
            .consumer_group("test-group")
            .security_protocol("PLAINTEXT")
            .build()
            .unwrap();
        assert!(!cfg.uses_tls());
    }

    // SSL プロトコルの場合に uses_tls が true を返すことを確認する。
    #[test]
    fn test_uses_tls_ssl() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9093".to_string()])
            .consumer_group("test-group")
            .security_protocol("SSL")
            .build()
            .unwrap();
        assert!(cfg.uses_tls());
    }

    // SASL_SSL プロトコルの場合に uses_tls が true を返すことを確認する。
    #[test]
    fn test_uses_tls_sasl_ssl() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9094".to_string()])
            .consumer_group("test-group")
            .security_protocol("SASL_SSL")
            .build()
            .unwrap();
        assert!(cfg.uses_tls());
    }

    // SASL_PLAINTEXT プロトコルの場合に uses_tls が false を返すことを確認する。
    #[test]
    fn test_uses_tls_sasl_plaintext() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9092".to_string()])
            .consumer_group("test-group")
            .security_protocol("SASL_PLAINTEXT")
            .build()
            .unwrap();
        assert!(!cfg.uses_tls());
    }

    // JSON デシリアライズ時にセキュリティプロトコルと接続タイムアウトのデフォルト値が設定されることを確認する。
    #[test]
    fn test_deserialize_defaults() {
        let json = r#"{"brokers": ["kafka:9092"], "consumer_group": "test-group"}"#;
        let cfg: KafkaConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.security_protocol, "SASL_SSL");
        assert_eq!(cfg.connection_timeout_ms, 5000);
    }

    // JSON デシリアライズ時にリクエストタイムアウトと最大メッセージサイズのデフォルト値が設定されることを確認する。
    #[test]
    fn test_deserialize_request_timeout_default() {
        let json = r#"{"brokers": ["kafka:9092"], "consumer_group": "test-group"}"#;
        let cfg: KafkaConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.request_timeout_ms, 30000);
        assert_eq!(cfg.max_message_bytes, 1_000_000);
    }

    // SASL 認証情報が client_properties に正しく含まれることを確認する。
    #[test]
    fn test_client_properties_with_sasl() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9092".to_string()])
            .consumer_group("test-group")
            .security_protocol("SASL_SSL")
            .sasl_mechanism("SCRAM-SHA-512")
            .sasl_username("user1")
            .sasl_password("secret1")
            .build()
            .unwrap();

        let props = cfg.client_properties();
        assert_eq!(
            props.get("security.protocol"),
            Some(&"SASL_SSL".to_string())
        );
        assert_eq!(
            props.get("sasl.mechanism"),
            Some(&"SCRAM-SHA-512".to_string())
        );
        assert_eq!(props.get("sasl.username"), Some(&"user1".to_string()));
        assert_eq!(props.get("sasl.password"), Some(&"secret1".to_string()));
    }

    // ブローカーが空のままビルドするとエラーになることを確認する。
    #[test]
    fn test_bootstrap_servers_empty_via_builder() {
        let err = KafkaConfig::builder().build().unwrap_err();
        assert!(matches!(err, KafkaError::ConfigurationError(_)));
    }

    // ビルダーでコンシューマーグループと SASL_SSL プロトコルを設定した場合に正しく構築されることを確認する。
    #[test]
    fn test_builder_with_consumer_group() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9092".to_string()])
            .consumer_group("auth-service-group")
            .security_protocol("SASL_SSL")
            .build()
            .unwrap();
        assert_eq!(cfg.consumer_group, "auth-service-group");
        assert!(cfg.uses_tls());
    }

    // ビルダーでカスタムタイムアウトを指定した場合に正しく設定されることを確認する。
    #[test]
    fn test_builder_custom_timeouts() {
        let cfg = KafkaConfig::builder()
            .brokers(vec!["kafka:9092".to_string()])
            .consumer_group("test-group")
            .connection_timeout_ms(10000)
            .request_timeout_ms(60000)
            .build()
            .unwrap();
        assert_eq!(cfg.connection_timeout_ms, 10000);
        assert_eq!(cfg.request_timeout_ms, 60000);
    }

    // コンシューマーグループなしでビルドするとエラーになることを確認する。
    #[test]
    fn test_builder_without_consumer_group() {
        let err = KafkaConfig::builder()
            .brokers(vec!["kafka:9092".to_string()])
            .build()
            .unwrap_err();
        assert!(matches!(err, KafkaError::ConfigurationError(_)));
    }
}
