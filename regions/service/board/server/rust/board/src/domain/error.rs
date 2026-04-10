// ボードドメインエラー型。
// Repository トレイトのインフラエラーをドメイン層に持ち込まないため、
// Infrastructure バリアントで anyhow::Error を包む。
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BoardError {
    #[error("WIP limit exceeded: column '{column_id}' has {current}/{limit} tasks")]
    WipLimitExceeded { column_id: String, current: i32, limit: i32 },
    #[error("version conflict: expected {expected}, got {actual}")]
    VersionConflict { expected: i32, actual: i32 },
    #[error("board column not found: {0}")]
    NotFound(String),
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    /// インフラ層（DB・ネットワーク等）のエラーをドメイン型に包むバリアント
    #[error("infrastructure error: {0}")]
    Infrastructure(#[from] anyhow::Error),
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // WipLimitExceeded エラーのメッセージに column_id / current / limit が含まれることを確認する
    #[test]
    fn test_wip_limit_exceeded_display() {
        let err = BoardError::WipLimitExceeded {
            column_id: "col-abc".to_string(),
            current: 5,
            limit: 5,
        };
        let msg = err.to_string();
        assert!(msg.contains("col-abc"), "column_id missing: {msg}");
        assert!(msg.contains("5"), "counts missing: {msg}");
    }

    // VersionConflict エラーのメッセージに expected / actual が含まれることを確認する
    #[test]
    fn test_version_conflict_display() {
        let err = BoardError::VersionConflict { expected: 3, actual: 5 };
        let msg = err.to_string();
        assert!(msg.contains("3"), "expected version missing: {msg}");
        assert!(msg.contains("5"), "actual version missing: {msg}");
    }

    // NotFound エラーのメッセージにリソース識別子が含まれることを確認する
    #[test]
    fn test_not_found_display() {
        let err = BoardError::NotFound("board_column:xyz-123".to_string());
        assert!(err.to_string().contains("xyz-123"));
    }

    // ValidationFailed エラーのメッセージに検証メッセージが含まれることを確認する
    #[test]
    fn test_validation_failed_display() {
        let err = BoardError::ValidationFailed("wip_limit must be positive".to_string());
        assert!(err.to_string().contains("wip_limit must be positive"));
    }

    // Infrastructure エラーが anyhow::Error から変換できることを確認する
    #[test]
    fn test_infrastructure_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("database connection timeout");
        let board_err: BoardError = anyhow_err.into();
        assert!(board_err.to_string().contains("database connection timeout"));
    }
}
