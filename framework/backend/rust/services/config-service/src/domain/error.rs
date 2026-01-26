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

    #[test]
    fn test_not_found() {
        let err = ConfigError::not_found("my-service", "feature.enabled");
        assert_eq!(err.error_code(), "CONFIG_NOT_FOUND");
        assert_eq!(err.to_grpc_code(), 5);
        assert!(err.to_string().contains("my-service"));
    }

    #[test]
    fn test_invalid_value() {
        let err = ConfigError::invalid_value("timeout", "must be positive");
        assert_eq!(err.error_code(), "CONFIG_INVALID_VALUE");
        assert_eq!(err.to_grpc_code(), 3);
    }

    #[test]
    fn test_storage() {
        let err = ConfigError::storage("connection failed");
        assert_eq!(err.error_code(), "CONFIG_STORAGE_ERROR");
        assert_eq!(err.to_grpc_code(), 14);
    }
}
