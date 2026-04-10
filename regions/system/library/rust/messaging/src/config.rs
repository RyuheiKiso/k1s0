use serde::{Deserialize, Serialize};

/// `MessagingConfig` は Kafka 接続設定を表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingConfig {
    /// Kafka ブローカーアドレスのリスト（例: `["kafka:9092"]`）
    pub brokers: Vec<String>,
    /// セキュリティプロトコル（PLAINTEXT / SSL / `SASL_PLAINTEXT` / `SASL_SSL`）
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    /// 接続タイムアウト（ミリ秒）
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    /// プロデューサーのバッチサイズ
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// スキーマレジストリの URL（例: "<http://schema-registry:8081>"）。
    /// 設定されている場合、プロデューサー/コンシューマーはスキーマ検証を有効化する。
    #[serde(default)]
    pub schema_registry_url: Option<String>,
}

/// セキュリティデフォルト: 本番環境では `SASL_SSL` を強制する。
/// 開発環境では config.dev.yaml / config.docker.yaml で明示的に PLAINTEXT を指定すること。
fn default_security_protocol() -> String {
    "SASL_SSL".to_string()
}

fn default_timeout_ms() -> u64 {
    5000
}

fn default_batch_size() -> usize {
    100
}

impl MessagingConfig {
    /// ブローカーアドレスをカンマ区切り文字列で返す（rdkafka 用）。
    #[must_use]
    pub fn brokers_string(&self) -> String {
        self.brokers.join(",")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // 単一ブローカーの場合に brokers_string がそのアドレスをそのまま返すことを確認する。
    #[test]
    fn test_brokers_string_single() {
        let cfg = MessagingConfig {
            brokers: vec!["kafka:9092".to_string()],
            security_protocol: "PLAINTEXT".to_string(),
            timeout_ms: 5000,
            batch_size: 100,
            schema_registry_url: None,
        };
        assert_eq!(cfg.brokers_string(), "kafka:9092");
    }

    // 複数ブローカーの場合に brokers_string がカンマ区切りのアドレス文字列を返すことを確認する。
    #[test]
    fn test_brokers_string_multiple() {
        let cfg = MessagingConfig {
            brokers: vec!["kafka-0:9092".to_string(), "kafka-1:9092".to_string()],
            security_protocol: "PLAINTEXT".to_string(),
            timeout_ms: 5000,
            batch_size: 100,
            schema_registry_url: None,
        };
        assert_eq!(cfg.brokers_string(), "kafka-0:9092,kafka-1:9092");
    }

    // JSON デシリアライズ時に MessagingConfig のデフォルト値が正しく設定されることを確認する。
    #[test]
    fn test_deserialize_defaults() {
        let json = r#"{"brokers": ["kafka:9092"]}"#;
        let cfg: MessagingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.security_protocol, "SASL_SSL");
        assert_eq!(cfg.timeout_ms, 5000);
        assert_eq!(cfg.batch_size, 100);
        assert!(cfg.schema_registry_url.is_none());
    }

    // 全フィールドを明示的に指定した JSON から正しくデシリアライズされることを確認する。
    #[test]
    fn test_deserialize_all_fields_explicit() {
        let json = r#"{
            "brokers": ["kafka-0:9092", "kafka-1:9092", "kafka-2:9092"],
            "security_protocol": "SASL_SSL",
            "timeout_ms": 10000,
            "batch_size": 500
        }"#;
        let cfg: MessagingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.brokers.len(), 3);
        assert_eq!(cfg.security_protocol, "SASL_SSL");
        assert_eq!(cfg.timeout_ms, 10000);
        assert_eq!(cfg.batch_size, 500);
    }

    // MessagingConfig をシリアライズ・デシリアライズしても全フィールドが保持されることを確認する。
    #[test]
    fn test_config_roundtrip() {
        let cfg = MessagingConfig {
            brokers: vec!["kafka-0:9092".to_string(), "kafka-1:9092".to_string()],
            security_protocol: "SSL".to_string(),
            timeout_ms: 3000,
            batch_size: 200,
            schema_registry_url: Some("http://schema-registry:8081".to_string()),
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let restored: MessagingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.brokers, cfg.brokers);
        assert_eq!(restored.security_protocol, cfg.security_protocol);
        assert_eq!(restored.timeout_ms, cfg.timeout_ms);
        assert_eq!(restored.batch_size, cfg.batch_size);
        assert_eq!(restored.schema_registry_url, cfg.schema_registry_url);
    }

    // 空のブローカーリストで brokers_string が空文字列を返すことを確認する。
    #[test]
    fn test_brokers_string_empty() {
        let cfg = MessagingConfig {
            brokers: vec![],
            security_protocol: "PLAINTEXT".to_string(),
            timeout_ms: 5000,
            batch_size: 100,
            schema_registry_url: None,
        };
        assert_eq!(cfg.brokers_string(), "");
    }

    // MessagingConfig の Clone が全フィールドを正しくコピーすることを確認する。
    #[test]
    fn test_config_clone() {
        let cfg = MessagingConfig {
            brokers: vec!["kafka:9092".to_string()],
            security_protocol: "PLAINTEXT".to_string(),
            timeout_ms: 5000,
            batch_size: 100,
            schema_registry_url: None,
        };
        let cloned = cfg.clone();
        assert_eq!(cloned.brokers, cfg.brokers);
        assert_eq!(cloned.security_protocol, cfg.security_protocol);
        assert_eq!(cloned.timeout_ms, cfg.timeout_ms);
        assert_eq!(cloned.batch_size, cfg.batch_size);
    }

    // 3つのブローカーの場合に brokers_string がカンマ区切りで連結されることを確認する。
    #[test]
    fn test_brokers_string_three() {
        let cfg = MessagingConfig {
            brokers: vec![
                "kafka-0:9092".to_string(),
                "kafka-1:9092".to_string(),
                "kafka-2:9092".to_string(),
            ],
            security_protocol: "PLAINTEXT".to_string(),
            timeout_ms: 5000,
            batch_size: 100,
            schema_registry_url: None,
        };
        assert_eq!(
            cfg.brokers_string(),
            "kafka-0:9092,kafka-1:9092,kafka-2:9092"
        );
    }

    // セキュリティプロトコルに SASL_PLAINTEXT を指定できることを確認する。
    #[test]
    fn test_security_protocol_sasl_plaintext() {
        let json = r#"{"brokers": ["kafka:9092"], "security_protocol": "SASL_PLAINTEXT"}"#;
        let cfg: MessagingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.security_protocol, "SASL_PLAINTEXT");
    }

    // schema_registry_url を指定した場合に正しくデシリアライズされることを確認する。
    #[test]
    fn test_schema_registry_url() {
        let json =
            r#"{"brokers": ["kafka:9092"], "schema_registry_url": "http://schema-registry:8081"}"#;
        let cfg: MessagingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            cfg.schema_registry_url,
            Some("http://schema-registry:8081".to_string())
        );
    }

    // schema_registry_url が省略された場合に None になることを確認する。
    #[test]
    fn test_schema_registry_url_none_by_default() {
        let json = r#"{"brokers": ["kafka:9092"]}"#;
        let cfg: MessagingConfig = serde_json::from_str(json).unwrap();
        assert!(cfg.schema_registry_url.is_none());
    }
}
