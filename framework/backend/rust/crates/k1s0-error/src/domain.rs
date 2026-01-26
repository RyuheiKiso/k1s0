//! Domain 層エラー
//!
//! transport 非依存のエラー型。HTTP/gRPC を意識しない純粋なドメインエラー。

use std::fmt;

/// エラーの種類
///
/// ドメインロジックで発生するエラーの分類。
/// transport（HTTP/gRPC）の詳細は含まない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// 入力不備（バリデーションエラー）
    InvalidInput,
    /// リソースが見つからない
    NotFound,
    /// 競合（重複、楽観ロック失敗等）
    Conflict,
    /// 認証エラー
    Unauthorized,
    /// 認可エラー（権限不足）
    Forbidden,
    /// 依存先の障害（外部サービス、DB等）
    DependencyFailure,
    /// 一時的な障害（リトライ可能）
    Transient,
    /// 不変条件違反（ビジネスルール違反）
    InvariantViolation,
    /// 内部エラー（予期しないエラー）
    Internal,
}

impl ErrorKind {
    /// エラーの種類を文字列で取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidInput => "INVALID_INPUT",
            Self::NotFound => "NOT_FOUND",
            Self::Conflict => "CONFLICT",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::DependencyFailure => "DEPENDENCY_FAILURE",
            Self::Transient => "TRANSIENT",
            Self::InvariantViolation => "INVARIANT_VIOLATION",
            Self::Internal => "INTERNAL",
        }
    }

    /// リトライ可能かどうか
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Transient | Self::DependencyFailure)
    }

    /// クライアント起因のエラーかどうか
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::InvalidInput
                | Self::NotFound
                | Self::Conflict
                | Self::Unauthorized
                | Self::Forbidden
                | Self::InvariantViolation
        )
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Domain 層エラー
///
/// transport 非依存のエラー型。HTTP status code や gRPC status は含まない。
#[derive(Debug, Clone)]
pub struct DomainError {
    /// エラーの種類
    kind: ErrorKind,
    /// エラーメッセージ
    message: String,
    /// 対象リソースの種類（オプション）
    resource_type: Option<String>,
    /// 対象リソースの ID（オプション）
    resource_id: Option<String>,
    /// 原因となったエラー（オプション）
    source_message: Option<String>,
}

impl DomainError {
    /// 新しいドメインエラーを作成
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            resource_type: None,
            resource_id: None,
            source_message: None,
        }
    }

    /// 入力不備エラーを作成
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidInput, message)
    }

    /// リソースが見つからないエラーを作成
    pub fn not_found(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        let resource_type = resource_type.into();
        let resource_id = resource_id.into();
        let message = format!("{} '{}' が見つかりません", resource_type, resource_id);
        Self {
            kind: ErrorKind::NotFound,
            message,
            resource_type: Some(resource_type),
            resource_id: Some(resource_id),
            source_message: None,
        }
    }

    /// 競合エラーを作成
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Conflict, message)
    }

    /// 重複エラーを作成
    pub fn duplicate(resource_type: impl Into<String>, field: impl Into<String>) -> Self {
        let resource_type = resource_type.into();
        let field = field.into();
        let message = format!("{} の {} は既に使用されています", resource_type, field);
        Self {
            kind: ErrorKind::Conflict,
            message,
            resource_type: Some(resource_type),
            resource_id: None,
            source_message: None,
        }
    }

    /// 認証エラーを作成
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Unauthorized, message)
    }

    /// 認可エラーを作成
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Forbidden, message)
    }

    /// 依存障害エラーを作成
    pub fn dependency_failure(dependency: impl Into<String>, message: impl Into<String>) -> Self {
        let dependency = dependency.into();
        let message = message.into();
        Self {
            kind: ErrorKind::DependencyFailure,
            message: format!("{}: {}", dependency, message),
            resource_type: Some(dependency),
            resource_id: None,
            source_message: Some(message),
        }
    }

    /// 一時障害エラーを作成
    pub fn transient(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Transient, message)
    }

    /// 不変条件違反エラーを作成
    pub fn invariant_violation(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvariantViolation, message)
    }

    /// 内部エラーを作成
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    /// 原因となったエラーを設定
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source_message = Some(source.into());
        self
    }

    /// リソース情報を設定
    pub fn with_resource(
        mut self,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        self.resource_type = Some(resource_type.into());
        self.resource_id = Some(resource_id.into());
        self
    }

    /// エラーの種類を取得
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// エラーメッセージを取得
    pub fn message(&self) -> &str {
        &self.message
    }

    /// リソースの種類を取得
    pub fn resource_type(&self) -> Option<&str> {
        self.resource_type.as_deref()
    }

    /// リソースの ID を取得
    pub fn resource_id(&self) -> Option<&str> {
        self.resource_id.as_deref()
    }

    /// 原因となったエラーを取得
    pub fn source_message(&self) -> Option<&str> {
        self.source_message.as_deref()
    }

    /// リトライ可能かどうか
    pub fn is_retryable(&self) -> bool {
        self.kind.is_retryable()
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DomainError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_kind_as_str() {
        assert_eq!(ErrorKind::InvalidInput.as_str(), "INVALID_INPUT");
        assert_eq!(ErrorKind::NotFound.as_str(), "NOT_FOUND");
        assert_eq!(ErrorKind::Conflict.as_str(), "CONFLICT");
    }

    #[test]
    fn test_error_kind_is_retryable() {
        assert!(ErrorKind::Transient.is_retryable());
        assert!(ErrorKind::DependencyFailure.is_retryable());
        assert!(!ErrorKind::InvalidInput.is_retryable());
        assert!(!ErrorKind::NotFound.is_retryable());
    }

    #[test]
    fn test_error_kind_is_client_error() {
        assert!(ErrorKind::InvalidInput.is_client_error());
        assert!(ErrorKind::NotFound.is_client_error());
        assert!(ErrorKind::Forbidden.is_client_error());
        assert!(!ErrorKind::Internal.is_client_error());
        assert!(!ErrorKind::Transient.is_client_error());
    }

    #[test]
    fn test_invalid_input() {
        let err = DomainError::invalid_input("名前は必須です");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert_eq!(err.message(), "名前は必須です");
    }

    #[test]
    fn test_not_found() {
        let err = DomainError::not_found("User", "user-123");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(err.message().contains("User"));
        assert!(err.message().contains("user-123"));
        assert_eq!(err.resource_type(), Some("User"));
        assert_eq!(err.resource_id(), Some("user-123"));
    }

    #[test]
    fn test_duplicate() {
        let err = DomainError::duplicate("User", "email");
        assert_eq!(err.kind(), ErrorKind::Conflict);
        assert!(err.message().contains("email"));
    }

    #[test]
    fn test_dependency_failure() {
        let err = DomainError::dependency_failure("PostgreSQL", "接続タイムアウト");
        assert_eq!(err.kind(), ErrorKind::DependencyFailure);
        assert!(err.is_retryable());
        assert!(err.message().contains("PostgreSQL"));
    }

    #[test]
    fn test_with_source() {
        let err = DomainError::internal("処理に失敗しました")
            .with_source("原因: メモリ不足");
        assert_eq!(err.source_message(), Some("原因: メモリ不足"));
    }

    #[test]
    fn test_with_resource() {
        let err = DomainError::new(ErrorKind::Internal, "エラー")
            .with_resource("Order", "order-456");
        assert_eq!(err.resource_type(), Some("Order"));
        assert_eq!(err.resource_id(), Some("order-456"));
    }

    #[test]
    fn test_display() {
        let err = DomainError::not_found("User", "user-123");
        let display = format!("{}", err);
        assert!(display.contains("User"));
        assert!(display.contains("user-123"));
    }
}
