//! gRPC サーバエラー
//!
//! gRPC サーバ操作で発生するエラーを定義する。

use thiserror::Error;

/// gRPC サーバエラー
#[derive(Debug, Error)]
pub enum GrpcServerError {
    /// 設定エラー
    #[error("configuration error: {message}")]
    Config { message: String },

    /// 初期化エラー
    #[error("initialization error: {message}")]
    Init { message: String },

    /// インターセプタエラー
    #[error("interceptor error: {message}")]
    Interceptor { message: String },

    /// デッドラインエラー（クライアントがデッドラインを指定していない）
    #[error("deadline not specified by client")]
    DeadlineNotSpecified,

    /// デッドライン超過
    #[error("deadline exceeded")]
    DeadlineExceeded,
}

impl GrpcServerError {
    /// 設定エラーを作成
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// 初期化エラーを作成
    pub fn init(message: impl Into<String>) -> Self {
        Self::Init {
            message: message.into(),
        }
    }

    /// インターセプタエラーを作成
    pub fn interceptor(message: impl Into<String>) -> Self {
        Self::Interceptor {
            message: message.into(),
        }
    }

    /// gRPC ステータスコードに変換
    pub fn to_grpc_status_code(&self) -> i32 {
        match self {
            Self::Config { .. } | Self::Init { .. } => 13, // INTERNAL
            Self::Interceptor { .. } => 13,                // INTERNAL
            Self::DeadlineNotSpecified => 3,               // INVALID_ARGUMENT
            Self::DeadlineExceeded => 4,                   // DEADLINE_EXCEEDED
        }
    }
}

/// gRPC ステータスコード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrpcStatusCode {
    /// 成功
    Ok = 0,
    /// キャンセル
    Cancelled = 1,
    /// 不明
    Unknown = 2,
    /// 無効な引数
    InvalidArgument = 3,
    /// デッドライン超過
    DeadlineExceeded = 4,
    /// 見つからない
    NotFound = 5,
    /// 既に存在
    AlreadyExists = 6,
    /// 権限なし
    PermissionDenied = 7,
    /// リソース枯渇
    ResourceExhausted = 8,
    /// 事前条件失敗
    FailedPrecondition = 9,
    /// 中断
    Aborted = 10,
    /// 範囲外
    OutOfRange = 11,
    /// 未実装
    Unimplemented = 12,
    /// 内部エラー
    Internal = 13,
    /// 利用不可
    Unavailable = 14,
    /// データ損失
    DataLoss = 15,
    /// 未認証
    Unauthenticated = 16,
}

impl GrpcStatusCode {
    /// コードから作成
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

    /// ログレベルを取得
    pub fn log_level(&self) -> LogLevel {
        match self {
            Self::Ok => LogLevel::Info,
            Self::InvalidArgument
            | Self::NotFound
            | Self::AlreadyExists
            | Self::FailedPrecondition
            | Self::OutOfRange
            | Self::Unauthenticated
            | Self::PermissionDenied => LogLevel::Warn,
            _ => LogLevel::Error,
        }
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

impl std::fmt::Display for GrpcStatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name(), *self as i32)
    }
}

/// ログレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// デバッグ
    Debug,
    /// 情報
    Info,
    /// 警告
    Warn,
    /// エラー
    Error,
}

impl LogLevel {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_server_error_config() {
        let err = GrpcServerError::config("invalid setting");
        assert!(matches!(err, GrpcServerError::Config { .. }));
        assert_eq!(err.to_grpc_status_code(), 13);
    }

    #[test]
    fn test_grpc_server_error_deadline() {
        let err = GrpcServerError::DeadlineNotSpecified;
        assert_eq!(err.to_grpc_status_code(), 3);

        let err = GrpcServerError::DeadlineExceeded;
        assert_eq!(err.to_grpc_status_code(), 4);
    }

    #[test]
    fn test_grpc_status_code_from_code() {
        assert_eq!(GrpcStatusCode::from_code(0), GrpcStatusCode::Ok);
        assert_eq!(GrpcStatusCode::from_code(5), GrpcStatusCode::NotFound);
        assert_eq!(GrpcStatusCode::from_code(999), GrpcStatusCode::Unknown);
    }

    #[test]
    fn test_grpc_status_code_name() {
        assert_eq!(GrpcStatusCode::Ok.name(), "OK");
        assert_eq!(GrpcStatusCode::NotFound.name(), "NOT_FOUND");
    }

    #[test]
    fn test_grpc_status_code_log_level() {
        assert_eq!(GrpcStatusCode::Ok.log_level(), LogLevel::Info);
        assert_eq!(GrpcStatusCode::NotFound.log_level(), LogLevel::Warn);
        assert_eq!(GrpcStatusCode::Internal.log_level(), LogLevel::Error);
    }

    #[test]
    fn test_grpc_status_code_is_client_error() {
        assert!(GrpcStatusCode::InvalidArgument.is_client_error());
        assert!(GrpcStatusCode::NotFound.is_client_error());
        assert!(!GrpcStatusCode::Internal.is_client_error());
    }

    #[test]
    fn test_grpc_status_code_is_server_error() {
        assert!(GrpcStatusCode::Internal.is_server_error());
        assert!(GrpcStatusCode::Unavailable.is_server_error());
        assert!(!GrpcStatusCode::NotFound.is_server_error());
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }
}
