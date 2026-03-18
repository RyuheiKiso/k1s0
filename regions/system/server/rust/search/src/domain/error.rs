//! Search サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Search ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    /// インデックスが見つからない
    #[error("index '{0}' not found")]
    NotFound(String),

    /// クエリの構文が無効
    #[error("invalid query: {0}")]
    InvalidQuery(String),

    /// インデックス作成が失敗
    #[error("indexing failed: {0}")]
    IndexingFailed(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// SearchError から ServiceError への変換実装
impl From<SearchError> for ServiceError {
    fn from(err: SearchError) -> Self {
        match err {
            SearchError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_SEARCH_NOT_FOUND"),
                message: msg,
            },
            SearchError::InvalidQuery(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_SEARCH_INVALID_QUERY"),
                message: msg,
                details: vec![],
            },
            SearchError::IndexingFailed(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_SEARCH_INDEXING_FAILED"),
                message: msg,
            },
            SearchError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_SEARCH_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            SearchError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_SEARCH_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
