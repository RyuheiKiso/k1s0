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

    // ========================================
    // Error Creation Tests
    // ========================================

    #[test]
    fn test_not_found() {
        let err = EndpointError::not_found("my-service");
        assert_eq!(err.error_code(), "ENDPOINT_NOT_FOUND");
        assert_eq!(err.to_grpc_code(), 5);
        assert!(err.to_string().contains("my-service"));
    }

    #[test]
    fn test_not_found_with_string() {
        let err = EndpointError::not_found(String::from("service"));
        assert!(err.to_string().contains("service"));
    }

    #[test]
    fn test_unresolvable() {
        let err = EndpointError::unresolvable("my-service", "no DNS record");
        assert_eq!(err.error_code(), "ENDPOINT_UNRESOLVABLE");
        assert_eq!(err.to_grpc_code(), 5);
        assert!(err.to_string().contains("my-service"));
        assert!(err.to_string().contains("no DNS record"));
    }

    #[test]
    fn test_unresolvable_with_strings() {
        let err = EndpointError::unresolvable(
            String::from("service"),
            String::from("reason"),
        );
        assert!(err.to_string().contains("service"));
        assert!(err.to_string().contains("reason"));
    }

    #[test]
    fn test_storage() {
        let err = EndpointError::storage("database connection failed");
        assert_eq!(err.error_code(), "ENDPOINT_STORAGE_ERROR");
        assert_eq!(err.to_grpc_code(), 14);
        assert!(err.to_string().contains("database connection failed"));
    }

    #[test]
    fn test_internal() {
        let err = EndpointError::internal("unexpected error");
        assert_eq!(err.error_code(), "ENDPOINT_INTERNAL_ERROR");
        assert_eq!(err.to_grpc_code(), 13);
        assert!(err.to_string().contains("unexpected error"));
    }

    // ========================================
    // gRPC Code Mapping Tests
    // ========================================

    #[test]
    fn test_grpc_code_not_found() {
        assert_eq!(EndpointError::not_found("").to_grpc_code(), 5);
        assert_eq!(EndpointError::unresolvable("", "").to_grpc_code(), 5);
    }

    #[test]
    fn test_grpc_code_unavailable() {
        assert_eq!(EndpointError::storage("").to_grpc_code(), 14);
    }

    #[test]
    fn test_grpc_code_internal() {
        assert_eq!(EndpointError::internal("").to_grpc_code(), 13);
    }

    // ========================================
    // Error Code Uniqueness Tests
    // ========================================

    #[test]
    fn test_all_error_codes_are_unique() {
        let codes = vec![
            EndpointError::not_found("").error_code(),
            EndpointError::unresolvable("", "").error_code(),
            EndpointError::storage("").error_code(),
            EndpointError::internal("").error_code(),
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
        let err = EndpointError::NotFound {
            service_name: "test-service".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("not found"));
        assert!(display.contains("test-service"));
    }

    #[test]
    fn test_display_unresolvable() {
        let err = EndpointError::UnresolvableService {
            service_name: "my-service".to_string(),
            reason: "DNS lookup failed".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("cannot resolve"));
        assert!(display.contains("my-service"));
        assert!(display.contains("DNS lookup failed"));
    }

    #[test]
    fn test_display_storage() {
        let err = EndpointError::Storage {
            message: "db error".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("storage error"));
        assert!(display.contains("db error"));
    }

    #[test]
    fn test_display_internal() {
        let err = EndpointError::Internal {
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
        let err = EndpointError::not_found("service");
        let debug = format!("{:?}", err);
        assert!(debug.contains("NotFound"));
    }

    // ========================================
    // std::error::Error Trait Tests
    // ========================================

    #[test]
    fn test_is_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(EndpointError::internal("test"));
        assert!(err.to_string().contains("test"));
    }

    // ========================================
    // Edge Case Tests
    // ========================================

    #[test]
    fn test_empty_strings() {
        let err = EndpointError::not_found("");
        assert!(err.to_string().contains("endpoint not found"));

        let err = EndpointError::unresolvable("", "");
        assert!(err.to_string().contains("cannot resolve"));

        let err = EndpointError::storage("");
        assert!(err.to_string().contains("storage error"));
    }

    #[test]
    fn test_long_strings() {
        let long_str = "x".repeat(10000);
        let err = EndpointError::storage(&long_str);
        assert!(err.to_string().contains(&long_str));
    }

    #[test]
    fn test_unicode_strings() {
        let err = EndpointError::not_found("サービス名");
        assert!(err.to_string().contains("サービス名"));
    }

    #[test]
    fn test_special_characters() {
        let err = EndpointError::unresolvable("service<test>", "reason: \"error\"");
        assert!(err.to_string().contains("service<test>"));
        assert!(err.to_string().contains("reason: \"error\""));
    }
}
