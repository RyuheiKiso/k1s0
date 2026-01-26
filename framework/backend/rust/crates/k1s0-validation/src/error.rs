//! エラー型
//!
//! バリデーションエラーとエラーコードを定義する。

use std::fmt;

use crate::ValidationErrors;

/// エラーコード
///
/// 運用で「見れば分かる」を実現するための安定した識別子。
/// gRPC/REST の両方で同じコードを使用する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// バリデーションエラー
    ValidationError,
    /// 必須フィールドが不足
    RequiredFieldMissing,
    /// フォーマット不正
    InvalidFormat,
    /// 範囲外の値
    OutOfRange,
    /// 長さ制約違反
    LengthViolation,
    /// パターン不一致
    PatternMismatch,
    /// 重複エラー
    DuplicateValue,
    /// 参照先が存在しない
    ReferenceNotFound,
    /// カスタムエラー
    Custom,
}

impl ErrorCode {
    /// エラーコードの文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ValidationError => "VALIDATION_ERROR",
            Self::RequiredFieldMissing => "REQUIRED_FIELD_MISSING",
            Self::InvalidFormat => "INVALID_FORMAT",
            Self::OutOfRange => "OUT_OF_RANGE",
            Self::LengthViolation => "LENGTH_VIOLATION",
            Self::PatternMismatch => "PATTERN_MISMATCH",
            Self::DuplicateValue => "DUPLICATE_VALUE",
            Self::ReferenceNotFound => "REFERENCE_NOT_FOUND",
            Self::Custom => "CUSTOM_VALIDATION_ERROR",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// バリデーションエラー
///
/// `ValidationErrors` をラップし、`std::error::Error` を実装する。
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// エラーの詳細
    errors: ValidationErrors,
}

impl ValidationError {
    /// 新しいバリデーションエラーを作成
    pub fn new(errors: ValidationErrors) -> Self {
        Self { errors }
    }

    /// エラーの詳細を取得
    pub fn errors(&self) -> &ValidationErrors {
        &self.errors
    }

    /// エラーの詳細を消費して取得
    pub fn into_errors(self) -> ValidationErrors {
        self.errors
    }

    /// エラーコードを取得
    pub fn error_code(&self) -> ErrorCode {
        self.errors.error_code()
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.errors.len();
        if count == 1 {
            write!(f, "バリデーションエラーが 1 件あります")
        } else {
            write!(f, "バリデーションエラーが {} 件あります", count)
        }
    }
}

impl std::error::Error for ValidationError {}

impl From<ValidationErrors> for ValidationError {
    fn from(errors: ValidationErrors) -> Self {
        Self::new(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FieldError;

    #[test]
    fn test_error_code_as_str() {
        assert_eq!(ErrorCode::ValidationError.as_str(), "VALIDATION_ERROR");
        assert_eq!(
            ErrorCode::RequiredFieldMissing.as_str(),
            "REQUIRED_FIELD_MISSING"
        );
        assert_eq!(ErrorCode::InvalidFormat.as_str(), "INVALID_FORMAT");
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(format!("{}", ErrorCode::ValidationError), "VALIDATION_ERROR");
    }

    #[test]
    fn test_validation_error_display() {
        let mut errors = ValidationErrors::new();
        errors.add_field_error(FieldError::required("name"));
        let error = ValidationError::new(errors);
        assert_eq!(
            format!("{}", error),
            "バリデーションエラーが 1 件あります"
        );

        let mut errors2 = ValidationErrors::new();
        errors2.add_field_error(FieldError::required("name"));
        errors2.add_field_error(FieldError::required("email"));
        let error2 = ValidationError::new(errors2);
        assert_eq!(
            format!("{}", error2),
            "バリデーションエラーが 2 件あります"
        );
    }

    #[test]
    fn test_validation_error_into_errors() {
        let mut errors = ValidationErrors::new();
        errors.add_field_error(FieldError::required("name"));
        let error = ValidationError::new(errors);
        let recovered = error.into_errors();
        assert_eq!(recovered.len(), 1);
    }
}
