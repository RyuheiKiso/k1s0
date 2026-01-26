//! エラーコードとステータスコード変換
//!
//! 運用で「見れば分かる」を実現するための安定した識別子と、
//! gRPC/HTTP ステータスコードへの統一的な変換を提供する。

use std::fmt;
use serde::{Deserialize, Serialize};

/// エラーコード
///
/// 運用で「見れば分かる」を実現するための安定した識別子。
/// application 層でドメインエラーに付与し、presentation 層で外部に公開する。
///
/// # 命名規則
///
/// - SCREAMING_SNAKE_CASE を使用
/// - プレフィックスでカテゴリを示す（例: `USER_`, `ORDER_`, `AUTH_`）
/// - 一度公開したコードは変更しない（後方互換性）
///
/// # 例
///
/// - `USER_NOT_FOUND`: ユーザーが見つからない
/// - `AUTH_TOKEN_EXPIRED`: 認証トークンが期限切れ
/// - `ORDER_ALREADY_CANCELLED`: 注文は既にキャンセル済み
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ErrorCode {
    /// エラーコード文字列
    code: String,
}

impl ErrorCode {
    /// 新しいエラーコードを作成
    ///
    /// # Arguments
    ///
    /// * `code` - エラーコード（SCREAMING_SNAKE_CASE 推奨）
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }

    /// エラーコード文字列を取得
    pub fn as_str(&self) -> &str {
        &self.code
    }

    // === 共通エラーコード ===

    /// 内部エラー
    pub fn internal() -> Self {
        Self::new("INTERNAL_ERROR")
    }

    /// 不明なエラー
    pub fn unknown() -> Self {
        Self::new("UNKNOWN_ERROR")
    }

    /// バリデーションエラー
    pub fn validation_error() -> Self {
        Self::new("VALIDATION_ERROR")
    }

    /// リソースが見つからない
    pub fn not_found() -> Self {
        Self::new("NOT_FOUND")
    }

    /// 重複エラー
    pub fn duplicate() -> Self {
        Self::new("DUPLICATE")
    }

    /// 競合エラー
    pub fn conflict() -> Self {
        Self::new("CONFLICT")
    }

    /// 認証エラー
    pub fn unauthenticated() -> Self {
        Self::new("UNAUTHENTICATED")
    }

    /// 認可エラー
    pub fn permission_denied() -> Self {
        Self::new("PERMISSION_DENIED")
    }

    /// 依存障害
    pub fn dependency_failure() -> Self {
        Self::new("DEPENDENCY_FAILURE")
    }

    /// 一時障害
    pub fn transient() -> Self {
        Self::new("TRANSIENT_ERROR")
    }

    /// レート制限
    pub fn rate_limited() -> Self {
        Self::new("RATE_LIMITED")
    }

    /// タイムアウト
    pub fn timeout() -> Self {
        Self::new("TIMEOUT")
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl From<&str> for ErrorCode {
    fn from(code: &str) -> Self {
        Self::new(code)
    }
}

impl From<String> for ErrorCode {
    fn from(code: String) -> Self {
        Self::new(code)
    }
}

/// gRPC ステータスコード
///
/// <https://grpc.io/docs/guides/status-codes/>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum GrpcCode {
    /// 成功
    Ok = 0,
    /// キャンセル
    Cancelled = 1,
    /// 不明なエラー
    Unknown = 2,
    /// 無効な引数
    InvalidArgument = 3,
    /// タイムアウト
    DeadlineExceeded = 4,
    /// 見つからない
    NotFound = 5,
    /// 既に存在する
    AlreadyExists = 6,
    /// 権限なし
    PermissionDenied = 7,
    /// リソース枯渇
    ResourceExhausted = 8,
    /// 前提条件違反
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

impl GrpcCode {
    /// 数値に変換
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 数値から変換
    pub fn from_i32(code: i32) -> Self {
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

    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
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

    /// HTTP ステータスコードに変換
    pub fn to_http_status(&self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::Cancelled => 499,
            Self::Unknown => 500,
            Self::InvalidArgument => 400,
            Self::DeadlineExceeded => 504,
            Self::NotFound => 404,
            Self::AlreadyExists => 409,
            Self::PermissionDenied => 403,
            Self::ResourceExhausted => 429,
            Self::FailedPrecondition => 400,
            Self::Aborted => 409,
            Self::OutOfRange => 400,
            Self::Unimplemented => 501,
            Self::Internal => 500,
            Self::Unavailable => 503,
            Self::DataLoss => 500,
            Self::Unauthenticated => 401,
        }
    }

    /// リトライ可能かどうか
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::DeadlineExceeded
                | Self::ResourceExhausted
                | Self::Aborted
                | Self::Unavailable
        )
    }
}

/// HTTP ステータスコード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpStatus(pub u16);

impl HttpStatus {
    // 成功系
    pub const OK: Self = Self(200);
    pub const CREATED: Self = Self(201);
    pub const ACCEPTED: Self = Self(202);
    pub const NO_CONTENT: Self = Self(204);

    // クライアントエラー系
    pub const BAD_REQUEST: Self = Self(400);
    pub const UNAUTHORIZED: Self = Self(401);
    pub const FORBIDDEN: Self = Self(403);
    pub const NOT_FOUND: Self = Self(404);
    pub const METHOD_NOT_ALLOWED: Self = Self(405);
    pub const CONFLICT: Self = Self(409);
    pub const GONE: Self = Self(410);
    pub const UNPROCESSABLE_ENTITY: Self = Self(422);
    pub const TOO_MANY_REQUESTS: Self = Self(429);

    // サーバーエラー系
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);
    pub const NOT_IMPLEMENTED: Self = Self(501);
    pub const BAD_GATEWAY: Self = Self(502);
    pub const SERVICE_UNAVAILABLE: Self = Self(503);
    pub const GATEWAY_TIMEOUT: Self = Self(504);

    /// 数値を取得
    pub fn as_u16(&self) -> u16 {
        self.0
    }

    /// 成功かどうか
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.0)
    }

    /// クライアントエラーかどうか
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.0)
    }

    /// サーバーエラーかどうか
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.0)
    }

    /// gRPC コードに変換
    pub fn to_grpc_code(&self) -> GrpcCode {
        match self.0 {
            200..=299 => GrpcCode::Ok,
            400 => GrpcCode::InvalidArgument,
            401 => GrpcCode::Unauthenticated,
            403 => GrpcCode::PermissionDenied,
            404 => GrpcCode::NotFound,
            409 => GrpcCode::AlreadyExists,
            429 => GrpcCode::ResourceExhausted,
            499 => GrpcCode::Cancelled,
            501 => GrpcCode::Unimplemented,
            503 => GrpcCode::Unavailable,
            504 => GrpcCode::DeadlineExceeded,
            _ if self.is_client_error() => GrpcCode::FailedPrecondition,
            _ => GrpcCode::Internal,
        }
    }

    /// 理由フレーズを取得
    pub fn reason_phrase(&self) -> &'static str {
        match self.0 {
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            409 => "Conflict",
            410 => "Gone",
            422 => "Unprocessable Entity",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            _ => "Unknown",
        }
    }
}

impl From<u16> for HttpStatus {
    fn from(code: u16) -> Self {
        Self(code)
    }
}

impl From<HttpStatus> for u16 {
    fn from(status: HttpStatus) -> Self {
        status.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let code = ErrorCode::new("USER_NOT_FOUND");
        assert_eq!(code.as_str(), "USER_NOT_FOUND");
    }

    #[test]
    fn test_display() {
        let code = ErrorCode::new("ORDER_CANCELLED");
        assert_eq!(format!("{}", code), "ORDER_CANCELLED");
    }

    #[test]
    fn test_from_str() {
        let code: ErrorCode = "TEST_CODE".into();
        assert_eq!(code.as_str(), "TEST_CODE");
    }

    #[test]
    fn test_from_string() {
        let code: ErrorCode = String::from("TEST_CODE").into();
        assert_eq!(code.as_str(), "TEST_CODE");
    }

    #[test]
    fn test_common_codes() {
        assert_eq!(ErrorCode::internal().as_str(), "INTERNAL_ERROR");
        assert_eq!(ErrorCode::not_found().as_str(), "NOT_FOUND");
        assert_eq!(ErrorCode::duplicate().as_str(), "DUPLICATE");
        assert_eq!(ErrorCode::unauthenticated().as_str(), "UNAUTHENTICATED");
        assert_eq!(ErrorCode::permission_denied().as_str(), "PERMISSION_DENIED");
    }

    #[test]
    fn test_equality() {
        let code1 = ErrorCode::new("TEST");
        let code2 = ErrorCode::new("TEST");
        let code3 = ErrorCode::new("OTHER");

        assert_eq!(code1, code2);
        assert_ne!(code1, code3);
    }

    #[test]
    fn test_clone() {
        let code1 = ErrorCode::new("TEST");
        let code2 = code1.clone();
        assert_eq!(code1, code2);
    }

    // GrpcCode tests

    #[test]
    fn test_grpc_code_as_i32() {
        assert_eq!(GrpcCode::Ok.as_i32(), 0);
        assert_eq!(GrpcCode::NotFound.as_i32(), 5);
        assert_eq!(GrpcCode::Internal.as_i32(), 13);
    }

    #[test]
    fn test_grpc_code_from_i32() {
        assert_eq!(GrpcCode::from_i32(0), GrpcCode::Ok);
        assert_eq!(GrpcCode::from_i32(5), GrpcCode::NotFound);
        assert_eq!(GrpcCode::from_i32(999), GrpcCode::Unknown);
    }

    #[test]
    fn test_grpc_code_to_http() {
        assert_eq!(GrpcCode::Ok.to_http_status(), 200);
        assert_eq!(GrpcCode::NotFound.to_http_status(), 404);
        assert_eq!(GrpcCode::Unauthenticated.to_http_status(), 401);
        assert_eq!(GrpcCode::PermissionDenied.to_http_status(), 403);
        assert_eq!(GrpcCode::Internal.to_http_status(), 500);
    }

    #[test]
    fn test_grpc_code_retryable() {
        assert!(GrpcCode::Unavailable.is_retryable());
        assert!(GrpcCode::DeadlineExceeded.is_retryable());
        assert!(!GrpcCode::NotFound.is_retryable());
        assert!(!GrpcCode::InvalidArgument.is_retryable());
    }

    // HttpStatus tests

    #[test]
    fn test_http_status_success() {
        assert!(HttpStatus::OK.is_success());
        assert!(HttpStatus::CREATED.is_success());
        assert!(!HttpStatus::BAD_REQUEST.is_success());
    }

    #[test]
    fn test_http_status_error() {
        assert!(HttpStatus::BAD_REQUEST.is_client_error());
        assert!(HttpStatus::NOT_FOUND.is_client_error());
        assert!(HttpStatus::INTERNAL_SERVER_ERROR.is_server_error());
        assert!(HttpStatus::SERVICE_UNAVAILABLE.is_server_error());
    }

    #[test]
    fn test_http_status_to_grpc() {
        assert_eq!(HttpStatus::OK.to_grpc_code(), GrpcCode::Ok);
        assert_eq!(HttpStatus::NOT_FOUND.to_grpc_code(), GrpcCode::NotFound);
        assert_eq!(HttpStatus::UNAUTHORIZED.to_grpc_code(), GrpcCode::Unauthenticated);
    }

    #[test]
    fn test_http_status_reason() {
        assert_eq!(HttpStatus::OK.reason_phrase(), "OK");
        assert_eq!(HttpStatus::NOT_FOUND.reason_phrase(), "Not Found");
        assert_eq!(HttpStatus::INTERNAL_SERVER_ERROR.reason_phrase(), "Internal Server Error");
    }
}
