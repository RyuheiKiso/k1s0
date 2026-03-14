use serde::{Deserialize, Serialize};

/// Schema Registry 接続設定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRegistryConfig {
    /// Schema Registry の URL。
    /// 例: "http://schema-registry:8081"（docker-compose）
    /// または "http://schema-registry.k1s0-system.svc.cluster.local:8081"（Kubernetes）
    pub url: String,

    /// スキーマ互換性モード。
    /// デフォルト: BACKWARD
    #[serde(default = "default_compatibility")]
    pub compatibility: CompatibilityMode,

    /// HTTP タイムアウト（秒）。
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

/// スキーマの互換性モード。
///
/// Confluent Schema Registry がサポートする互換性レベルに対応する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompatibilityMode {
    /// 後方互換性: 新しいスキーマで古いデータを読める。
    Backward,
    /// 推移的後方互換性: すべての過去バージョンと後方互換。
    BackwardTransitive,
    /// 前方互換性: 古いスキーマで新しいデータを読める。
    Forward,
    /// 推移的前方互換性: すべての過去バージョンと前方互換。
    ForwardTransitive,
    /// 完全互換性: 後方互換かつ前方互換。
    Full,
    /// 推移的完全互換性: すべての過去バージョンと完全互換。
    FullTransitive,
    /// 互換性チェックなし。
    None,
}

fn default_compatibility() -> CompatibilityMode {
    CompatibilityMode::Backward
}

fn default_timeout_secs() -> u64 {
    30
}

impl SchemaRegistryConfig {
    /// 指定した URL で設定を作成する。
    ///
    /// 互換性モードは BACKWARD、タイムアウトは 30 秒がデフォルト値として設定される。
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            compatibility: default_compatibility(),
            timeout_secs: default_timeout_secs(),
        }
    }

    /// Kafka トピック名から Schema Registry のサブジェクト名を生成する。
    ///
    /// Confluent の規則に従い `{topic-name}-value` 形式を返す。
    /// トピック名の命名規則: `k1s0.{tier}.{domain}.{event-type}.{version}`
    pub fn subject_name(topic: &str) -> String {
        format!("{}-value", topic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // new で作成した設定のデフォルト値が正しいことを確認する。
    #[test]
    fn test_new_defaults() {
        let cfg = SchemaRegistryConfig::new("http://schema-registry:8081");
        assert_eq!(cfg.url, "http://schema-registry:8081");
        assert_eq!(cfg.compatibility, CompatibilityMode::Backward);
        assert_eq!(cfg.timeout_secs, 30);
    }

    // 標準トピック名からサブジェクト名が正しく生成されることを確認する。
    #[test]
    fn test_subject_name_standard_topic() {
        let subject = SchemaRegistryConfig::subject_name("k1s0.system.auth.user-created.v1");
        assert_eq!(subject, "k1s0.system.auth.user-created.v1-value");
    }

    // シンプルなトピック名からサブジェクト名が正しく生成されることを確認する。
    #[test]
    fn test_subject_name_simple_topic() {
        let subject = SchemaRegistryConfig::subject_name("orders");
        assert_eq!(subject, "orders-value");
    }

    // ビジネス層トピック名からサブジェクト名が正しく生成されることを確認する。
    #[test]
    fn test_subject_name_business_topic() {
        let subject =
            SchemaRegistryConfig::subject_name("k1s0.business.accounting.invoice-issued.v2");
        assert_eq!(subject, "k1s0.business.accounting.invoice-issued.v2-value");
    }

    // URLのみ指定した JSON からデシリアライズしたとき、デフォルト値が適用されることを確認する。
    #[test]
    fn test_deserialize_defaults() {
        let json = r#"{"url": "http://localhost:8081"}"#;
        let cfg: SchemaRegistryConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.compatibility, CompatibilityMode::Backward);
        assert_eq!(cfg.timeout_secs, 30);
    }

    // カスタム互換性モードとタイムアウトを JSON からデシリアライズできることを確認する。
    #[test]
    fn test_deserialize_custom_compatibility() {
        let json =
            r#"{"url": "http://localhost:8081", "compatibility": "FULL", "timeout_secs": 60}"#;
        let cfg: SchemaRegistryConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.compatibility, CompatibilityMode::Full);
        assert_eq!(cfg.timeout_secs, 60);
    }

    // CompatibilityMode が SCREAMING_SNAKE_CASE の JSON 文字列にシリアライズされることを確認する。
    #[test]
    fn test_serialize_compatibility_mode() {
        let mode = CompatibilityMode::BackwardTransitive;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""BACKWARD_TRANSITIVE""#);
    }

    // CompatibilityMode::None がシリアライズ・デシリアライズで正しく往復することを確認する。
    #[test]
    fn test_compatibility_mode_none() {
        let mode = CompatibilityMode::None;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""NONE""#);
        let back: CompatibilityMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, CompatibilityMode::None);
    }

    // SchemaRegistryConfig と CompatibilityMode のクローン・コピーが正しく機能することを確認する。
    #[test]
    fn test_clone_and_copy() {
        let cfg = SchemaRegistryConfig::new("http://schema-registry:8081");
        let cloned = cfg.clone();
        assert_eq!(cloned.url, cfg.url);
        assert_eq!(cloned.compatibility, cfg.compatibility);

        let mode = CompatibilityMode::Forward;
        let copied = mode;
        assert_eq!(mode, copied);
    }

    // Kubernetes 内の完全修飾ドメイン名URLが正しく設定されることを確認する。
    #[test]
    fn test_kubernetes_url() {
        let cfg =
            SchemaRegistryConfig::new("http://schema-registry.k1s0-system.svc.cluster.local:8081");
        assert!(cfg.url.contains("cluster.local"));
    }

    // service 層のトピック名からサブジェクト名が正しく生成されることを確認する。
    #[test]
    fn test_subject_name_service_topic() {
        let subject = SchemaRegistryConfig::subject_name("k1s0.service.notification.email-sent.v1");
        assert_eq!(subject, "k1s0.service.notification.email-sent.v1-value");
    }

    // 空文字列のトピック名からもサブジェクト名が生成されることを確認する。
    #[test]
    fn test_subject_name_empty_topic() {
        let subject = SchemaRegistryConfig::subject_name("");
        assert_eq!(subject, "-value");
    }

    // FORWARD_TRANSITIVE の互換性モードがシリアライズ・デシリアライズで正しく往復することを確認する。
    #[test]
    fn test_compatibility_mode_forward_transitive() {
        let mode = CompatibilityMode::ForwardTransitive;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""FORWARD_TRANSITIVE""#);
        let back: CompatibilityMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, CompatibilityMode::ForwardTransitive);
    }

    // FULL_TRANSITIVE の互換性モードがシリアライズ・デシリアライズで正しく往復することを確認する。
    #[test]
    fn test_compatibility_mode_full_transitive() {
        let mode = CompatibilityMode::FullTransitive;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""FULL_TRANSITIVE""#);
        let back: CompatibilityMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, CompatibilityMode::FullTransitive);
    }

    // FORWARD の互換性モードがシリアライズ・デシリアライズで正しく往復することを確認する。
    #[test]
    fn test_compatibility_mode_forward() {
        let mode = CompatibilityMode::Forward;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""FORWARD""#);
    }

    // BACKWARD の互換性モードがシリアライズで正しい文字列になることを確認する。
    #[test]
    fn test_compatibility_mode_backward_serialize() {
        let mode = CompatibilityMode::Backward;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""BACKWARD""#);
    }

    // 全フィールドを含む JSON からデシリアライズし、カスタムタイムアウトが適用されることを確認する。
    #[test]
    fn test_deserialize_full_config() {
        let json = r#"{
            "url": "http://sr:8081",
            "compatibility": "FULL_TRANSITIVE",
            "timeout_secs": 120
        }"#;
        let cfg: SchemaRegistryConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.url, "http://sr:8081");
        assert_eq!(cfg.compatibility, CompatibilityMode::FullTransitive);
        assert_eq!(cfg.timeout_secs, 120);
    }
}
