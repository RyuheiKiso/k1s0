//! k1s0-validation
//!
//! API 境界での入力バリデーションを統一するライブラリ。
//!
//! # 設計方針
//!
//! - フィールド単位のバリデーションエラーを表現
//! - REST（problem+json）と gRPC（INVALID_ARGUMENT + error_code）の両方に対応
//! - OpenAPI/proto と整合するバリデーション方針
//!
//! # 使用例
//!
//! ```
//! use k1s0_validation::{ValidationErrors, FieldError, ErrorCode};
//!
//! let mut errors = ValidationErrors::new();
//!
//! // フィールドエラーを追加
//! errors.add_field_error(FieldError::required("name"));
//! errors.add_field_error(FieldError::invalid_format("email", "有効なメールアドレスを入力してください"));
//!
//! if !errors.is_empty() {
//!     // REST の場合: problem+json に変換
//!     let problem = errors.to_problem_details("/users", "ユーザー作成");
//!
//!     // gRPC の場合: error_code と詳細情報を取得
//!     let error_code = errors.error_code();
//!     let details = errors.to_grpc_details();
//! }
//! ```

mod error;
mod field;
mod grpc;
mod problem;

pub use error::{ErrorCode, ValidationError};
pub use field::{FieldError, FieldErrorKind};
pub use grpc::GrpcErrorDetails;
pub use problem::ProblemDetails;

use std::collections::HashMap;

/// バリデーションエラーのコレクション
///
/// 複数のフィールドエラーを集約し、REST/gRPC 両方の形式で出力できる。
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    /// フィールド名をキーとしたエラーのマップ
    errors: HashMap<String, Vec<FieldError>>,
}

impl ValidationErrors {
    /// 新しい空のエラーコレクションを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// フィールドエラーを追加
    pub fn add_field_error(&mut self, error: FieldError) {
        self.errors
            .entry(error.field().to_string())
            .or_default()
            .push(error);
    }

    /// 特定のフィールドにエラーがあるか確認
    pub fn has_field_error(&self, field: &str) -> bool {
        self.errors.contains_key(field)
    }

    /// エラーが空かどうか確認
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// エラーの総数を取得
    pub fn len(&self) -> usize {
        self.errors.values().map(|v| v.len()).sum()
    }

    /// すべてのフィールドエラーを取得
    pub fn field_errors(&self) -> &HashMap<String, Vec<FieldError>> {
        &self.errors
    }

    /// エラーコードを取得（バリデーションエラーは常に VALIDATION_ERROR）
    pub fn error_code(&self) -> ErrorCode {
        ErrorCode::ValidationError
    }

    /// REST 用の problem+json 形式に変換
    ///
    /// # Arguments
    ///
    /// * `instance` - 問題が発生したリソースのパス（例: "/users"）
    /// * `title` - 問題のタイトル（例: "ユーザー作成"）
    pub fn to_problem_details(&self, instance: &str, title: &str) -> ProblemDetails {
        ProblemDetails::from_validation_errors(self, instance, title)
    }

    /// gRPC 用のエラー詳細に変換
    pub fn to_grpc_details(&self) -> GrpcErrorDetails {
        GrpcErrorDetails::from_validation_errors(self)
    }

    /// ValidationError に変換
    pub fn into_error(self) -> ValidationError {
        ValidationError::new(self)
    }
}

impl From<FieldError> for ValidationErrors {
    fn from(error: FieldError) -> Self {
        let mut errors = Self::new();
        errors.add_field_error(error);
        errors
    }
}

impl From<Vec<FieldError>> for ValidationErrors {
    fn from(field_errors: Vec<FieldError>) -> Self {
        let mut errors = Self::new();
        for error in field_errors {
            errors.add_field_error(error);
        }
        errors
    }
}

/// バリデーション結果の型エイリアス
pub type ValidationResult<T> = Result<T, ValidationError>;

/// バリデーションを実行するトレイト
///
/// リクエスト DTO に実装することで、統一されたバリデーションを提供できる。
pub trait Validate {
    /// バリデーションを実行
    ///
    /// エラーがある場合は `ValidationErrors` を返す。
    fn validate(&self) -> Result<(), ValidationErrors>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_errors_new() {
        let errors = ValidationErrors::new();
        assert!(errors.is_empty());
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_add_field_error() {
        let mut errors = ValidationErrors::new();
        errors.add_field_error(FieldError::required("name"));
        errors.add_field_error(FieldError::required("email"));

        assert!(!errors.is_empty());
        assert_eq!(errors.len(), 2);
        assert!(errors.has_field_error("name"));
        assert!(errors.has_field_error("email"));
        assert!(!errors.has_field_error("age"));
    }

    #[test]
    fn test_multiple_errors_per_field() {
        let mut errors = ValidationErrors::new();
        errors.add_field_error(FieldError::required("password"));
        errors.add_field_error(FieldError::min_length("password", 8));

        assert_eq!(errors.len(), 2);
        assert_eq!(errors.field_errors().get("password").unwrap().len(), 2);
    }

    #[test]
    fn test_error_code() {
        let errors = ValidationErrors::new();
        assert_eq!(errors.error_code(), ErrorCode::ValidationError);
    }

    #[test]
    fn test_from_field_error() {
        let errors: ValidationErrors = FieldError::required("name").into();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_from_vec_field_error() {
        let errors: ValidationErrors = vec![
            FieldError::required("name"),
            FieldError::required("email"),
        ]
        .into();
        assert_eq!(errors.len(), 2);
    }
}
