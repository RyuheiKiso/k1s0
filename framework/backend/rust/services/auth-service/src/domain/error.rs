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

    #[test]
    fn test_authentication_failed() {
        let err = AuthError::authentication_failed("invalid password");
        assert_eq!(err.error_code(), "AUTHENTICATION_FAILED");
        assert_eq!(err.to_grpc_code(), 16);
    }

    #[test]
    fn test_user_not_found() {
        let err = AuthError::user_not_found(123);
        assert_eq!(err.error_code(), "USER_NOT_FOUND");
        assert_eq!(err.to_grpc_code(), 5);
    }

    #[test]
    fn test_permission_denied() {
        let err = AuthError::permission_denied("user:write");
        assert_eq!(err.error_code(), "PERMISSION_DENIED");
        assert_eq!(err.to_grpc_code(), 7);
    }

    #[test]
    fn test_invalid_token() {
        let err = AuthError::invalid_token("malformed JWT");
        assert_eq!(err.error_code(), "INVALID_TOKEN");
        assert_eq!(err.to_grpc_code(), 16);
    }
}
