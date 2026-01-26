//! フィールドエラー
//!
//! 個別フィールドのバリデーションエラーを表現する。

use crate::ErrorCode;

/// フィールドエラーの種類
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldErrorKind {
    /// 必須フィールドが不足
    Required,
    /// フォーマット不正
    InvalidFormat,
    /// 最小値違反
    MinValue {
        /// 最小値
        min: String,
    },
    /// 最大値違反
    MaxValue {
        /// 最大値
        max: String,
    },
    /// 範囲外
    OutOfRange {
        /// 最小値
        min: Option<String>,
        /// 最大値
        max: Option<String>,
    },
    /// 最小文字数違反
    MinLength {
        /// 最小文字数
        min: usize,
    },
    /// 最大文字数違反
    MaxLength {
        /// 最大文字数
        max: usize,
    },
    /// パターン不一致
    Pattern {
        /// 期待されるパターン（正規表現）
        pattern: String,
    },
    /// 重複
    Duplicate,
    /// 参照先が存在しない
    NotFound,
    /// 列挙値以外
    InvalidEnum {
        /// 許可される値のリスト
        allowed: Vec<String>,
    },
    /// カスタムエラー
    Custom {
        /// エラーコード
        code: String,
    },
}

impl FieldErrorKind {
    /// エラーコードに変換
    pub fn to_error_code(&self) -> ErrorCode {
        match self {
            Self::Required => ErrorCode::RequiredFieldMissing,
            Self::InvalidFormat => ErrorCode::InvalidFormat,
            Self::MinValue { .. } | Self::MaxValue { .. } | Self::OutOfRange { .. } => {
                ErrorCode::OutOfRange
            }
            Self::MinLength { .. } | Self::MaxLength { .. } => ErrorCode::LengthViolation,
            Self::Pattern { .. } | Self::InvalidEnum { .. } => ErrorCode::PatternMismatch,
            Self::Duplicate => ErrorCode::DuplicateValue,
            Self::NotFound => ErrorCode::ReferenceNotFound,
            Self::Custom { .. } => ErrorCode::Custom,
        }
    }
}

/// フィールドエラー
///
/// 個別フィールドのバリデーションエラーを表現する。
#[derive(Debug, Clone)]
pub struct FieldError {
    /// フィールド名（ネストの場合は `.` 区切り。例: `address.city`）
    field: String,
    /// エラーの種類
    kind: FieldErrorKind,
    /// 人間向けのエラーメッセージ
    message: String,
}

impl FieldError {
    /// 新しいフィールドエラーを作成
    pub fn new(field: impl Into<String>, kind: FieldErrorKind, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            kind,
            message: message.into(),
        }
    }

    /// 必須エラーを作成
    pub fn required(field: impl Into<String>) -> Self {
        let field = field.into();
        Self::new(
            &field,
            FieldErrorKind::Required,
            format!("'{}' は必須です", field),
        )
    }

    /// フォーマット不正エラーを作成
    pub fn invalid_format(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(field, FieldErrorKind::InvalidFormat, message)
    }

    /// 最小値違反エラーを作成
    pub fn min_value<T: std::fmt::Display>(field: impl Into<String>, min: T) -> Self {
        let field = field.into();
        let min_str = min.to_string();
        Self::new(
            &field,
            FieldErrorKind::MinValue {
                min: min_str.clone(),
            },
            format!("'{}' は {} 以上である必要があります", field, min_str),
        )
    }

    /// 最大値違反エラーを作成
    pub fn max_value<T: std::fmt::Display>(field: impl Into<String>, max: T) -> Self {
        let field = field.into();
        let max_str = max.to_string();
        Self::new(
            &field,
            FieldErrorKind::MaxValue {
                max: max_str.clone(),
            },
            format!("'{}' は {} 以下である必要があります", field, max_str),
        )
    }

    /// 範囲外エラーを作成
    pub fn out_of_range<T: std::fmt::Display>(
        field: impl Into<String>,
        min: Option<T>,
        max: Option<T>,
    ) -> Self {
        let field = field.into();
        let min_str = min.map(|v| v.to_string());
        let max_str = max.map(|v| v.to_string());

        let message = match (&min_str, &max_str) {
            (Some(min), Some(max)) => {
                format!("'{}' は {} から {} の範囲である必要があります", field, min, max)
            }
            (Some(min), None) => format!("'{}' は {} 以上である必要があります", field, min),
            (None, Some(max)) => format!("'{}' は {} 以下である必要があります", field, max),
            (None, None) => format!("'{}' は範囲外です", field),
        };

        Self::new(
            &field,
            FieldErrorKind::OutOfRange {
                min: min_str,
                max: max_str,
            },
            message,
        )
    }

    /// 最小文字数違反エラーを作成
    pub fn min_length(field: impl Into<String>, min: usize) -> Self {
        let field = field.into();
        Self::new(
            &field,
            FieldErrorKind::MinLength { min },
            format!("'{}' は {} 文字以上である必要があります", field, min),
        )
    }

    /// 最大文字数違反エラーを作成
    pub fn max_length(field: impl Into<String>, max: usize) -> Self {
        let field = field.into();
        Self::new(
            &field,
            FieldErrorKind::MaxLength { max },
            format!("'{}' は {} 文字以下である必要があります", field, max),
        )
    }

    /// パターン不一致エラーを作成
    pub fn pattern(field: impl Into<String>, pattern: impl Into<String>) -> Self {
        let field = field.into();
        let pattern = pattern.into();
        Self::new(
            &field,
            FieldErrorKind::Pattern {
                pattern: pattern.clone(),
            },
            format!("'{}' は指定されたパターンに一致しません", field),
        )
    }

    /// 重複エラーを作成
    pub fn duplicate(field: impl Into<String>) -> Self {
        let field = field.into();
        Self::new(
            &field,
            FieldErrorKind::Duplicate,
            format!("'{}' は既に使用されています", field),
        )
    }

    /// 参照先が存在しないエラーを作成
    pub fn not_found(field: impl Into<String>) -> Self {
        let field = field.into();
        Self::new(
            &field,
            FieldErrorKind::NotFound,
            format!("'{}' で指定されたリソースが見つかりません", field),
        )
    }

    /// 列挙値以外エラーを作成
    pub fn invalid_enum(field: impl Into<String>, allowed: Vec<String>) -> Self {
        let field = field.into();
        let allowed_str = allowed.join(", ");
        Self::new(
            &field,
            FieldErrorKind::InvalidEnum {
                allowed: allowed.clone(),
            },
            format!(
                "'{}' は次のいずれかである必要があります: {}",
                field, allowed_str
            ),
        )
    }

    /// カスタムエラーを作成
    pub fn custom(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            field,
            FieldErrorKind::Custom { code: code.into() },
            message,
        )
    }

    /// フィールド名を取得
    pub fn field(&self) -> &str {
        &self.field
    }

    /// エラーの種類を取得
    pub fn kind(&self) -> &FieldErrorKind {
        &self.kind
    }

    /// エラーメッセージを取得
    pub fn message(&self) -> &str {
        &self.message
    }

    /// エラーコードを取得
    pub fn error_code(&self) -> ErrorCode {
        self.kind.to_error_code()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required() {
        let error = FieldError::required("name");
        assert_eq!(error.field(), "name");
        assert_eq!(error.kind(), &FieldErrorKind::Required);
        assert!(error.message().contains("必須"));
        assert_eq!(error.error_code(), ErrorCode::RequiredFieldMissing);
    }

    #[test]
    fn test_invalid_format() {
        let error = FieldError::invalid_format("email", "有効なメールアドレスを入力してください");
        assert_eq!(error.field(), "email");
        assert_eq!(error.kind(), &FieldErrorKind::InvalidFormat);
        assert_eq!(error.error_code(), ErrorCode::InvalidFormat);
    }

    #[test]
    fn test_min_value() {
        let error = FieldError::min_value("age", 18);
        assert_eq!(error.field(), "age");
        assert!(matches!(error.kind(), FieldErrorKind::MinValue { min } if min == "18"));
        assert!(error.message().contains("18"));
        assert_eq!(error.error_code(), ErrorCode::OutOfRange);
    }

    #[test]
    fn test_max_value() {
        let error = FieldError::max_value("quantity", 100);
        assert_eq!(error.field(), "quantity");
        assert!(matches!(error.kind(), FieldErrorKind::MaxValue { max } if max == "100"));
        assert_eq!(error.error_code(), ErrorCode::OutOfRange);
    }

    #[test]
    fn test_out_of_range() {
        let error = FieldError::out_of_range("score", Some(0), Some(100));
        assert_eq!(error.field(), "score");
        assert!(error.message().contains("0"));
        assert!(error.message().contains("100"));
    }

    #[test]
    fn test_min_length() {
        let error = FieldError::min_length("password", 8);
        assert_eq!(error.field(), "password");
        assert!(matches!(error.kind(), FieldErrorKind::MinLength { min } if *min == 8));
        assert_eq!(error.error_code(), ErrorCode::LengthViolation);
    }

    #[test]
    fn test_max_length() {
        let error = FieldError::max_length("bio", 500);
        assert_eq!(error.field(), "bio");
        assert!(matches!(error.kind(), FieldErrorKind::MaxLength { max } if *max == 500));
    }

    #[test]
    fn test_duplicate() {
        let error = FieldError::duplicate("username");
        assert_eq!(error.field(), "username");
        assert_eq!(error.kind(), &FieldErrorKind::Duplicate);
        assert_eq!(error.error_code(), ErrorCode::DuplicateValue);
    }

    #[test]
    fn test_not_found() {
        let error = FieldError::not_found("user_id");
        assert_eq!(error.field(), "user_id");
        assert_eq!(error.kind(), &FieldErrorKind::NotFound);
        assert_eq!(error.error_code(), ErrorCode::ReferenceNotFound);
    }

    #[test]
    fn test_invalid_enum() {
        let error = FieldError::invalid_enum(
            "status",
            vec!["active".to_string(), "inactive".to_string()],
        );
        assert_eq!(error.field(), "status");
        assert!(error.message().contains("active"));
        assert!(error.message().contains("inactive"));
        assert_eq!(error.error_code(), ErrorCode::PatternMismatch);
    }

    #[test]
    fn test_custom() {
        let error = FieldError::custom("field", "CUSTOM_CODE", "カスタムエラーメッセージ");
        assert_eq!(error.field(), "field");
        assert!(matches!(error.kind(), FieldErrorKind::Custom { code } if code == "CUSTOM_CODE"));
        assert_eq!(error.error_code(), ErrorCode::Custom);
    }

    #[test]
    fn test_nested_field() {
        let error = FieldError::required("address.city");
        assert_eq!(error.field(), "address.city");
    }
}
