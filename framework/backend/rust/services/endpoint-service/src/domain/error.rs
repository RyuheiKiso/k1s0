//! エンドポイントサービスエラー

use std::fmt;

/// エンドポイントサービスエラー
#[derive(Debug)]
pub enum EndpointError {
    /// エンドポイントが見つからない
    NotFound {
        service_name: String,
    },
    /// サービスを解決できない
    UnresolvableService {
        service_name: String,
        reason: String,
    },
    /// ストレージエラー
    Storage {
        message: String,
    },
    /// 内部エラー
    Internal {
        message: String,
    },
}

impl EndpointError {
    /// NotFound エラーを作成
    pub fn not_found(service_name: impl Into<String>) -> Self {
        Self::NotFound {
            service_name: service_name.into(),
        }
    }

    /// UnresolvableService エラーを作成
    pub fn unresolvable(service_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::UnresolvableService {
            service_name: service_name.into(),
            reason: reason.into(),
        }
    }

    /// Storage エラーを作成
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage {
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
            Self::NotFound { .. } => "ENDPOINT_NOT_FOUND",
            Self::UnresolvableService { .. } => "ENDPOINT_UNRESOLVABLE",
            Self::Storage { .. } => "ENDPOINT_STORAGE_ERROR",
            Self::Internal { .. } => "ENDPOINT_INTERNAL_ERROR",
        }
    }

    /// gRPC ステータスコードに変換
    pub fn to_grpc_code(&self) -> i32 {
        match self {
            Self::NotFound { .. } => 5,  // NOT_FOUND
            Self::UnresolvableService { .. } => 5,  // NOT_FOUND
            Self::Storage { .. } => 14,  // UNAVAILABLE
            Self::Internal { .. } => 13,  // INTERNAL
        }
    }
}

impl fmt::Display for EndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { service_name } => {
                write!(f, "endpoint not found: {}", service_name)
            }
            Self::UnresolvableService { service_name, reason } => {
                write!(f, "cannot resolve {}: {}", service_name, reason)
            }
            Self::Storage { message } => {
                write!(f, "storage error: {}", message)
            }
            Self::Internal { message } => {
                write!(f, "internal error: {}", message)
            }
        }
    }
}

impl std::error::Error for EndpointError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found() {
        let err = EndpointError::not_found("my-service");
        assert_eq!(err.error_code(), "ENDPOINT_NOT_FOUND");
        assert_eq!(err.to_grpc_code(), 5);
    }

    #[test]
    fn test_unresolvable() {
        let err = EndpointError::unresolvable("my-service", "no DNS record");
        assert_eq!(err.error_code(), "ENDPOINT_UNRESOLVABLE");
        assert!(err.to_string().contains("my-service"));
    }
}
