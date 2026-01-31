//! k1s0 gRPC クライアント共通基盤
//!
//! gRPC クライアント呼び出しの共通基盤を提供する。
//!
//! # 主な機能
//!
//! - **deadline 必須**: タイムアウト下限/上限のバリデーション
//! - **retry 原則 0**: リトライは明示的な opt-in（ADR 参照必須）
//! - **トレース伝播**: W3C Trace Context (traceparent) の自動付与
//! - **メタデータ自動付与**: trace_id, request_id, tenant_id 等
//! - **サービスディスカバリ**: 論理名 → アドレス解決（K8s DNS 対応）
//!
//! # 設計原則
//!
//! 1. **deadline 必須**: 無制限呼び出しを入りにくくする
//! 2. **retry 原則 0**: 冪等でない RPC のリトライ増幅を防ぐ
//! 3. **例外の明示**: retry opt-in は ADR 参照付きで管理
//! 4. **共通ビルダー**: サービスが生の client/channel を直接組まない
//!
//! # 使用例
//!
//! ```rust
//! use k1s0_grpc_client::{
//!     GrpcClientBuilder, GrpcClientConfig, ServiceDiscoveryConfig, ServiceEndpoint,
//!     CallOptions, RequestMetadata,
//! };
//!
//! // クライアント接続の構築
//! let conn = GrpcClientBuilder::new("my-service")
//!     .target_address("localhost:50051")
//!     .build()
//!     .unwrap();
//!
//! // 呼び出しオプションの構築
//! let options = CallOptions::new()
//!     .with_timeout_ms(5000)
//!     .with_trace_id("abc123")
//!     .with_request_id("req-001");
//! ```
//!
//! # サービスディスカバリ
//!
//! K8s DNS ベースの論理名解決をサポート:
//!
//! ```rust
//! use k1s0_grpc_client::{ServiceDiscoveryConfig, ServiceEndpoint, ServiceResolver};
//!
//! let config = ServiceDiscoveryConfig::builder()
//!     .default_namespace("default")
//!     .service("auth-service", ServiceEndpoint::new("auth.example.com", 50051))
//!     .build();
//!
//! let resolver = ServiceResolver::new(config);
//!
//! // 明示的に登録されたサービス
//! let addr = resolver.resolve("auth-service").unwrap();
//! assert_eq!(addr, "auth.example.com:50051");
//!
//! // K8s DNS 解決
//! let addr = resolver.resolve("config-service").unwrap();
//! assert!(addr.contains("config-service.default"));
//! ```
//!
//! # リトライ設定（opt-in）
//!
//! リトライは原則無効。有効にするには ADR 参照が必須:
//!
//! ```rust
//! use k1s0_grpc_client::{GrpcClientConfig, RetryConfig};
//!
//! // リトライ無効（デフォルト）
//! let config = GrpcClientConfig::default();
//! assert!(!config.retry().is_enabled());
//!
//! // リトライ有効（ADR 必須）
//! let retry = RetryConfig::enabled("ADR-001")
//!     .max_attempts(3)
//!     .build()
//!     .unwrap();
//!
//! let config = GrpcClientConfig::builder()
//!     .retry(retry)
//!     .build()
//!     .unwrap();
//! ```
//!
//! # 依存先設定（config/{env}.yaml から読み込み）
//!
//! サービス実装が「生のホスト名/URL」を持たずに依存先を解決:
//!
//! ```rust
//! use k1s0_grpc_client::{DependenciesConfig, ServiceDependency, ServiceDiscoveryConfig};
//!
//! // 依存先設定の構築
//! let deps = DependenciesConfig::builder()
//!     .discovery(
//!         ServiceDiscoveryConfig::builder()
//!             .default_namespace("production")
//!             .default_port(50051)
//!             .build()
//!     )
//!     // 論理名で解決（K8s DNS）
//!     .service("auth-service", ServiceDependency::logical().timeout_ms(5000))
//!     // 明示的なエンドポイント
//!     .explicit_service("external-api", "https://api.example.com:443")
//!     .build();
//!
//! // エンドポイントの解決
//! let resolved = deps.resolve_endpoint("auth-service").unwrap();
//! assert!(resolved.address().contains("auth-service.production"));
//! ```

pub mod builder;
pub mod config;
pub mod discovery;
pub mod error;
pub mod interceptors;
pub mod stream;

// Re-exports
pub use builder::{CallOptions, ChannelPoolConfig, GrpcClientBuilder, GrpcClientConnection};
pub use config::{
    GrpcClientConfig, GrpcClientConfigBuilder, RetryConfig, RetryConfigBuilder, TlsConfig,
    TlsConfigBuilder, DEFAULT_CONNECT_TIMEOUT_MS, DEFAULT_TIMEOUT_MS, MAX_TIMEOUT_MS,
    MIN_TIMEOUT_MS,
};
pub use discovery::{
    DependenciesConfig, DependenciesConfigBuilder, ResolvedEndpoint, ServiceDependency,
    ServiceDiscoveryConfig, ServiceDiscoveryConfigBuilder, ServiceEndpoint, ServiceResolver,
    validate_service_name,
};
pub use error::{GrpcClientError, GrpcStatus};
pub use interceptors::{
    ClientMetricLabels, InterceptorContext, InterceptorResult, MetadataKeys, RequestMetadata,
    ResponseMetadata,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_usage() {
        // クライアント接続の構築
        let conn = GrpcClientBuilder::new("my-service")
            .target_address("localhost:50051")
            .build()
            .unwrap();

        assert_eq!(conn.service_name(), "my-service");
        assert_eq!(conn.address(), "localhost:50051");
    }

    #[test]
    fn test_with_discovery() {
        let config = ServiceDiscoveryConfig::builder()
            .default_namespace("default")
            .service(
                "auth-service",
                ServiceEndpoint::new("auth.example.com", 50051),
            )
            .build();

        let conn = GrpcClientBuilder::new("my-service")
            .target_service("auth-service")
            .discovery(config)
            .build()
            .unwrap();

        assert_eq!(conn.target_service(), "auth-service");
        assert_eq!(conn.endpoint().host(), "auth.example.com");
    }

    #[test]
    fn test_with_config() {
        let config = GrpcClientConfig::builder()
            .timeout_ms(10_000)
            .connect_timeout_ms(2_000)
            .build()
            .unwrap();

        let conn = GrpcClientBuilder::new("my-service")
            .target_address("localhost:50051")
            .config(config)
            .build()
            .unwrap();

        assert_eq!(conn.timeout().as_millis(), 10_000);
        assert_eq!(conn.connect_timeout().as_millis(), 2_000);
    }

    #[test]
    fn test_retry_disabled_by_default() {
        let config = GrpcClientConfig::default();
        assert!(!config.retry().is_enabled());
    }

    #[test]
    fn test_retry_requires_adr() {
        // デフォルト（disabled）のリトライ設定はバリデーションが通る
        let retry = RetryConfig::disabled();
        assert!(retry.validate().is_ok());
        assert!(!retry.is_enabled());
    }

    #[test]
    fn test_call_options() {
        let options = CallOptions::new()
            .with_timeout_ms(5000)
            .with_trace_id("abc123")
            .with_request_id("req-001")
            .with_tenant_id("tenant-1");

        assert_eq!(options.timeout_ms(), Some(5000));
        assert_eq!(
            options.metadata().trace_id,
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_request_metadata() {
        let metadata = RequestMetadata::new()
            .with_trace_id("abc123")
            .with_span_id("def456")
            .with_request_id("req-001");

        let traceparent = metadata.to_traceparent().unwrap();
        assert!(traceparent.contains("abc123"));
        assert!(traceparent.contains("def456"));
    }

    #[test]
    fn test_service_resolver() {
        let config = ServiceDiscoveryConfig::builder()
            .default_namespace("default")
            .build();

        let resolver = ServiceResolver::new(config);
        let addr = resolver.resolve("my-service").unwrap();
        assert!(addr.contains("my-service.default.svc.cluster.local"));
    }

    #[test]
    fn test_timeout_validation() {
        // 下限未満
        let result = GrpcClientConfig::builder().timeout_ms(50).build();
        assert!(result.is_err());

        // 上限超過
        let result = GrpcClientConfig::builder().timeout_ms(400_000).build();
        assert!(result.is_err());

        // 正常範囲
        let result = GrpcClientConfig::builder().timeout_ms(30_000).build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_grpc_status() {
        assert_eq!(GrpcStatus::Ok.code(), 0);
        assert_eq!(GrpcStatus::NotFound.code(), 5);
        assert_eq!(GrpcStatus::Internal.code(), 13);

        assert!(GrpcStatus::Unavailable.is_retryable());
        assert!(!GrpcStatus::NotFound.is_retryable());

        assert!(GrpcStatus::NotFound.is_client_error());
        assert!(GrpcStatus::Internal.is_server_error());
    }

    #[test]
    fn test_metadata_keys() {
        assert_eq!(MetadataKeys::TRACEPARENT, "traceparent");
        assert_eq!(MetadataKeys::X_REQUEST_ID, "x-request-id");
        assert_eq!(MetadataKeys::X_ERROR_CODE, "x-error-code");
    }

    #[test]
    fn test_dependencies_config_basic() {
        let deps = DependenciesConfig::builder()
            .discovery(
                ServiceDiscoveryConfig::builder()
                    .default_namespace("default")
                    .default_port(50051)
                    .build(),
            )
            .service(
                "auth-service",
                ServiceDependency::logical().timeout_ms(5000),
            )
            .explicit_service("external-api", "api.example.com:8080")
            .build();

        assert_eq!(deps.service_count(), 2);

        // 論理名で解決
        let auth = deps.resolve_endpoint("auth-service").unwrap();
        assert!(auth.address().contains("auth-service.default"));
        assert_eq!(auth.timeout_ms, 5000);

        // 明示的なエンドポイント
        let external = deps.resolve_endpoint("external-api").unwrap();
        assert_eq!(external.endpoint.host(), "api.example.com");
        assert_eq!(external.endpoint.port(), 8080);
    }

    #[test]
    fn test_validate_service_name_basic() {
        assert!(validate_service_name("auth-service").is_ok());
        assert!(validate_service_name("config-service").is_ok());
        assert!(validate_service_name("Auth-Service").is_err());
        assert!(validate_service_name("-service").is_err());
        assert!(validate_service_name("service-").is_err());
    }
}
