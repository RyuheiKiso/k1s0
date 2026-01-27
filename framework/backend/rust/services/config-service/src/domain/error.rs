//! 設定サービスエラー

use std::fmt;

/// 設定サービスエラー
#[derive(Debug)]
pub enum ConfigError {
    /// 設定が見つからない
    NotFound {
        service_name: String,
        key: String,
    },
    /// 無効な値
    InvalidValue {
        key: String,
        reason: String,
    },
    /// ストレージエラー
    Storage {
        message: String,
    },
    /// キャッシュエラー
    Cache {
        message: String,
    },
    /// 内部エラー
    Internal {
        message: String,
    },
}

impl ConfigError {
    /// NotFound エラーを作成
    pub fn not_found(service_name: impl Into<String>, key: impl Into<String>) -> Self {
        Self::NotFound {
            service_name: service_name.into(),
            key: key.into(),
        }
    }

    /// InvalidValue エラーを作成
    pub fn invalid_value(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidValue {
            key: key.into(),
            reason: reason.into(),
        }
    }

    /// Storage エラーを作成
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage {
            message: message.into(),
        }
    }

    /// Cache エラーを作成
    pub fn cache(message: impl Into<String>) -> Self {
        Self::Cache {
            message: message.into(),
        }
    }

    /// Internal エラーを作成
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// エラーコードを取得
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "CONFIG_NOT_FOUND",
            Self::InvalidValue { .. } => "CONFIG_INVALID_VALUE",
            Self::Storage { .. } => "CONFIG_STORAGE_ERROR",
            Self::Cache { .. } => "CONFIG_CACHE_ERROR",
            Self::Internal { .. } => "CONFIG_INTERNAL_ERROR",
        }
    }

    /// gRPC ステータスコードに変換
    pub fn to_grpc_code(&self) -> i32 {
        match self {
            Self::NotFound { .. } => 5,  // NOT_FOUND
            Self::InvalidValue { .. } => 3,  // INVALID_ARGUMENT
            Self::Storage { .. } => 14,  // UNAVAILABLE
            Self::Cache { .. } => 14,  // UNAVAILABLE
            Self::Internal { .. } => 13,  // INTERNAL
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { service_name, key } => {
                write!(f, "setting not found: {}/{}", service_name, key)
            }
            Self::InvalidValue { key, reason } => {
                write!(f, "invalid value for {}: {}", key, reason)
            }
            Self::Storage { message } => {
                write!(f, "storage error: {}", message)
            }
            Self::Cache { message } => {
                write!(f, "cache error: {}", message)
            }
            Self::Internal { message } => {
                write!(f, "internal error: {}", message)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Error Creation Tests
    // ========================================

    #[test]
    fn test_not_found() {
        let err = ConfigError::not_found("my-service", "feature.enabled");
        assert_eq!(err.error_code(), "CONFIG_NOT_FOUND");
        assert_eq!(err.to_grpc_code(), 5);
        assert!(err.to_string().contains("my-service"));
        assert!(err.to_string().contains("feature.enabled"));
    }

    #[test]
    fn test_not_found_with_string() {
        let err = ConfigError::not_found(String::from("service"), String::from("key"));
        assert!(err.to_string().contains("service"));
        assert!(err.to_string().contains("key"));
    }

    #[test]
    fn test_invalid_value() {
        let err = ConfigError::invalid_value("timeout", "must be positive");
        assert_eq!(err.error_code(), "CONFIG_INVALID_VALUE");
        assert_eq!(err.to_grpc_code(), 3);
        assert!(err.to_string().contains("timeout"));
        assert!(err.to_string().contains("must be positive"));
    }

    #[test]
    fn test_storage() {
        let err = ConfigError::storage("connection failed");
        assert_eq!(err.error_code(), "CONFIG_STORAGE_ERROR");
        assert_eq!(err.to_grpc_code(), 14);
        assert!(err.to_string().contains("connection failed"));
    }

    #[test]
    fn test_cache() {
        let err = ConfigError::cache("cache timeout");
        assert_eq!(err.error_code(), "CONFIG_CACHE_ERROR");
        assert_eq!(err.to_grpc_code(), 14);
        assert!(err.to_string().contains("cache timeout"));
    }

    #[test]
    fn test_internal() {
        let err = ConfigError::internal("unexpected error");
        assert_eq!(err.error_code(), "CONFIG_INTERNAL_ERROR");
        assert_eq!(err.to_grpc_code(), 13);
        assert!(err.to_string().contains("unexpected error"));
    }

    // ========================================
    // gRPC Code Mapping Tests
    // ========================================

    #[test]
    fn test_grpc_code_not_found() {
        assert_eq!(ConfigError::not_found("", "").to_grpc_code(), 5);
    }

    #[test]
    fn test_grpc_code_invalid_argument() {
        assert_eq!(ConfigError::invalid_value("", "").to_grpc_code(), 3);
    }

    #[test]
    fn test_grpc_code_unavailable() {
        assert_eq!(ConfigError::storage("").to_grpc_code(), 14);
        assert_eq!(ConfigError::cache("").to_grpc_code(), 14);
    }

    #[test]
    fn test_grpc_code_internal() {
        assert_eq!(ConfigError::internal("").to_grpc_code(), 13);
    }

    // ========================================
    // Error Code Uniqueness Tests
    // ========================================

    #[test]
    fn test_all_error_codes_are_unique() {
        let codes = vec![
            ConfigError::not_found("", "").error_code(),
            ConfigError::invalid_value("", "").error_code(),
            ConfigError::storage("").error_code(),
            ConfigError::cache("").error_code(),
            ConfigError::internal("").error_code(),
        ];

        let mut unique_codes = codes.clone();
        unique_codes.sort();
        unique_codes.dedup();

        assert_eq!(codes.len(), unique_codes.len(), "Error codes should be unique");
    }

    // ========================================
    // Display Trait Tests
    // ========================================

    #[test]
    fn test_display_not_found() {
        let err = ConfigError::NotFound {
            service_name: "test-service".to_string(),
            key: "test-key".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("not found"));
        assert!(display.contains("test-service"));
        assert!(display.contains("test-key"));
    }

    #[test]
    fn test_display_invalid_value() {
        let err = ConfigError::InvalidValue {
            key: "timeout".to_string(),
            reason: "must be > 0".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("invalid value"));
        assert!(display.contains("timeout"));
        assert!(display.contains("must be > 0"));
    }

    #[test]
    fn test_display_storage() {
        let err = ConfigError::Storage {
            message: "db error".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("storage error"));
        assert!(display.contains("db error"));
    }

    #[test]
    fn test_display_cache() {
        let err = ConfigError::Cache {
            message: "cache miss".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("cache error"));
        assert!(display.contains("cache miss"));
    }

    #[test]
    fn test_display_internal() {
        let err = ConfigError::Internal {
            message: "panic".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("internal error"));
        assert!(display.contains("panic"));
    }

    // ========================================
    // Debug Trait Tests
    // ========================================

    #[test]
    fn test_debug_format() {
        let err = ConfigError::not_found("svc", "key");
        let debug = format!("{:?}", err);
        assert!(debug.contains("NotFound"));
    }

    // ========================================
    // std::error::Error Trait Tests
    // ========================================

    #[test]
    fn test_is_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(ConfigError::internal("test"));
        assert!(err.to_string().contains("test"));
    }

    // ========================================
    // Edge Case Tests
    // ========================================

    #[test]
    fn test_empty_strings() {
        let err = ConfigError::not_found("", "");
        assert!(err.to_string().contains("/"));

        let err = ConfigError::invalid_value("", "");
        assert!(err.to_string().contains(":"));

        let err = ConfigError::storage("");
        assert!(err.to_string().contains("storage error:"));
    }

    #[test]
    fn test_long_strings() {
        let long_str = "x".repeat(10000);
        let err = ConfigError::storage(&long_str);
        assert!(err.to_string().contains(&long_str));
    }

    #[test]
    fn test_unicode_strings() {
        let err = ConfigError::not_found("サービス", "設定キー");
        assert!(err.to_string().contains("サービス"));
        assert!(err.to_string().contains("設定キー"));
    }

    #[test]
    fn test_special_characters() {
        let err = ConfigError::storage("error: \"test\" <>&");
        assert!(err.to_string().contains("error: \"test\" <>&"));
    }
}
