//! gRPC クライアントビルダー
//!
//! 共通ビルダーで gRPC クライアントを初期化する。
//! deadline 必須、retry 原則 0 などのルールを強制する。

use crate::config::GrpcClientConfig;
use crate::discovery::{ServiceDiscoveryConfig, ServiceEndpoint, ServiceResolver};
use crate::error::GrpcClientError;
use crate::interceptors::RequestMetadata;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// gRPC クライアント接続情報
///
/// ビルダーで構築された接続情報を保持する。
#[derive(Debug, Clone)]
pub struct GrpcClientConnection {
    /// エンドポイント
    endpoint: ServiceEndpoint,
    /// クライアント設定
    config: GrpcClientConfig,
    /// サービス名（呼び出し元）
    service_name: String,
    /// ターゲットサービス名（呼び出し先）
    target_service: String,
}

impl GrpcClientConnection {
    /// エンドポイントを取得
    pub fn endpoint(&self) -> &ServiceEndpoint {
        &self.endpoint
    }

    /// クライアント設定を取得
    pub fn config(&self) -> &GrpcClientConfig {
        &self.config
    }

    /// サービス名を取得
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// ターゲットサービス名を取得
    pub fn target_service(&self) -> &str {
        &self.target_service
    }

    /// URI を取得
    pub fn uri(&self) -> String {
        self.endpoint.to_uri()
    }

    /// アドレスを取得
    pub fn address(&self) -> String {
        self.endpoint.to_address()
    }

    /// タイムアウトを取得
    pub fn timeout(&self) -> Duration {
        self.config.timeout()
    }

    /// 接続タイムアウトを取得
    pub fn connect_timeout(&self) -> Duration {
        self.config.connect_timeout()
    }

    /// TLS が有効かどうか
    pub fn is_tls(&self) -> bool {
        self.endpoint.is_tls() || self.config.tls().is_enabled()
    }

    /// リトライが有効かどうか
    pub fn is_retry_enabled(&self) -> bool {
        self.config.retry().is_enabled()
    }
}

/// gRPC クライアントビルダー
///
/// 共通の設定でクライアント接続を構築する。
#[derive(Debug)]
pub struct GrpcClientBuilder {
    service_name: String,
    target_service: Option<String>,
    target_address: Option<String>,
    config: Option<GrpcClientConfig>,
    discovery: Option<ServiceDiscoveryConfig>,
    default_metadata: RequestMetadata,
}

impl GrpcClientBuilder {
    /// 新しいビルダーを作成
    ///
    /// `service_name` は呼び出し元のサービス名（必須）。
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            target_service: None,
            target_address: None,
            config: None,
            discovery: None,
            default_metadata: RequestMetadata::new(),
        }
    }

    /// ターゲットサービスを設定（論理名）
    ///
    /// サービスディスカバリで解決される。
    pub fn target_service(mut self, service: impl Into<String>) -> Self {
        self.target_service = Some(service.into());
        self
    }

    /// ターゲットアドレスを直接設定
    ///
    /// 形式: `host:port` or `https://host:port`
    pub fn target_address(mut self, address: impl Into<String>) -> Self {
        self.target_address = Some(address.into());
        self
    }

    /// クライアント設定を設定
    pub fn config(mut self, config: GrpcClientConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// サービスディスカバリ設定を設定
    pub fn discovery(mut self, discovery: ServiceDiscoveryConfig) -> Self {
        self.discovery = Some(discovery);
        self
    }

    /// タイムアウトを設定（ミリ秒）
    ///
    /// config が設定されている場合は上書きされる。
    pub fn timeout_ms(mut self, ms: u64) -> Self {
        // 新しい config を作成（既存の設定は上書き）
        self.config = GrpcClientConfig::builder()
            .timeout_ms(ms)
            .build()
            .ok();
        self
    }

    /// デフォルトメタデータを設定
    pub fn default_metadata(mut self, metadata: RequestMetadata) -> Self {
        self.default_metadata = metadata;
        self
    }

    /// テナント ID をデフォルトメタデータに設定
    pub fn default_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.default_metadata = self.default_metadata.with_tenant_id(tenant_id);
        self
    }

    /// 接続をビルド
    pub fn build(self) -> Result<GrpcClientConnection, GrpcClientError> {
        // ターゲットの解決
        let (target_service, endpoint) = self.resolve_target()?;

        // 設定のバリデーション
        let config = self.config.unwrap_or_default();
        config.validate()?;

        Ok(GrpcClientConnection {
            endpoint,
            config,
            service_name: self.service_name,
            target_service,
        })
    }

    /// ターゲットを解決
    fn resolve_target(&self) -> Result<(String, ServiceEndpoint), GrpcClientError> {
        // 直接アドレスが指定されている場合
        if let Some(ref address) = self.target_address {
            let endpoint = ServiceEndpoint::from_address(address)?;
            let target_service = self
                .target_service
                .clone()
                .unwrap_or_else(|| address.clone());
            return Ok((target_service, endpoint));
        }

        // サービス名から解決
        let target_service = self.target_service.clone().ok_or_else(|| {
            GrpcClientError::config("either target_service or target_address is required")
        })?;

        let discovery = self.discovery.clone().unwrap_or_default();
        let resolver = ServiceResolver::new(discovery);
        let endpoint = resolver.resolve_endpoint(&target_service)?;

        Ok((target_service, endpoint))
    }
}

/// 呼び出しオプション
///
/// 個別の gRPC 呼び出しに対するオプション。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CallOptions {
    /// タイムアウト（ミリ秒）。設定されない場合はデフォルトを使用。
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_ms: Option<u64>,
    /// リクエストメタデータ
    #[serde(default)]
    metadata: RequestMetadata,
    /// 圧縮を有効にするか
    #[serde(default)]
    compression: bool,
}

impl CallOptions {
    /// 新しいオプションを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// タイムアウトを設定（ミリ秒）
    pub fn with_timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// タイムアウトを設定（Duration）
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout_ms = Some(duration.as_millis() as u64);
        self
    }

    /// メタデータを設定
    pub fn with_metadata(mut self, metadata: RequestMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// トレース ID を設定
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_trace_id(trace_id);
        self
    }

    /// リクエスト ID を設定
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_request_id(request_id);
        self
    }

    /// テナント ID を設定
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_tenant_id(tenant_id);
        self
    }

    /// ユーザー ID を設定
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_user_id(user_id);
        self
    }

    /// 圧縮を設定
    pub fn with_compression(mut self, compression: bool) -> Self {
        self.compression = compression;
        self
    }

    /// タイムアウト（ミリ秒）を取得
    pub fn timeout_ms(&self) -> Option<u64> {
        self.timeout_ms
    }

    /// タイムアウト（Duration）を取得
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout_ms.map(Duration::from_millis)
    }

    /// メタデータを取得
    pub fn metadata(&self) -> &RequestMetadata {
        &self.metadata
    }

    /// 圧縮が有効かどうか
    pub fn compression(&self) -> bool {
        self.compression
    }

    /// デフォルトメタデータとマージ
    pub fn merge_metadata(mut self, default: &RequestMetadata) -> Self {
        // デフォルトの値を引き継ぎ、明示的に設定された値で上書き
        if self.metadata.trace_id.is_none() {
            self.metadata.trace_id = default.trace_id.clone();
        }
        if self.metadata.span_id.is_none() {
            self.metadata.span_id = default.span_id.clone();
        }
        if self.metadata.request_id.is_none() {
            self.metadata.request_id = default.request_id.clone();
        }
        if self.metadata.tenant_id.is_none() {
            self.metadata.tenant_id = default.tenant_id.clone();
        }
        if self.metadata.user_id.is_none() {
            self.metadata.user_id = default.user_id.clone();
        }
        self
    }
}

/// チャネルプール設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPoolConfig {
    /// 最大接続数
    #[serde(default = "default_max_connections")]
    max_connections: usize,
    /// アイドルタイムアウト（秒）
    #[serde(default = "default_idle_timeout_secs")]
    idle_timeout_secs: u64,
}

fn default_max_connections() -> usize {
    10
}

fn default_idle_timeout_secs() -> u64 {
    60
}

impl ChannelPoolConfig {
    /// デフォルト設定を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 最大接続数を設定
    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// アイドルタイムアウトを設定（秒）
    pub fn with_idle_timeout_secs(mut self, secs: u64) -> Self {
        self.idle_timeout_secs = secs;
        self
    }

    /// 最大接続数を取得
    pub fn max_connections(&self) -> usize {
        self.max_connections
    }

    /// アイドルタイムアウトを取得
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }
}

impl Default for ChannelPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            idle_timeout_secs: default_idle_timeout_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_client_builder_with_address() {
        let conn = GrpcClientBuilder::new("my-service")
            .target_address("localhost:50051")
            .build()
            .unwrap();

        assert_eq!(conn.service_name(), "my-service");
        assert_eq!(conn.address(), "localhost:50051");
    }

    #[test]
    fn test_grpc_client_builder_with_service() {
        let discovery = ServiceDiscoveryConfig::builder()
            .service("auth-service", ServiceEndpoint::new("auth.example.com", 50051))
            .build();

        let conn = GrpcClientBuilder::new("my-service")
            .target_service("auth-service")
            .discovery(discovery)
            .build()
            .unwrap();

        assert_eq!(conn.target_service(), "auth-service");
        assert_eq!(conn.endpoint().host(), "auth.example.com");
    }

    #[test]
    fn test_grpc_client_builder_requires_target() {
        let result = GrpcClientBuilder::new("my-service").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_grpc_client_connection_properties() {
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

        assert_eq!(conn.timeout(), Duration::from_millis(10_000));
        assert_eq!(conn.connect_timeout(), Duration::from_millis(2_000));
        assert!(!conn.is_tls());
        assert!(!conn.is_retry_enabled());
    }

    #[test]
    fn test_grpc_client_connection_tls() {
        let conn = GrpcClientBuilder::new("my-service")
            .target_address("https://secure.example.com:443")
            .build()
            .unwrap();

        assert!(conn.is_tls());
        assert_eq!(conn.uri(), "https://secure.example.com:443");
    }

    #[test]
    fn test_call_options() {
        let options = CallOptions::new()
            .with_timeout_ms(5000)
            .with_trace_id("abc123")
            .with_request_id("req-001")
            .with_tenant_id("tenant-1");

        assert_eq!(options.timeout_ms(), Some(5000));
        assert_eq!(options.metadata().trace_id, Some("abc123".to_string()));
        assert_eq!(options.metadata().request_id, Some("req-001".to_string()));
        assert_eq!(options.metadata().tenant_id, Some("tenant-1".to_string()));
    }

    #[test]
    fn test_call_options_duration() {
        let options = CallOptions::new().with_timeout(Duration::from_secs(10));
        assert_eq!(options.timeout(), Some(Duration::from_secs(10)));
    }

    #[test]
    fn test_call_options_merge_metadata() {
        let default = RequestMetadata::new()
            .with_trace_id("default-trace")
            .with_tenant_id("default-tenant");

        let options = CallOptions::new()
            .with_trace_id("custom-trace") // 上書き
            .merge_metadata(&default);

        // trace_id は上書きされる
        assert_eq!(options.metadata().trace_id, Some("custom-trace".to_string()));
        // tenant_id はデフォルトが使われる
        assert_eq!(options.metadata().tenant_id, Some("default-tenant".to_string()));
    }

    #[test]
    fn test_channel_pool_config() {
        let config = ChannelPoolConfig::new()
            .with_max_connections(20)
            .with_idle_timeout_secs(120);

        assert_eq!(config.max_connections(), 20);
        assert_eq!(config.idle_timeout(), Duration::from_secs(120));
    }

    #[test]
    fn test_channel_pool_config_default() {
        let config = ChannelPoolConfig::default();
        assert_eq!(config.max_connections(), 10);
        assert_eq!(config.idle_timeout(), Duration::from_secs(60));
    }

    #[test]
    fn test_grpc_client_builder_default_metadata() {
        let conn = GrpcClientBuilder::new("my-service")
            .target_address("localhost:50051")
            .default_tenant_id("tenant-1")
            .build()
            .unwrap();

        assert_eq!(conn.service_name(), "my-service");
    }
}
