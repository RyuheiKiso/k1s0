//! GraphQL Gateway サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// GraphqlGateway ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum GraphqlGatewayError {
    /// スキーマが見つからない
    #[error("schema '{0}' not found")]
    NotFound(String),

    /// クエリの解析に失敗
    #[error("query parse failed: {0}")]
    QueryParseFailed(String),

    /// 上流サービスへのリクエストが失敗
    #[error("upstream service error: {0}")]
    UpstreamError(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// GraphqlGatewayError から ServiceError への変換実装
impl From<GraphqlGatewayError> for ServiceError {
    fn from(err: GraphqlGatewayError) -> Self {
        match err {
            GraphqlGatewayError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_GQLGW_NOT_FOUND"),
                message: msg,
            },
            GraphqlGatewayError::QueryParseFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_GQLGW_QUERY_PARSE_FAILED"),
                message: msg,
                details: vec![],
            },
            GraphqlGatewayError::UpstreamError(msg) => ServiceError::ServiceUnavailable {
                code: ErrorCode::new("SYS_GQLGW_UPSTREAM_ERROR"),
                message: msg,
            },
            GraphqlGatewayError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_GQLGW_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            GraphqlGatewayError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_GQLGW_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
