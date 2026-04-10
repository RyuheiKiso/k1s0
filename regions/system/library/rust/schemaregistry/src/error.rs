/// `SchemaRegistryError` は Schema Registry 操作に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum SchemaRegistryError {
    /// HTTP リクエストが失敗した。
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// 指定したスキーマが見つからない。
    #[error("Schema not found: subject={subject}, version={version:?}")]
    SchemaNotFound {
        /// スキーマのサブジェクト名。
        subject: String,
        /// スキーマのバージョン（None の場合は latest を指す）。
        version: Option<i32>,
    },

    /// 互換性チェックが失敗した。
    #[error("Compatibility check failed for subject {subject}: {reason}")]
    CompatibilityViolation {
        /// サブジェクト名。
        subject: String,
        /// 失敗理由。
        reason: String,
    },

    /// スキーマの形式が不正。
    #[error("Invalid schema: {0}")]
    InvalidSchema(String),

    /// JSON シリアライズ／デシリアライズに失敗した。
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Schema Registry サービスが利用不可。
    #[error("Schema Registry unavailable: {0}")]
    Unavailable(String),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // バージョン指定の SchemaNotFound エラーメッセージにサブジェクトとバージョンが含まれることを確認する。
    #[test]
    fn test_schema_not_found_with_version() {
        let err = SchemaRegistryError::SchemaNotFound {
            subject: "my-topic-value".to_string(),
            version: Some(3),
        };
        let msg = err.to_string();
        assert!(msg.contains("my-topic-value"));
        assert!(msg.contains("3"));
    }

    // バージョン未指定の SchemaNotFound エラーメッセージにサブジェクト名が含まれることを確認する。
    #[test]
    fn test_schema_not_found_without_version() {
        let err = SchemaRegistryError::SchemaNotFound {
            subject: "my-topic-value".to_string(),
            version: None,
        };
        let msg = err.to_string();
        assert!(msg.contains("my-topic-value"));
        assert!(msg.contains("None"));
    }

    // CompatibilityViolation エラーメッセージにサブジェクトと理由が含まれることを確認する。
    #[test]
    fn test_compatibility_violation() {
        let err = SchemaRegistryError::CompatibilityViolation {
            subject: "orders-value".to_string(),
            reason: "removed required field".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("orders-value"));
        assert!(msg.contains("removed required field"));
    }

    // InvalidSchema エラーメッセージに詳細が含まれることを確認する。
    #[test]
    fn test_invalid_schema() {
        let err = SchemaRegistryError::InvalidSchema("missing syntax declaration".to_string());
        assert!(err.to_string().contains("missing syntax declaration"));
    }

    // serde_json エラーから Serialization エラーへの変換が正しく機能することを確認する。
    #[test]
    fn test_serialization_error_from_serde() {
        let raw = "{invalid json}";
        let serde_err = serde_json::from_str::<serde_json::Value>(raw).unwrap_err();
        let err = SchemaRegistryError::from(serde_err);
        assert!(err.to_string().contains("Serialization error"));
    }

    // Unavailable エラーメッセージに接続エラーの詳細が含まれることを確認する。
    #[test]
    fn test_unavailable() {
        let err = SchemaRegistryError::Unavailable("connection refused".to_string());
        assert!(err.to_string().contains("connection refused"));
    }

    // すべてのエラーバリアントが std::error::Error トレイトを実装していることを確認する。
    #[test]
    fn test_all_variants_implement_error_trait() {
        fn assert_error<E: std::error::Error>(_: &E) {}

        assert_error(&SchemaRegistryError::SchemaNotFound {
            subject: "t".to_string(),
            version: None,
        });
        assert_error(&SchemaRegistryError::CompatibilityViolation {
            subject: "t".to_string(),
            reason: "r".to_string(),
        });
        assert_error(&SchemaRegistryError::InvalidSchema("test".to_string()));
        assert_error(&SchemaRegistryError::Unavailable("test".to_string()));
    }

    // SchemaNotFound エラーの Debug 出力にバリアント名が含まれることを確認する。
    #[test]
    fn test_schema_not_found_debug() {
        let err = SchemaRegistryError::SchemaNotFound {
            subject: "test-topic-value".to_string(),
            version: Some(5),
        };
        let debug = format!("{:?}", err);
        assert!(debug.contains("SchemaNotFound"));
        assert!(debug.contains("test-topic-value"));
    }

    // CompatibilityViolation の表示文字列がサブジェクトと理由の両方を含むことを確認する。
    #[test]
    fn test_compatibility_violation_display_format() {
        let err = SchemaRegistryError::CompatibilityViolation {
            subject: "events-value".to_string(),
            reason: "field type changed from int to string".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("events-value"));
        assert!(display.contains("field type changed from int to string"));
        assert!(display.starts_with("Compatibility check failed"));
    }

    // InvalidSchema の表示文字列が "Invalid schema:" プレフィックスを持つことを確認する。
    #[test]
    fn test_invalid_schema_display_prefix() {
        let err = SchemaRegistryError::InvalidSchema("bad proto syntax".to_string());
        assert!(err.to_string().starts_with("Invalid schema:"));
    }

    // Unavailable の表示文字列が "Schema Registry unavailable:" プレフィックスを持つことを確認する。
    #[test]
    fn test_unavailable_display_prefix() {
        let err = SchemaRegistryError::Unavailable("timeout after 30s".to_string());
        assert!(err.to_string().starts_with("Schema Registry unavailable:"));
    }
}
