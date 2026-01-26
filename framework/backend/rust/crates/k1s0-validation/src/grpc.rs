//! gRPC エラー詳細
//!
//! gRPC API 用のバリデーションエラー表現。
//! `INVALID_ARGUMENT` ステータスコードと組み合わせて使用する。

use serde::{Deserialize, Serialize};

use crate::{ErrorCode, FieldError, ValidationErrors};

/// gRPC エラー詳細
///
/// gRPC の `Status` に付与するエラー詳細を表現する。
/// `tonic::Status::with_details()` などで metadata に含めて送信できる。
///
/// # 使用例
///
/// ```ignore
/// use tonic::Status;
/// use k1s0_validation::{ValidationErrors, FieldError, GrpcErrorDetails};
///
/// let mut errors = ValidationErrors::new();
/// errors.add_field_error(FieldError::required("name"));
///
/// let details = errors.to_grpc_details();
/// let status = Status::invalid_argument(details.message())
///     .with_details(details.to_json().unwrap().into_bytes().into());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcErrorDetails {
    /// エラーコード
    pub error_code: String,

    /// エラーメッセージ
    pub message: String,

    /// フィールドごとのエラー詳細
    pub field_violations: Vec<FieldViolation>,

    /// トレースID（設定されている場合）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

/// フィールド違反
///
/// Google の `BadRequest.FieldViolation` に相当。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldViolation {
    /// フィールド名（ネストの場合は `.` 区切り）
    pub field: String,

    /// エラーコード
    pub code: String,

    /// エラーメッセージ
    pub description: String,
}

/// gRPC ステータスコード（参考用）
///
/// gRPC の標準ステータスコードの一部。
/// バリデーションエラーは `INVALID_ARGUMENT` を使用する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrpcStatusCode {
    /// 成功
    Ok = 0,
    /// クライアントエラー（引数不正）
    InvalidArgument = 3,
    /// リソースが見つからない
    NotFound = 5,
    /// 既に存在する
    AlreadyExists = 6,
    /// 権限不足
    PermissionDenied = 7,
    /// 認証エラー
    Unauthenticated = 16,
}

impl GrpcStatusCode {
    /// 数値に変換
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl GrpcErrorDetails {
    /// バリデーションエラーから GrpcErrorDetails を作成
    pub fn from_validation_errors(errors: &ValidationErrors) -> Self {
        let field_violations: Vec<FieldViolation> = errors
            .field_errors()
            .iter()
            .flat_map(|(_, errs)| {
                errs.iter().map(|e| FieldViolation {
                    field: e.field().to_string(),
                    code: e.error_code().to_string(),
                    description: e.message().to_string(),
                })
            })
            .collect();

        let message = if field_violations.len() == 1 {
            format!(
                "バリデーションエラー: {}",
                field_violations[0].description
            )
        } else {
            format!(
                "バリデーションエラーが {} 件あります",
                field_violations.len()
            )
        };

        Self {
            error_code: ErrorCode::ValidationError.to_string(),
            message,
            field_violations,
            trace_id: None,
        }
    }

    /// トレースIDを設定
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// エラーメッセージを取得
    pub fn message(&self) -> &str {
        &self.message
    }

    /// フィールド違反の数を取得
    pub fn violation_count(&self) -> usize {
        self.field_violations.len()
    }

    /// 推奨される gRPC ステータスコードを取得
    pub fn suggested_status_code(&self) -> GrpcStatusCode {
        GrpcStatusCode::InvalidArgument
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

impl FieldViolation {
    /// 新しいフィールド違反を作成
    pub fn new(
        field: impl Into<String>,
        code: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            description: description.into(),
        }
    }

    /// FieldError から作成
    pub fn from_field_error(error: &FieldError) -> Self {
        Self {
            field: error.field().to_string(),
            code: error.error_code().to_string(),
            description: error.message().to_string(),
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

        let details = GrpcErrorDetails::from_validation_errors(&errors);

        assert_eq!(details.error_code, "VALIDATION_ERROR");
        assert_eq!(details.violation_count(), 2);
        assert!(details.message.contains("2 件"));
    }

    #[test]
    fn test_single_error_message() {
        let mut errors = ValidationErrors::new();
        errors.add_field_error(FieldError::required("name"));

        let details = GrpcErrorDetails::from_validation_errors(&errors);

        assert!(details.message.contains("'name' は必須です"));
    }

    #[test]
    fn test_with_trace_id() {
        let errors = ValidationErrors::new();
        let details = GrpcErrorDetails::from_validation_errors(&errors).with_trace_id("trace-123");

        assert_eq!(details.trace_id, Some("trace-123".to_string()));
    }

    #[test]
    fn test_suggested_status_code() {
        let errors = ValidationErrors::new();
        let details = GrpcErrorDetails::from_validation_errors(&errors);

        assert_eq!(
            details.suggested_status_code(),
            GrpcStatusCode::InvalidArgument
        );
    }

    #[test]
    fn test_to_json() {
        let mut errors = ValidationErrors::new();
        errors.add_field_error(FieldError::required("name"));

        let details = GrpcErrorDetails::from_validation_errors(&errors);
        let json = details.to_json().unwrap();

        assert!(json.contains("VALIDATION_ERROR"));
        assert!(json.contains("field_violations"));
        assert!(json.contains("name"));
    }

    #[test]
    fn test_deserialize() {
        let json = r#"{
            "error_code": "VALIDATION_ERROR",
            "message": "バリデーションエラー",
            "field_violations": [
                {"field": "name", "code": "REQUIRED_FIELD_MISSING", "description": "必須です"}
            ]
        }"#;

        let details: GrpcErrorDetails = serde_json::from_str(json).unwrap();
        assert_eq!(details.error_code, "VALIDATION_ERROR");
        assert_eq!(details.field_violations.len(), 1);
    }

    #[test]
    fn test_grpc_status_code() {
        assert_eq!(GrpcStatusCode::Ok.as_i32(), 0);
        assert_eq!(GrpcStatusCode::InvalidArgument.as_i32(), 3);
        assert_eq!(GrpcStatusCode::NotFound.as_i32(), 5);
    }

    #[test]
    fn test_field_violation_from_field_error() {
        let error = FieldError::required("username");
        let violation = FieldViolation::from_field_error(&error);

        assert_eq!(violation.field, "username");
        assert_eq!(violation.code, "REQUIRED_FIELD_MISSING");
        assert!(violation.description.contains("必須"));
    }
}
