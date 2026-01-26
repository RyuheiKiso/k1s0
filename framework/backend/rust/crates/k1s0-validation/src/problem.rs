//! Problem Details (RFC 7807)
//!
//! REST API 用のエラー表現。`application/problem+json` 形式に準拠。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{ErrorCode, FieldError, ValidationErrors};

/// Problem Details (RFC 7807)
///
/// REST API のエラーレスポンスを表現する。
/// `application/problem+json` 形式に準拠。
///
/// # 例
///
/// ```json
/// {
///   "type": "https://k1s0.io/errors/validation-error",
///   "title": "ユーザー作成",
///   "status": 400,
///   "detail": "入力値に問題があります",
///   "instance": "/users",
///   "error_code": "VALIDATION_ERROR",
///   "errors": {
///     "name": [
///       {
///         "code": "REQUIRED_FIELD_MISSING",
///         "message": "'name' は必須です"
///       }
///     ],
///     "email": [
///       {
///         "code": "INVALID_FORMAT",
///         "message": "有効なメールアドレスを入力してください"
///       }
///     ]
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    /// エラータイプの URI
    #[serde(rename = "type")]
    pub type_uri: String,

    /// エラーのタイトル
    pub title: String,

    /// HTTP ステータスコード
    pub status: u16,

    /// エラーの詳細説明
    pub detail: String,

    /// 問題が発生したリソースのパス
    pub instance: String,

    /// エラーコード（k1s0 固有）
    pub error_code: String,

    /// フィールドごとのエラー詳細
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub errors: HashMap<String, Vec<FieldErrorDetail>>,

    /// トレースID（設定されている場合）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

/// フィールドエラーの詳細
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldErrorDetail {
    /// エラーコード
    pub code: String,

    /// エラーメッセージ
    pub message: String,
}

impl ProblemDetails {
    /// バリデーションエラーから ProblemDetails を作成
    ///
    /// # Arguments
    ///
    /// * `errors` - バリデーションエラー
    /// * `instance` - 問題が発生したリソースのパス
    /// * `title` - 問題のタイトル
    pub fn from_validation_errors(
        errors: &ValidationErrors,
        instance: &str,
        title: &str,
    ) -> Self {
        let field_errors = errors
            .field_errors()
            .iter()
            .map(|(field, errs)| {
                let details: Vec<FieldErrorDetail> = errs
                    .iter()
                    .map(|e| FieldErrorDetail {
                        code: e.error_code().to_string(),
                        message: e.message().to_string(),
                    })
                    .collect();
                (field.clone(), details)
            })
            .collect();

        Self {
            type_uri: "https://k1s0.io/errors/validation-error".to_string(),
            title: title.to_string(),
            status: 400,
            detail: "入力値に問題があります".to_string(),
            instance: instance.to_string(),
            error_code: ErrorCode::ValidationError.to_string(),
            errors: field_errors,
            trace_id: None,
        }
    }

    /// トレースIDを設定
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// JSON 文字列に変換
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 整形された JSON 文字列に変換
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl FieldErrorDetail {
    /// 新しいフィールドエラー詳細を作成
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// FieldError から作成
    pub fn from_field_error(error: &FieldError) -> Self {
        Self {
            code: error.error_code().to_string(),
            message: error.message().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_validation_errors() {
        let mut errors = ValidationErrors::new();
        errors.add_field_error(FieldError::required("name"));
        errors.add_field_error(FieldError::invalid_format(
            "email",
            "有効なメールアドレスを入力してください",
        ));

        let problem = ProblemDetails::from_validation_errors(&errors, "/users", "ユーザー作成");

        assert_eq!(problem.status, 400);
        assert_eq!(problem.error_code, "VALIDATION_ERROR");
        assert_eq!(problem.instance, "/users");
        assert_eq!(problem.title, "ユーザー作成");
        assert!(problem.errors.contains_key("name"));
        assert!(problem.errors.contains_key("email"));
    }

    #[test]
    fn test_with_trace_id() {
        let errors = ValidationErrors::new();
        let problem = ProblemDetails::from_validation_errors(&errors, "/users", "ユーザー作成")
            .with_trace_id("abc123");

        assert_eq!(problem.trace_id, Some("abc123".to_string()));
    }

    #[test]
    fn test_to_json() {
        let mut errors = ValidationErrors::new();
        errors.add_field_error(FieldError::required("name"));

        let problem = ProblemDetails::from_validation_errors(&errors, "/users", "ユーザー作成");
        let json = problem.to_json().unwrap();

        assert!(json.contains("VALIDATION_ERROR"));
        assert!(json.contains("name"));
        assert!(json.contains("REQUIRED_FIELD_MISSING"));
    }

    #[test]
    fn test_deserialize() {
        let json = r#"{
            "type": "https://k1s0.io/errors/validation-error",
            "title": "テスト",
            "status": 400,
            "detail": "入力値に問題があります",
            "instance": "/test",
            "error_code": "VALIDATION_ERROR",
            "errors": {
                "field1": [{"code": "REQUIRED_FIELD_MISSING", "message": "必須です"}]
            }
        }"#;

        let problem: ProblemDetails = serde_json::from_str(json).unwrap();
        assert_eq!(problem.status, 400);
        assert_eq!(problem.errors.len(), 1);
    }

    #[test]
    fn test_empty_errors_not_serialized() {
        let errors = ValidationErrors::new();
        let problem = ProblemDetails::from_validation_errors(&errors, "/test", "テスト");
        let json = problem.to_json().unwrap();

        // errors が空の場合はフィールド自体が出力されない
        assert!(!json.contains("\"errors\""));
    }
}
