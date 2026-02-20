/// SchemaRegistryError は Schema Registry 操作に関するエラーを表す。
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
mod tests {
    use super::*;

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

    #[test]
    fn test_invalid_schema() {
        let err = SchemaRegistryError::InvalidSchema("missing syntax declaration".to_string());
        assert!(err.to_string().contains("missing syntax declaration"));
    }

    #[test]
    fn test_serialization_error_from_serde() {
        let raw = "{invalid json}";
        let serde_err = serde_json::from_str::<serde_json::Value>(raw).unwrap_err();
        let err = SchemaRegistryError::from(serde_err);
        assert!(err.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_unavailable() {
        let err = SchemaRegistryError::Unavailable("connection refused".to_string());
        assert!(err.to_string().contains("connection refused"));
    }
}
