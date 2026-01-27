//! 認証エラー型

use thiserror::Error;

/// 認証エラー
#[derive(Debug, Error)]
pub enum AuthError {
    /// 認証失敗（ユーザーが見つからない、パスワードが間違っている等）
    #[error("authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    /// ユーザーが見つからない
    #[error("user not found: {user_id}")]
    UserNotFound { user_id: i64 },

    /// アカウントがロックされている
    #[error("account locked: {user_id}")]
    AccountLocked { user_id: i64 },

    /// アカウントが無効
    #[error("account inactive: {user_id}")]
    AccountInactive { user_id: i64 },

    /// トークンが無効
    #[error("invalid token: {reason}")]
    InvalidToken { reason: String },

    /// トークンの有効期限切れ
    #[error("token expired")]
    TokenExpired,

    /// パーミッションが拒否された
    #[error("permission denied: {permission_key}")]
    PermissionDenied { permission_key: String },

    /// ストレージエラー
    #[error("storage error: {message}")]
    Storage { message: String },

    /// 内部エラー
    #[error("internal error: {message}")]
    Internal { message: String },
}

impl AuthError {
    /// 認証失敗エラーを作成
    pub fn authentication_failed(reason: impl Into<String>) -> Self {
        Self::AuthenticationFailed {
            reason: reason.into(),
        }
    }

    /// ユーザーが見つからないエラーを作成
    pub fn user_not_found(user_id: i64) -> Self {
        Self::UserNotFound { user_id }
    }

    /// アカウントロックエラーを作成
    pub fn account_locked(user_id: i64) -> Self {
        Self::AccountLocked { user_id }
    }

    /// アカウント無効エラーを作成
    pub fn account_inactive(user_id: i64) -> Self {
        Self::AccountInactive { user_id }
    }

    /// 無効なトークンエラーを作成
    pub fn invalid_token(reason: impl Into<String>) -> Self {
        Self::InvalidToken {
            reason: reason.into(),
        }
    }

    /// パーミッション拒否エラーを作成
    pub fn permission_denied(permission_key: impl Into<String>) -> Self {
        Self::PermissionDenied {
            permission_key: permission_key.into(),
        }
    }

    /// ストレージエラーを作成
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage {
            message: message.into(),
        }
    }

    /// 内部エラーを作成
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// エラーコードを取得
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::AuthenticationFailed { .. } => "AUTHENTICATION_FAILED",
            Self::UserNotFound { .. } => "USER_NOT_FOUND",
            Self::AccountLocked { .. } => "ACCOUNT_LOCKED",
            Self::AccountInactive { .. } => "ACCOUNT_INACTIVE",
            Self::InvalidToken { .. } => "INVALID_TOKEN",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::PermissionDenied { .. } => "PERMISSION_DENIED",
            Self::Storage { .. } => "STORAGE_ERROR",
            Self::Internal { .. } => "INTERNAL_ERROR",
        }
    }

    /// gRPCステータスコードを取得
    pub fn to_grpc_code(&self) -> i32 {
        match self {
            Self::AuthenticationFailed { .. } => 16, // UNAUTHENTICATED
            Self::UserNotFound { .. } => 5,          // NOT_FOUND
            Self::AccountLocked { .. } => 7,         // PERMISSION_DENIED
            Self::AccountInactive { .. } => 7,       // PERMISSION_DENIED
            Self::InvalidToken { .. } => 16,         // UNAUTHENTICATED
            Self::TokenExpired => 16,                // UNAUTHENTICATED
            Self::PermissionDenied { .. } => 7,      // PERMISSION_DENIED
            Self::Storage { .. } => 13,              // INTERNAL
            Self::Internal { .. } => 13,             // INTERNAL
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Error Creation Tests
    // ========================================

    #[test]
    fn test_authentication_failed() {
        let err = AuthError::authentication_failed("invalid password");
        assert_eq!(err.error_code(), "AUTHENTICATION_FAILED");
        assert_eq!(err.to_grpc_code(), 16);
        assert!(err.to_string().contains("invalid password"));
    }

    #[test]
    fn test_authentication_failed_with_string() {
        let err = AuthError::authentication_failed(String::from("bad credentials"));
        assert!(err.to_string().contains("bad credentials"));
    }

    #[test]
    fn test_user_not_found() {
        let err = AuthError::user_not_found(123);
        assert_eq!(err.error_code(), "USER_NOT_FOUND");
        assert_eq!(err.to_grpc_code(), 5);
        assert!(err.to_string().contains("123"));
    }

    #[test]
    fn test_user_not_found_with_zero_id() {
        let err = AuthError::user_not_found(0);
        assert!(err.to_string().contains("0"));
    }

    #[test]
    fn test_user_not_found_with_negative_id() {
        let err = AuthError::user_not_found(-1);
        assert!(err.to_string().contains("-1"));
    }

    #[test]
    fn test_account_locked() {
        let err = AuthError::account_locked(456);
        assert_eq!(err.error_code(), "ACCOUNT_LOCKED");
        assert_eq!(err.to_grpc_code(), 7);
        assert!(err.to_string().contains("456"));
    }

    #[test]
    fn test_account_inactive() {
        let err = AuthError::account_inactive(789);
        assert_eq!(err.error_code(), "ACCOUNT_INACTIVE");
        assert_eq!(err.to_grpc_code(), 7);
        assert!(err.to_string().contains("789"));
    }

    #[test]
    fn test_invalid_token() {
        let err = AuthError::invalid_token("malformed JWT");
        assert_eq!(err.error_code(), "INVALID_TOKEN");
        assert_eq!(err.to_grpc_code(), 16);
        assert!(err.to_string().contains("malformed JWT"));
    }

    #[test]
    fn test_token_expired() {
        let err = AuthError::TokenExpired;
        assert_eq!(err.error_code(), "TOKEN_EXPIRED");
        assert_eq!(err.to_grpc_code(), 16);
        assert!(err.to_string().contains("expired"));
    }

    #[test]
    fn test_permission_denied() {
        let err = AuthError::permission_denied("user:write");
        assert_eq!(err.error_code(), "PERMISSION_DENIED");
        assert_eq!(err.to_grpc_code(), 7);
        assert!(err.to_string().contains("user:write"));
    }

    #[test]
    fn test_storage_error() {
        let err = AuthError::storage("database connection failed");
        assert_eq!(err.error_code(), "STORAGE_ERROR");
        assert_eq!(err.to_grpc_code(), 13);
        assert!(err.to_string().contains("database connection failed"));
    }

    #[test]
    fn test_internal_error() {
        let err = AuthError::internal("unexpected error occurred");
        assert_eq!(err.error_code(), "INTERNAL_ERROR");
        assert_eq!(err.to_grpc_code(), 13);
        assert!(err.to_string().contains("unexpected error occurred"));
    }

    // ========================================
    // gRPC Code Mapping Tests
    // ========================================

    #[test]
    fn test_grpc_code_unauthenticated() {
        // UNAUTHENTICATED = 16
        assert_eq!(AuthError::authentication_failed("").to_grpc_code(), 16);
        assert_eq!(AuthError::invalid_token("").to_grpc_code(), 16);
        assert_eq!(AuthError::TokenExpired.to_grpc_code(), 16);
    }

    #[test]
    fn test_grpc_code_not_found() {
        // NOT_FOUND = 5
        assert_eq!(AuthError::user_not_found(0).to_grpc_code(), 5);
    }

    #[test]
    fn test_grpc_code_permission_denied() {
        // PERMISSION_DENIED = 7
        assert_eq!(AuthError::account_locked(0).to_grpc_code(), 7);
        assert_eq!(AuthError::account_inactive(0).to_grpc_code(), 7);
        assert_eq!(AuthError::permission_denied("").to_grpc_code(), 7);
    }

    #[test]
    fn test_grpc_code_internal() {
        // INTERNAL = 13
        assert_eq!(AuthError::storage("").to_grpc_code(), 13);
        assert_eq!(AuthError::internal("").to_grpc_code(), 13);
    }

    // ========================================
    // Error Code Uniqueness Tests
    // ========================================

    #[test]
    fn test_all_error_codes_are_unique() {
        let codes = vec![
            AuthError::authentication_failed("").error_code(),
            AuthError::user_not_found(0).error_code(),
            AuthError::account_locked(0).error_code(),
            AuthError::account_inactive(0).error_code(),
            AuthError::invalid_token("").error_code(),
            AuthError::TokenExpired.error_code(),
            AuthError::permission_denied("").error_code(),
            AuthError::storage("").error_code(),
            AuthError::internal("").error_code(),
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
    fn test_display_authentication_failed() {
        let err = AuthError::AuthenticationFailed {
            reason: "test reason".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("authentication failed"));
        assert!(display.contains("test reason"));
    }

    #[test]
    fn test_display_user_not_found() {
        let err = AuthError::UserNotFound { user_id: 42 };
        let display = format!("{}", err);
        assert!(display.contains("user not found"));
        assert!(display.contains("42"));
    }

    #[test]
    fn test_display_account_locked() {
        let err = AuthError::AccountLocked { user_id: 99 };
        let display = format!("{}", err);
        assert!(display.contains("account locked"));
        assert!(display.contains("99"));
    }

    #[test]
    fn test_display_account_inactive() {
        let err = AuthError::AccountInactive { user_id: 77 };
        let display = format!("{}", err);
        assert!(display.contains("account inactive"));
        assert!(display.contains("77"));
    }

    #[test]
    fn test_display_invalid_token() {
        let err = AuthError::InvalidToken {
            reason: "bad format".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("invalid token"));
        assert!(display.contains("bad format"));
    }

    #[test]
    fn test_display_permission_denied() {
        let err = AuthError::PermissionDenied {
            permission_key: "admin:delete".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("permission denied"));
        assert!(display.contains("admin:delete"));
    }

    #[test]
    fn test_display_storage() {
        let err = AuthError::Storage {
            message: "db error".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("storage error"));
        assert!(display.contains("db error"));
    }

    #[test]
    fn test_display_internal() {
        let err = AuthError::Internal {
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
        let err = AuthError::authentication_failed("test");
        let debug = format!("{:?}", err);
        assert!(debug.contains("AuthenticationFailed"));
    }

    // ========================================
    // Edge Case Tests
    // ========================================

    #[test]
    fn test_empty_reason_strings() {
        let err = AuthError::authentication_failed("");
        assert!(err.to_string().contains("authentication failed"));

        let err = AuthError::invalid_token("");
        assert!(err.to_string().contains("invalid token"));

        let err = AuthError::permission_denied("");
        assert!(err.to_string().contains("permission denied"));
    }

    #[test]
    fn test_long_reason_strings() {
        let long_reason = "x".repeat(10000);
        let err = AuthError::authentication_failed(&long_reason);
        assert!(err.to_string().contains(&long_reason));
    }

    #[test]
    fn test_unicode_in_reason() {
        let err = AuthError::authentication_failed("無効なパスワード");
        assert!(err.to_string().contains("無効なパスワード"));
    }

    #[test]
    fn test_special_characters_in_reason() {
        let err = AuthError::authentication_failed("error: \"test\" <>&");
        assert!(err.to_string().contains("error: \"test\" <>&"));
    }
}
