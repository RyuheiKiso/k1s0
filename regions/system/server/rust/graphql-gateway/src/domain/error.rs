//! GraphQL Gateway サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で GraphQL エラーコードを決定する。
//!
//! M-15 監査対応: gRPC Status コードから型安全な GraphQL エラーカテゴリに変換する仕組みを提供する。
//! `graphql_handler.rs` の `classify_domain_error` は文字列マッチング依存だったが、
//! `GrpcErrorCategory` を通じて `tonic::Status` から直接分類できるようにした。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// `GraphqlGateway` ドメイン固有のエラー型。
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

/// gRPC Status コードから GraphQL エラーコードへの型安全な分類。
/// `tonic::Status` を直接受け取りエラーカテゴリを返す。
/// M-15 監査対応: 文字列マッチングに依存しない型安全な分類の実装基盤。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrpcErrorCategory {
    /// `認証エラー（tonic::Code::Unauthenticated`）
    Unauthenticated,
    /// `権限エラー（tonic::Code::PermissionDenied`）
    Forbidden,
    /// `バリデーションエラー（tonic::Code::InvalidArgument` / `FailedPrecondition` / `OutOfRange`）
    ValidationError,
    /// バックエンドエラー（その他）
    BackendError,
}

impl GrpcErrorCategory {
    /// `tonic::Status` から `GrpcErrorCategory` に変換する。
    /// usecase 層または adapter 層で `tonic::Status` を直接受け取った場合に使用する。
    #[must_use] 
    pub fn from_tonic_code(code: tonic::Code) -> Self {
        match code {
            tonic::Code::Unauthenticated => GrpcErrorCategory::Unauthenticated,
            tonic::Code::PermissionDenied => GrpcErrorCategory::Forbidden,
            tonic::Code::InvalidArgument
            | tonic::Code::FailedPrecondition
            | tonic::Code::OutOfRange => GrpcErrorCategory::ValidationError,
            _ => GrpcErrorCategory::BackendError,
        }
    }

    /// GraphQL エラーコード文字列を返す。
    /// `graphql_handler.rs` の `gql_error()` に渡す &'static str として使用する。
    #[must_use] 
    pub fn as_graphql_code(&self) -> &'static str {
        match self {
            GrpcErrorCategory::Unauthenticated => "UNAUTHENTICATED",
            GrpcErrorCategory::Forbidden => "FORBIDDEN",
            GrpcErrorCategory::ValidationError => "VALIDATION_ERROR",
            GrpcErrorCategory::BackendError => "BACKEND_ERROR",
        }
    }
}

/// `GraphqlGatewayError` から `ServiceError` への変換実装
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
