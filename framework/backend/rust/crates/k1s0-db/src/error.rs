//! データベースエラー
//!
//! DB操作で発生するエラーを統一的に扱う。
//! `error_code` による分類とステータスコード変換を提供。

use thiserror::Error;

/// データベースエラー
#[derive(Debug, Error)]
pub enum DbError {
    /// 接続エラー
    #[error("database connection failed: {message}")]
    Connection {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 接続タイムアウト
    #[error("database connection timeout after {timeout_ms}ms")]
    ConnectionTimeout { timeout_ms: u64 },

    /// プール枯渇
    #[error("connection pool exhausted: max {max_size} connections")]
    PoolExhausted { max_size: u32 },

    /// クエリタイムアウト
    #[error("query timeout after {timeout_ms}ms")]
    QueryTimeout { timeout_ms: u64 },

    /// レコードが見つからない
    #[error("record not found: {entity}")]
    NotFound { entity: String, key: String },

    /// 一意制約違反
    #[error("unique constraint violation: {constraint}")]
    UniqueViolation { constraint: String, column: String },

    /// 外部キー制約違反
    #[error("foreign key constraint violation: {constraint}")]
    ForeignKeyViolation { constraint: String },

    /// 楽観的ロック衝突
    #[error("optimistic lock conflict: {entity}")]
    OptimisticLockConflict { entity: String, key: String },

    /// トランザクションエラー
    #[error("transaction error: {message}")]
    Transaction { message: String },

    /// トランザクションロールバック
    #[error("transaction rolled back: {reason}")]
    TransactionRolledBack { reason: String },

    /// マイグレーションエラー
    #[error("migration failed: {message}")]
    Migration {
        message: String,
        version: Option<String>,
    },

    /// 設定エラー
    #[error("database configuration error: {message}")]
    Config { message: String },

    /// 内部エラー
    #[error("internal database error: {message}")]
    Internal { message: String },
}

impl DbError {
    /// 接続エラーを作成
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
            source: None,
        }
    }

    /// 接続エラーを作成（原因付き）
    pub fn connection_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Connection {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// 接続タイムアウトエラーを作成
    pub fn connection_timeout(timeout_ms: u64) -> Self {
        Self::ConnectionTimeout { timeout_ms }
    }

    /// プール枯渇エラーを作成
    pub fn pool_exhausted(max_size: u32) -> Self {
        Self::PoolExhausted { max_size }
    }

    /// クエリタイムアウトエラーを作成
    pub fn query_timeout(timeout_ms: u64) -> Self {
        Self::QueryTimeout { timeout_ms }
    }

    /// NotFoundエラーを作成
    pub fn not_found(entity: impl Into<String>, key: impl Into<String>) -> Self {
        Self::NotFound {
            entity: entity.into(),
            key: key.into(),
        }
    }

    /// 一意制約違反エラーを作成
    pub fn unique_violation(constraint: impl Into<String>, column: impl Into<String>) -> Self {
        Self::UniqueViolation {
            constraint: constraint.into(),
            column: column.into(),
        }
    }

    /// 外部キー制約違反エラーを作成
    pub fn foreign_key_violation(constraint: impl Into<String>) -> Self {
        Self::ForeignKeyViolation {
            constraint: constraint.into(),
        }
    }

    /// 楽観的ロック衝突エラーを作成
    pub fn optimistic_lock_conflict(entity: impl Into<String>, key: impl Into<String>) -> Self {
        Self::OptimisticLockConflict {
            entity: entity.into(),
            key: key.into(),
        }
    }

    /// トランザクションエラーを作成
    pub fn transaction(message: impl Into<String>) -> Self {
        Self::Transaction {
            message: message.into(),
        }
    }

    /// トランザクションロールバックエラーを作成
    pub fn transaction_rolled_back(reason: impl Into<String>) -> Self {
        Self::TransactionRolledBack {
            reason: reason.into(),
        }
    }

    /// マイグレーションエラーを作成
    pub fn migration(message: impl Into<String>) -> Self {
        Self::Migration {
            message: message.into(),
            version: None,
        }
    }

    /// マイグレーションエラーを作成（バージョン付き）
    pub fn migration_with_version(message: impl Into<String>, version: impl Into<String>) -> Self {
        Self::Migration {
            message: message.into(),
            version: Some(version.into()),
        }
    }

    /// 設定エラーを作成
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// 内部エラーを作成
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// クエリエラーを作成（内部エラーとして扱う）
    pub fn query(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// error_code を取得
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Connection { .. } => "DB_CONNECTION_ERROR",
            Self::ConnectionTimeout { .. } => "DB_CONNECTION_TIMEOUT",
            Self::PoolExhausted { .. } => "DB_POOL_EXHAUSTED",
            Self::QueryTimeout { .. } => "DB_QUERY_TIMEOUT",
            Self::NotFound { .. } => "DB_NOT_FOUND",
            Self::UniqueViolation { .. } => "DB_UNIQUE_VIOLATION",
            Self::ForeignKeyViolation { .. } => "DB_FK_VIOLATION",
            Self::OptimisticLockConflict { .. } => "DB_OPTIMISTIC_LOCK_CONFLICT",
            Self::Transaction { .. } => "DB_TRANSACTION_ERROR",
            Self::TransactionRolledBack { .. } => "DB_TRANSACTION_ROLLED_BACK",
            Self::Migration { .. } => "DB_MIGRATION_ERROR",
            Self::Config { .. } => "DB_CONFIG_ERROR",
            Self::Internal { .. } => "DB_INTERNAL_ERROR",
        }
    }

    /// リトライ可能かどうか
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Connection { .. }
                | Self::ConnectionTimeout { .. }
                | Self::PoolExhausted { .. }
                | Self::QueryTimeout { .. }
                | Self::OptimisticLockConflict { .. }
        )
    }

    /// クライアントエラーかどうか
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::NotFound { .. }
                | Self::UniqueViolation { .. }
                | Self::ForeignKeyViolation { .. }
                | Self::Config { .. }
        )
    }

    /// gRPC ステータスコードに変換
    pub fn to_grpc_status_code(&self) -> i32 {
        match self {
            Self::NotFound { .. } => 5,                // NOT_FOUND
            Self::UniqueViolation { .. } => 6,         // ALREADY_EXISTS
            Self::ForeignKeyViolation { .. } => 9,     // FAILED_PRECONDITION
            Self::OptimisticLockConflict { .. } => 10, // ABORTED
            Self::ConnectionTimeout { .. } => 4,       // DEADLINE_EXCEEDED
            Self::QueryTimeout { .. } => 4,            // DEADLINE_EXCEEDED
            Self::PoolExhausted { .. } => 8,           // RESOURCE_EXHAUSTED
            Self::Connection { .. } => 14,             // UNAVAILABLE
            Self::Config { .. } => 3,                  // INVALID_ARGUMENT
            _ => 13,                                   // INTERNAL
        }
    }

    /// HTTP ステータスコードに変換
    pub fn to_http_status_code(&self) -> u16 {
        match self {
            Self::NotFound { .. } => 404,               // Not Found
            Self::UniqueViolation { .. } => 409,        // Conflict
            Self::ForeignKeyViolation { .. } => 400,    // Bad Request
            Self::OptimisticLockConflict { .. } => 409, // Conflict
            Self::ConnectionTimeout { .. } => 504,      // Gateway Timeout
            Self::QueryTimeout { .. } => 504,           // Gateway Timeout
            Self::PoolExhausted { .. } => 503,          // Service Unavailable
            Self::Connection { .. } => 503,             // Service Unavailable
            Self::Config { .. } => 500,                 // Internal Server Error
            _ => 500,                                   // Internal Server Error
        }
    }
}

/// データベース操作の結果型
pub type DbResult<T> = Result<T, DbError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error() {
        let err = DbError::connection("failed to connect");
        assert_eq!(err.error_code(), "DB_CONNECTION_ERROR");
        assert!(err.is_retryable());
        assert!(!err.is_client_error());
    }

    #[test]
    fn test_not_found_error() {
        let err = DbError::not_found("User", "123");
        assert_eq!(err.error_code(), "DB_NOT_FOUND");
        assert!(!err.is_retryable());
        assert!(err.is_client_error());
        assert_eq!(err.to_http_status_code(), 404);
        assert_eq!(err.to_grpc_status_code(), 5);
    }

    #[test]
    fn test_unique_violation_error() {
        let err = DbError::unique_violation("users_email_key", "email");
        assert_eq!(err.error_code(), "DB_UNIQUE_VIOLATION");
        assert!(err.is_client_error());
        assert_eq!(err.to_http_status_code(), 409);
    }

    #[test]
    fn test_optimistic_lock_conflict() {
        let err = DbError::optimistic_lock_conflict("Order", "456");
        assert_eq!(err.error_code(), "DB_OPTIMISTIC_LOCK_CONFLICT");
        assert!(err.is_retryable());
        assert_eq!(err.to_grpc_status_code(), 10);
    }

    #[test]
    fn test_timeout_errors() {
        let conn_timeout = DbError::connection_timeout(5000);
        assert_eq!(conn_timeout.error_code(), "DB_CONNECTION_TIMEOUT");
        assert!(conn_timeout.is_retryable());

        let query_timeout = DbError::query_timeout(30000);
        assert_eq!(query_timeout.error_code(), "DB_QUERY_TIMEOUT");
        assert!(query_timeout.is_retryable());
    }

    #[test]
    fn test_migration_error() {
        let err = DbError::migration_with_version("failed to apply", "0001");
        assert_eq!(err.error_code(), "DB_MIGRATION_ERROR");
        assert!(!err.is_retryable());
    }
}
