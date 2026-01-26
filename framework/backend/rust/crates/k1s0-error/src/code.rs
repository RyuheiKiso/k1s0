//! エラーコード
//!
//! 運用で「見れば分かる」を実現するための安定した識別子。

use std::fmt;

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
}
