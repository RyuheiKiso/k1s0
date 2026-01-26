//! k1s0 gRPC サーバ共通基盤
//!
//! gRPC サーバの共通基盤を提供する。
//!
//! # 主な機能
//!
//! - **サーバ初期化**: OTel/ログ/メトリクスの共通エントリ
//! - **トレースコンテキスト伝播**: W3C Trace Context 対応
//! - **error_code/status 統一**: 内外変換の固定
//! - **request_id 相関**: リクエスト追跡
//! - **テナント情報読み取り**: マルチテナント対応
//! - **デッドライン検知**: 未指定/逸脱の検知
//!
//! # 設計原則
//!
//! 1. **共通インターセプタ**: "最低限の礼儀"をテンプレで自動有効
//! 2. **error_code 必須**: エラー時は必ず error_code を付与
//! 3. **デッドライン検知**: クライアントがデッドラインを指定していない場合の検知
//! 4. **構造化ログ**: JSON 形式で必須フィールドを統一
//!
//! # 使用例
//!
//! ```rust
//! use k1s0_grpc_server::{
//!     GrpcServerConfig, InterceptorConfig, DeadlinePolicy,
//!     RequestContext, ResponseMetadata, RequestLog,
//! };
//! use k1s0_grpc_server::error::{GrpcStatusCode, LogLevel};
//!
//! // サーバ設定の構築
//! let config = GrpcServerConfig::builder()
//!     .service_name("my-service")
//!     .env("dev")
//!     .port(50051)
//!     .build()
//!     .unwrap();
//!
//! // リクエストコンテキストの作成
//! let ctx = RequestContext::new();
//!
//! // レスポンスメタデータの作成
//! let resp = ResponseMetadata::from_context(&ctx)
//!     .with_error_code("USER_NOT_FOUND");
//!
//! // リクエストログの作成
//! let log = RequestLog::new(
//!     LogLevel::Info,
//!     "request completed",
//!     "my-service",
//!     "dev",
//!     &ctx,
//! )
//! .with_grpc("UserService", "GetUser", GrpcStatusCode::Ok)
//! .with_latency(42.5);
//!
//! println!("{}", log.to_json().unwrap());
//! ```
//!
//! # デッドラインポリシー
//!
//! クライアントがデッドラインを指定していない場合の動作を設定できる:
//!
//! ```rust
//! use k1s0_grpc_server::{InterceptorConfig, DeadlinePolicy};
//!
//! // 許可（ログ/メトリクスのみ）
//! let config = InterceptorConfig {
//!     deadline_policy: DeadlinePolicy::Allow,
//!     ..Default::default()
//! };
//!
//! // 警告（ログ/メトリクス + ヘッダ通知）
//! let config = InterceptorConfig {
//!     deadline_policy: DeadlinePolicy::Warn,
//!     ..Default::default()
//! };
//!
//! // 拒否（INVALID_ARGUMENT で返す）
//! let config = InterceptorConfig {
//!     deadline_policy: DeadlinePolicy::Reject,
//!     ..Default::default()
//! };
//! ```

pub mod config;
pub mod error;
pub mod interceptors;

// Re-exports
pub use config::{
    DeadlinePolicy, GrpcServerConfig, GrpcServerConfigBuilder, InterceptorConfig, TlsConfig,
};
pub use error::{GrpcServerError, GrpcStatusCode, LogLevel};
pub use interceptors::{
    InterceptorError, InterceptorResult, MetadataKeys, RequestContext, RequestLog,
    ResponseMetadata, ServerMetricLabels,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_usage() {
        let config = GrpcServerConfig::builder()
            .service_name("my-service")
            .env("dev")
            .build()
            .unwrap();

        assert_eq!(config.service_name(), "my-service");
        assert_eq!(config.env(), "dev");
    }

    #[test]
    fn test_request_context() {
        let ctx = RequestContext::new();
        assert!(!ctx.trace_id.is_empty());
        assert!(!ctx.request_id.is_empty());
    }

    #[test]
    fn test_response_metadata() {
        let ctx = RequestContext::new();
        let resp = ResponseMetadata::from_context(&ctx);

        assert_eq!(resp.trace_id, Some(ctx.trace_id.clone()));
    }

    #[test]
    fn test_request_log() {
        let ctx = RequestContext::new();
        let log = RequestLog::new(LogLevel::Info, "test", "my-service", "dev", &ctx);

        let json = log.to_json().unwrap();
        assert!(json.contains("my-service"));
    }

    #[test]
    fn test_deadline_policy() {
        let config = InterceptorConfig {
            deadline_policy: DeadlinePolicy::Reject,
            ..Default::default()
        };

        assert_eq!(config.deadline_policy, DeadlinePolicy::Reject);
    }

    #[test]
    fn test_grpc_status_code() {
        assert_eq!(GrpcStatusCode::Ok as i32, 0);
        assert_eq!(GrpcStatusCode::NotFound as i32, 5);
        assert_eq!(GrpcStatusCode::Internal as i32, 13);
    }

    #[test]
    fn test_interceptor_error() {
        let error = InterceptorError::deadline_not_specified();
        assert_eq!(error.status_code, GrpcStatusCode::InvalidArgument);
        assert_eq!(error.error_code, Some("DEADLINE_NOT_SPECIFIED".to_string()));
    }
}
