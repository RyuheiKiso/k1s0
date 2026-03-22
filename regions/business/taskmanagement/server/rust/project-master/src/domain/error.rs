// プロジェクトマスタドメインエラー定義。
// 各ユースケース・インフラ層で発生するドメイン固有エラーを表現する。
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectMasterError {
    /// プロジェクトタイプが見つからない
    #[error("project type not found: {0}")]
    ProjectTypeNotFound(String),

    /// ステータス定義が見つからない
    #[error("status definition not found: {0}")]
    StatusDefinitionNotFound(String),

    /// テナント拡張が見つからない
    #[error("tenant extension not found: tenant={0}, status={1}")]
    TenantExtensionNotFound(String, String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// コードの重複
    #[error("duplicate code: {0}")]
    DuplicateCode(String),

    /// バリデーションスキーマが不正
    #[error("invalid validation schema: {0}")]
    InvalidValidationSchema(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}
