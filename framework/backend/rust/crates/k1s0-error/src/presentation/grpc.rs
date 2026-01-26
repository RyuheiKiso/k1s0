//! gRPC エラー表現
//!
//! application 層のエラーを gRPC Status + metadata 形式で表現する。

use serde::{Deserialize, Serialize};

use crate::application::AppError;
use crate::domain::ErrorKind;

/// gRPC ステータスコード
///
/// gRPC の標準ステータスコード。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrpcStatusCode {
    /// 成功
    Ok = 0,
    /// キャンセル
    Cancelled = 1,
    /// 不明なエラー
    Unknown = 2,
    /// 引数不正
    InvalidArgument = 3,
    /// タイムアウト
    DeadlineExceeded = 4,
    /// リソースが見つからない
    NotFound = 5,
    /// 既に存在
    AlreadyExists = 6,
    /// 権限不足
    PermissionDenied = 7,
    /// リソース枯渇
    ResourceExhausted = 8,
    /// 事前条件不成立
    FailedPrecondition = 9,
    /// 処理中断
    Aborted = 10,
    /// 範囲外
    OutOfRange = 11,
    /// 未実装
    Unimplemented = 12,
    /// 内部エラー
    Internal = 13,
    /// サービス利用不可
    Unavailable = 14,
    /// データ損失
    DataLoss = 15,
    /// 認証エラー
    Unauthenticated = 16,
}

impl GrpcStatusCode {
    /// 数値に変換
    pub fn as_i32(self) -> i32 {
        self as i32
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
}

/// gRPC エラー
///
/// application 層のエラーを gRPC レスポンス向けに変換したもの。
#[derive(Debug, Clone)]
pub struct GrpcError {
    /// gRPC ステータスコード
    status_code: GrpcStatusCode,
    /// エラーコード（k1s0 固有）
    error_code: String,
    /// エラーメッセージ
    message: String,
    /// トレース ID
    trace_id: Option<String>,
    /// リクエスト ID
    request_id: Option<String>,
    /// ヒント
    hint: Option<String>,
    /// リソースの種類
    resource_type: Option<String>,
    /// リソースの ID
    resource_id: Option<String>,
}

impl GrpcError {
    /// AppError から GrpcError を作成
    pub fn from_app_error(app_error: &AppError) -> Self {
        let status_code = Self::kind_to_status_code(app_error.kind());
        let domain_error = app_error.domain_error();

        Self {
            status_code,
            error_code: app_error.error_code().to_string(),
            message: app_error.message().to_string(),
            trace_id: app_error.context().trace_id.clone(),
            request_id: app_error.context().request_id.clone(),
            hint: app_error.hint().map(|s| s.to_string()),
            resource_type: domain_error.resource_type().map(|s| s.to_string()),
            resource_id: domain_error.resource_id().map(|s| s.to_string()),
        }
    }

    /// エラーの種類を gRPC ステータスコードに変換
    fn kind_to_status_code(kind: ErrorKind) -> GrpcStatusCode {
        match kind {
            ErrorKind::InvalidInput | ErrorKind::InvariantViolation => {
                GrpcStatusCode::InvalidArgument
            }
            ErrorKind::Unauthorized => GrpcStatusCode::Unauthenticated,
            ErrorKind::Forbidden => GrpcStatusCode::PermissionDenied,
            ErrorKind::NotFound => GrpcStatusCode::NotFound,
            ErrorKind::Conflict => GrpcStatusCode::AlreadyExists,
            ErrorKind::DependencyFailure | ErrorKind::Transient => GrpcStatusCode::Unavailable,
            ErrorKind::Internal => GrpcStatusCode::Internal,
        }
    }

    /// gRPC ステータスコードを取得
    pub fn status_code(&self) -> GrpcStatusCode {
        self.status_code
    }

    /// エラーコードを取得
    pub fn error_code(&self) -> &str {
        &self.error_code
    }

    /// エラーメッセージを取得
    pub fn message(&self) -> &str {
        &self.message
    }

    /// トレース ID を取得
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    /// リクエスト ID を取得
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// ヒントを取得
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    /// エラー詳細を JSON 形式で取得（metadata 用）
    pub fn to_details(&self) -> GrpcErrorDetails {
        GrpcErrorDetails {
            error_code: self.error_code.clone(),
            message: self.message.clone(),
            trace_id: self.trace_id.clone(),
            request_id: self.request_id.clone(),
            hint: self.hint.clone(),
            resource_type: self.resource_type.clone(),
            resource_id: self.resource_id.clone(),
        }
    }

    /// エラー詳細を JSON 文字列で取得
    pub fn to_details_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.to_details())
    }
}

/// gRPC エラー詳細
///
/// gRPC の metadata に含めるエラー詳細。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcErrorDetails {
    /// エラーコード
    pub error_code: String,

    /// エラーメッセージ
    pub message: String,

    /// トレース ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    /// リクエスト ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// ヒント
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,

    /// リソースの種類
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,

    /// リソースの ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
}

impl GrpcErrorDetails {
    /// JSON 文字列に変換
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomainError, ErrorCode};

    #[test]
    fn test_grpc_status_code() {
        assert_eq!(GrpcStatusCode::Ok.as_i32(), 0);
        assert_eq!(GrpcStatusCode::InvalidArgument.as_i32(), 3);
        assert_eq!(GrpcStatusCode::NotFound.as_i32(), 5);
        assert_eq!(GrpcStatusCode::Internal.as_i32(), 13);
    }

    #[test]
    fn test_grpc_status_code_as_str() {
        assert_eq!(GrpcStatusCode::Ok.as_str(), "OK");
        assert_eq!(GrpcStatusCode::InvalidArgument.as_str(), "INVALID_ARGUMENT");
        assert_eq!(GrpcStatusCode::NotFound.as_str(), "NOT_FOUND");
    }

    #[test]
    fn test_from_app_error_not_found() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"));
        let grpc_err = GrpcError::from_app_error(&app_err);

        assert_eq!(grpc_err.status_code(), GrpcStatusCode::NotFound);
        assert_eq!(grpc_err.error_code(), "USER_NOT_FOUND");
    }

    #[test]
    fn test_from_app_error_with_context() {
        let domain_err = DomainError::internal("test");
        let app_err = AppError::from_domain(domain_err, ErrorCode::internal())
            .with_trace_id("trace-123")
            .with_request_id("req-456");
        let grpc_err = GrpcError::from_app_error(&app_err);

        assert_eq!(grpc_err.trace_id(), Some("trace-123"));
        assert_eq!(grpc_err.request_id(), Some("req-456"));
    }

    #[test]
    fn test_status_code_mapping() {
        let cases = vec![
            (ErrorKind::InvalidInput, GrpcStatusCode::InvalidArgument),
            (ErrorKind::Unauthorized, GrpcStatusCode::Unauthenticated),
            (ErrorKind::Forbidden, GrpcStatusCode::PermissionDenied),
            (ErrorKind::NotFound, GrpcStatusCode::NotFound),
            (ErrorKind::Conflict, GrpcStatusCode::AlreadyExists),
            (ErrorKind::Internal, GrpcStatusCode::Internal),
            (ErrorKind::DependencyFailure, GrpcStatusCode::Unavailable),
            (ErrorKind::Transient, GrpcStatusCode::Unavailable),
        ];

        for (kind, expected_status) in cases {
            assert_eq!(GrpcError::kind_to_status_code(kind), expected_status);
        }
    }

    #[test]
    fn test_to_details() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"))
            .with_trace_id("trace-123");
        let grpc_err = GrpcError::from_app_error(&app_err);
        let details = grpc_err.to_details();

        assert_eq!(details.error_code, "USER_NOT_FOUND");
        assert_eq!(details.trace_id, Some("trace-123".to_string()));
    }

    #[test]
    fn test_to_details_json() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"));
        let grpc_err = GrpcError::from_app_error(&app_err);

        let json = grpc_err.to_details_json().unwrap();
        assert!(json.contains("USER_NOT_FOUND"));
    }

    #[test]
    fn test_details_deserialize() {
        let json = r#"{
            "error_code": "USER_NOT_FOUND",
            "message": "User 'user-123' が見つかりません",
            "trace_id": "trace-123"
        }"#;

        let details: GrpcErrorDetails = serde_json::from_str(json).unwrap();
        assert_eq!(details.error_code, "USER_NOT_FOUND");
        assert_eq!(details.trace_id, Some("trace-123".to_string()));
    }
}
