//! gRPC クライアントエラー
//!
//! gRPC クライアント操作で発生するエラーを定義する。

use thiserror::Error;

/// gRPC クライアントエラー
#[derive(Debug, Error)]
pub enum GrpcClientError {
    /// 設定エラー
    #[error("configuration error: {message}")]
    Config { message: String },

    /// タイムアウトエラー
    #[error("timeout: deadline exceeded after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// 接続エラー
    #[error("connection error: {message}")]
    Connection { message: String },

    /// サービスディスカバリエラー
    #[error("service discovery error: {message}")]
    ServiceDiscovery { message: String },

    /// gRPC ステータスエラー
    #[error("grpc status error: {status} - {message}")]
    Status {
        status: GrpcStatus,
        message: String,
        error_code: Option<String>,
    },

    /// インターセプタエラー
    #[error("interceptor error: {message}")]
    Interceptor { message: String },
}

impl GrpcClientError {
    /// 設定エラーを作成
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// タイムアウトエラーを作成
    pub fn timeout(timeout_ms: u64) -> Self {
        Self::Timeout { timeout_ms }
    }

    /// 接続エラーを作成
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
        }
    }

    /// サービスディスカバリエラーを作成
    pub fn service_discovery(message: impl Into<String>) -> Self {
        Self::ServiceDiscovery {
            message: message.into(),
        }
    }

    /// gRPC ステータスエラーを作成
    pub fn status(status: GrpcStatus, message: impl Into<String>) -> Self {
        Self::Status {
            status,
            message: message.into(),
            error_code: None,
        }
    }

    /// gRPC ステータスエラーを error_code 付きで作成
    pub fn status_with_code(
        status: GrpcStatus,
        message: impl Into<String>,
        error_code: impl Into<String>,
    ) -> Self {
        Self::Status {
            status,
            message: message.into(),
            error_code: Some(error_code.into()),
        }
    }

    /// インターセプタエラーを作成
    pub fn interceptor(message: impl Into<String>) -> Self {
        Self::Interceptor {
            message: message.into(),
        }
    }

    /// error_code を取得
    pub fn error_code(&self) -> Option<&str> {
        match self {
            Self::Status { error_code, .. } => error_code.as_deref(),
            _ => None,
        }
    }

    /// リトライ可能かどうか
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Timeout { .. } => false,
            Self::Connection { .. } => true,
            Self::Status { status, .. } => status.is_retryable(),
            _ => false,
        }
    }
}

/// gRPC ステータスコード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrpcStatus {
    /// 成功
    Ok,
    /// キャンセル
    Cancelled,
    /// 不明
    Unknown,
    /// 無効な引数
    InvalidArgument,
    /// デッドライン超過
    DeadlineExceeded,
    /// 見つからない
    NotFound,
    /// 既に存在
    AlreadyExists,
    /// 権限なし
    PermissionDenied,
    /// リソース枯渇
    ResourceExhausted,
    /// 事前条件失敗
    FailedPrecondition,
    /// 中断
    Aborted,
    /// 範囲外
    OutOfRange,
    /// 未実装
    Unimplemented,
    /// 内部エラー
    Internal,
    /// 利用不可
    Unavailable,
    /// データ損失
    DataLoss,
    /// 未認証
    Unauthenticated,
}

impl GrpcStatus {
    /// ステータスコード（数値）を取得
    pub fn code(&self) -> i32 {
        match self {
            Self::Ok => 0,
            Self::Cancelled => 1,
            Self::Unknown => 2,
            Self::InvalidArgument => 3,
            Self::DeadlineExceeded => 4,
            Self::NotFound => 5,
            Self::AlreadyExists => 6,
            Self::PermissionDenied => 7,
            Self::ResourceExhausted => 8,
            Self::FailedPrecondition => 9,
            Self::Aborted => 10,
            Self::OutOfRange => 11,
            Self::Unimplemented => 12,
            Self::Internal => 13,
            Self::Unavailable => 14,
            Self::DataLoss => 15,
            Self::Unauthenticated => 16,
        }
    }

    /// ステータス名を取得
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Cancelled => "CANCELLED",
            Self::Unknown => "UNKNOWN",
            Self::InvalidArgument => "INVALID_ARGUMENT",
            Self::DeadlineExceeded => "DEADLINE_EXCEEDED",
            Self::NotFound => "NOT_FOUND",
            Self::AlreadyExists => "ALREADY_EXISTS",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::ResourceExhausted => "RESOURCE_EXHAUSTED",
            Self::FailedPrecondition => "FAILED_PRECONDITION",
            Self::Aborted => "ABORTED",
            Self::OutOfRange => "OUT_OF_RANGE",
            Self::Unimplemented => "UNIMPLEMENTED",
            Self::Internal => "INTERNAL",
            Self::Unavailable => "UNAVAILABLE",
            Self::DataLoss => "DATA_LOSS",
            Self::Unauthenticated => "UNAUTHENTICATED",
        }
    }

    /// ステータスコードから作成
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Ok,
            1 => Self::Cancelled,
            2 => Self::Unknown,
            3 => Self::InvalidArgument,
            4 => Self::DeadlineExceeded,
            5 => Self::NotFound,
            6 => Self::AlreadyExists,
            7 => Self::PermissionDenied,
            8 => Self::ResourceExhausted,
            9 => Self::FailedPrecondition,
            10 => Self::Aborted,
            11 => Self::OutOfRange,
            12 => Self::Unimplemented,
            13 => Self::Internal,
            14 => Self::Unavailable,
            15 => Self::DataLoss,
            16 => Self::Unauthenticated,
            _ => Self::Unknown,
        }
    }

    /// リトライ可能かどうか
    ///
    /// 原則としてリトライは禁止だが、一部のステータスは
    /// 明示的な opt-in で許可される可能性がある。
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Unavailable | Self::ResourceExhausted)
    }

    /// クライアント起因のエラーかどうか
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::InvalidArgument
                | Self::NotFound
                | Self::AlreadyExists
                | Self::PermissionDenied
                | Self::FailedPrecondition
                | Self::OutOfRange
                | Self::Unauthenticated
        )
    }

    /// サーバー起因のエラーかどうか
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Self::Unknown
                | Self::Internal
                | Self::Unavailable
                | Self::DataLoss
                | Self::Unimplemented
        )
    }
}

impl std::fmt::Display for GrpcStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name(), self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_client_error_config() {
        let err = GrpcClientError::config("invalid timeout");
        assert!(matches!(err, GrpcClientError::Config { .. }));
        assert!(err.to_string().contains("invalid timeout"));
    }

    #[test]
    fn test_grpc_client_error_timeout() {
        let err = GrpcClientError::timeout(5000);
        assert!(matches!(err, GrpcClientError::Timeout { timeout_ms: 5000 }));
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_grpc_client_error_connection() {
        let err = GrpcClientError::connection("refused");
        assert!(err.is_retryable());
    }

    #[test]
    fn test_grpc_client_error_status() {
        let err = GrpcClientError::status(GrpcStatus::NotFound, "resource not found");
        assert!(!err.is_retryable());
        assert!(err.error_code().is_none());
    }

    #[test]
    fn test_grpc_client_error_status_with_code() {
        let err =
            GrpcClientError::status_with_code(GrpcStatus::NotFound, "user not found", "USER_NOT_FOUND");
        assert_eq!(err.error_code(), Some("USER_NOT_FOUND"));
    }

    #[test]
    fn test_grpc_status_code() {
        assert_eq!(GrpcStatus::Ok.code(), 0);
        assert_eq!(GrpcStatus::InvalidArgument.code(), 3);
        assert_eq!(GrpcStatus::Internal.code(), 13);
    }

    #[test]
    fn test_grpc_status_name() {
        assert_eq!(GrpcStatus::Ok.name(), "OK");
        assert_eq!(GrpcStatus::DeadlineExceeded.name(), "DEADLINE_EXCEEDED");
    }

    #[test]
    fn test_grpc_status_from_code() {
        assert_eq!(GrpcStatus::from_code(0), GrpcStatus::Ok);
        assert_eq!(GrpcStatus::from_code(5), GrpcStatus::NotFound);
        assert_eq!(GrpcStatus::from_code(999), GrpcStatus::Unknown);
    }

    #[test]
    fn test_grpc_status_is_retryable() {
        assert!(GrpcStatus::Unavailable.is_retryable());
        assert!(GrpcStatus::ResourceExhausted.is_retryable());
        assert!(!GrpcStatus::NotFound.is_retryable());
        assert!(!GrpcStatus::InvalidArgument.is_retryable());
    }

    #[test]
    fn test_grpc_status_is_client_error() {
        assert!(GrpcStatus::InvalidArgument.is_client_error());
        assert!(GrpcStatus::NotFound.is_client_error());
        assert!(!GrpcStatus::Internal.is_client_error());
    }

    #[test]
    fn test_grpc_status_is_server_error() {
        assert!(GrpcStatus::Internal.is_server_error());
        assert!(GrpcStatus::Unavailable.is_server_error());
        assert!(!GrpcStatus::NotFound.is_server_error());
    }

    #[test]
    fn test_grpc_status_display() {
        assert_eq!(format!("{}", GrpcStatus::Ok), "OK (0)");
        assert_eq!(format!("{}", GrpcStatus::NotFound), "NOT_FOUND (5)");
    }
}
