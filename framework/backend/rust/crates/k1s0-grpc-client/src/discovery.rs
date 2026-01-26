//! サービスディスカバリ
//!
//! 論理名からアドレスを解決する仕組みを提供する。
//! K8s DNS ベースの解決を前提とし、環境差は設定で吸収する。
//!
//! # 設定例
//!
//! ```yaml
//! # config/{env}.yaml
//! dependencies:
//!   discovery:
//!     default_namespace: default
//!     cluster_domain: svc.cluster.local
//!     default_port: 50051
//!   services:
//!     auth-service:
//!       # 論理名のみの場合は K8s DNS で解決
//!       timeout_ms: 5000
//!     config-service:
//!       # 明示的なエンドポイント指定
//!       endpoint: config-service.production:50051
//!       timeout_ms: 10000
//!     external-api:
//!       # 外部サービス
//!       endpoint: https://api.external.example.com:443
//!       tls: true
//!       timeout_ms: 30000
//! ```

use crate::error::GrpcClientError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// サービスディスカバリ設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    /// サービスマッピング（論理名 → エンドポイント情報）
    #[serde(default)]
    services: HashMap<String, ServiceEndpoint>,
    /// デフォルトの namespace（K8s）
    #[serde(default)]
    default_namespace: Option<String>,
    /// デフォルトのクラスタドメイン（K8s）
    #[serde(default = "default_cluster_domain")]
    cluster_domain: String,
    /// デフォルトのポート
    #[serde(default = "default_port")]
    default_port: u16,
}

fn default_cluster_domain() -> String {
    "svc.cluster.local".to_string()
}

fn default_port() -> u16 {
    50051
}

impl ServiceDiscoveryConfig {
    /// 新しい設定を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// ビルダーを作成
    pub fn builder() -> ServiceDiscoveryConfigBuilder {
        ServiceDiscoveryConfigBuilder::new()
    }

    /// サービスを登録
    pub fn register(&mut self, name: impl Into<String>, endpoint: ServiceEndpoint) {
        self.services.insert(name.into(), endpoint);
    }

    /// サービスエンドポイントを取得
    pub fn get(&self, name: &str) -> Option<&ServiceEndpoint> {
        self.services.get(name)
    }

    /// 論理名からアドレスを解決
    ///
    /// 解決優先順位:
    /// 1. 明示的に登録されたアドレス
    /// 2. K8s DNS 形式（`{service}.{namespace}.{cluster_domain}:{port}`）
    pub fn resolve(&self, logical_name: &str) -> Result<String, GrpcClientError> {
        // 明示的に登録されている場合
        if let Some(endpoint) = self.services.get(logical_name) {
            return Ok(endpoint.to_address());
        }

        // K8s DNS 形式で解決
        self.resolve_k8s_dns(logical_name)
    }

    /// K8s DNS 形式でアドレスを解決
    fn resolve_k8s_dns(&self, logical_name: &str) -> Result<String, GrpcClientError> {
        // 論理名のパース（形式: `{service}` or `{service}.{namespace}`）
        let parts: Vec<&str> = logical_name.split('.').collect();

        let (service, namespace) = match parts.len() {
            1 => {
                let service = parts[0];
                let namespace = self.default_namespace.as_deref().ok_or_else(|| {
                    GrpcClientError::service_discovery(format!(
                        "cannot resolve '{}': no namespace specified and default_namespace is not set",
                        logical_name
                    ))
                })?;
                (service, namespace)
            }
            2 => (parts[0], parts[1]),
            _ => {
                // 既に完全修飾名（FQDN）の場合はそのまま使用
                return Ok(format!("{}:{}", logical_name, self.default_port));
            }
        };

        Ok(format!(
            "{}.{}.{}:{}",
            service, namespace, self.cluster_domain, self.default_port
        ))
    }

    /// デフォルト namespace を取得
    pub fn default_namespace(&self) -> Option<&str> {
        self.default_namespace.as_deref()
    }

    /// クラスタドメインを取得
    pub fn cluster_domain(&self) -> &str {
        &self.cluster_domain
    }

    /// デフォルトポートを取得
    pub fn default_port(&self) -> u16 {
        self.default_port
    }

    /// 登録されているサービス数を取得
    pub fn service_count(&self) -> usize {
        self.services.len()
    }
}

/// サービスディスカバリ設定ビルダー
#[derive(Debug, Default)]
pub struct ServiceDiscoveryConfigBuilder {
    services: HashMap<String, ServiceEndpoint>,
    default_namespace: Option<String>,
    cluster_domain: Option<String>,
    default_port: Option<u16>,
}

impl ServiceDiscoveryConfigBuilder {
    /// 新しいビルダーを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// サービスを登録
    pub fn service(mut self, name: impl Into<String>, endpoint: ServiceEndpoint) -> Self {
        self.services.insert(name.into(), endpoint);
        self
    }

    /// デフォルト namespace を設定
    pub fn default_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.default_namespace = Some(namespace.into());
        self
    }

    /// クラスタドメインを設定
    pub fn cluster_domain(mut self, domain: impl Into<String>) -> Self {
        self.cluster_domain = Some(domain.into());
        self
    }

    /// デフォルトポートを設定
    pub fn default_port(mut self, port: u16) -> Self {
        self.default_port = Some(port);
        self
    }

    /// 設定をビルド
    pub fn build(self) -> ServiceDiscoveryConfig {
        ServiceDiscoveryConfig {
            services: self.services,
            default_namespace: self.default_namespace,
            cluster_domain: self.cluster_domain.unwrap_or_else(default_cluster_domain),
            default_port: self.default_port.unwrap_or_else(default_port),
        }
    }
}

/// サービスエンドポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// ホスト名または IP アドレス
    host: String,
    /// ポート番号
    port: u16,
    /// TLS を使用するかどうか
    #[serde(default)]
    tls: bool,
}

impl ServiceEndpoint {
    /// 新しいエンドポイントを作成
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            tls: false,
        }
    }

    /// TLS 付きのエンドポイントを作成
    pub fn with_tls(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            tls: true,
        }
    }

    /// アドレス文字列から作成
    ///
    /// 形式: `host:port` or `https://host:port`
    pub fn from_address(address: &str) -> Result<Self, GrpcClientError> {
        let (address, tls) = if address.starts_with("https://") {
            (address.trim_start_matches("https://"), true)
        } else if address.starts_with("http://") {
            (address.trim_start_matches("http://"), false)
        } else {
            (address, false)
        };

        let parts: Vec<&str> = address.split(':').collect();
        match parts.len() {
            1 => Ok(Self {
                host: parts[0].to_string(),
                port: if tls { 443 } else { default_port() },
                tls,
            }),
            2 => {
                let port = parts[1].parse().map_err(|_| {
                    GrpcClientError::service_discovery(format!("invalid port: {}", parts[1]))
                })?;
                Ok(Self {
                    host: parts[0].to_string(),
                    port,
                    tls,
                })
            }
            _ => Err(GrpcClientError::service_discovery(format!(
                "invalid address format: {}",
                address
            ))),
        }
    }

    /// ホストを取得
    pub fn host(&self) -> &str {
        &self.host
    }

    /// ポートを取得
    pub fn port(&self) -> u16 {
        self.port
    }

    /// TLS を使用するかどうか
    pub fn is_tls(&self) -> bool {
        self.tls
    }

    /// アドレス文字列を取得（`host:port` 形式）
    pub fn to_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// URI を取得
    pub fn to_uri(&self) -> String {
        let scheme = if self.tls { "https" } else { "http" };
        format!("{}://{}:{}", scheme, self.host, self.port)
    }
}

/// サービスリゾルバ
///
/// 論理名からアドレスを解決する。
#[derive(Debug, Clone)]
pub struct ServiceResolver {
    config: ServiceDiscoveryConfig,
}

impl ServiceResolver {
    /// 設定から作成
    pub fn new(config: ServiceDiscoveryConfig) -> Self {
        Self { config }
    }

    /// 論理名からアドレスを解決
    pub fn resolve(&self, logical_name: &str) -> Result<String, GrpcClientError> {
        self.config.resolve(logical_name)
    }

    /// 論理名からエンドポイントを解決
    pub fn resolve_endpoint(&self, logical_name: &str) -> Result<ServiceEndpoint, GrpcClientError> {
        // 明示的に登録されている場合
        if let Some(endpoint) = self.config.get(logical_name) {
            return Ok(endpoint.clone());
        }

        // 解決してエンドポイントを作成
        let address = self.config.resolve(logical_name)?;
        ServiceEndpoint::from_address(&address)
    }

    /// 設定を取得
    pub fn config(&self) -> &ServiceDiscoveryConfig {
        &self.config
    }
}

impl Default for ServiceResolver {
    fn default() -> Self {
        Self::new(ServiceDiscoveryConfig::default())
    }
}

/// 依存先サービス設定
///
/// config/{env}.yaml の `dependencies` セクションに対応。
/// サービス実装が「生のホスト名/URL」を持たずに依存先を解決できるようにする。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependenciesConfig {
    /// サービスディスカバリ設定
    #[serde(default)]
    discovery: ServiceDiscoveryConfig,
    /// 依存先サービスの定義
    #[serde(default)]
    services: HashMap<String, ServiceDependency>,
}

impl DependenciesConfig {
    /// 新しい設定を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// ビルダーを作成
    pub fn builder() -> DependenciesConfigBuilder {
        DependenciesConfigBuilder::new()
    }

    /// ディスカバリ設定を取得
    pub fn discovery(&self) -> &ServiceDiscoveryConfig {
        &self.discovery
    }

    /// 依存先サービスを取得
    pub fn get_service(&self, name: &str) -> Option<&ServiceDependency> {
        self.services.get(name)
    }

    /// 登録されているサービス名を取得
    pub fn service_names(&self) -> impl Iterator<Item = &str> {
        self.services.keys().map(|s| s.as_str())
    }

    /// サービス数を取得
    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    /// 依存先のエンドポイントを解決
    ///
    /// 1. ServiceDependency に endpoint が明示されていればそれを使用
    /// 2. なければ ServiceDiscoveryConfig で論理名から解決
    pub fn resolve_endpoint(&self, service_name: &str) -> Result<ResolvedEndpoint, GrpcClientError> {
        let dependency = self.services.get(service_name);

        // エンドポイントの解決
        let endpoint = if let Some(dep) = dependency {
            if let Some(explicit_endpoint) = &dep.endpoint {
                ServiceEndpoint::from_address(explicit_endpoint)?
            } else {
                // 論理名で解決
                let address = self.discovery.resolve(service_name)?;
                let mut ep = ServiceEndpoint::from_address(&address)?;
                if dep.tls {
                    ep = ServiceEndpoint::with_tls(ep.host(), ep.port());
                }
                ep
            }
        } else {
            // 未登録のサービスは論理名で解決を試みる
            let address = self.discovery.resolve(service_name)?;
            ServiceEndpoint::from_address(&address)?
        };

        // タイムアウト設定
        let timeout_ms = dependency.map(|d| d.timeout_ms).unwrap_or(DEFAULT_TIMEOUT_MS);

        Ok(ResolvedEndpoint {
            service_name: service_name.to_string(),
            endpoint,
            timeout_ms,
        })
    }

    /// すべての依存先を解決
    pub fn resolve_all(&self) -> Vec<Result<ResolvedEndpoint, GrpcClientError>> {
        self.services
            .keys()
            .map(|name| self.resolve_endpoint(name))
            .collect()
    }

    /// 設定をバリデーション
    pub fn validate(&self) -> Result<(), GrpcClientError> {
        for (name, dep) in &self.services {
            // タイムアウトの検証
            if dep.timeout_ms < MIN_TIMEOUT_MS {
                return Err(GrpcClientError::config(format!(
                    "service '{}': timeout_ms {} is below minimum {}",
                    name, dep.timeout_ms, MIN_TIMEOUT_MS
                )));
            }
            if dep.timeout_ms > MAX_TIMEOUT_MS {
                return Err(GrpcClientError::config(format!(
                    "service '{}': timeout_ms {} exceeds maximum {}",
                    name, dep.timeout_ms, MAX_TIMEOUT_MS
                )));
            }

            // エンドポイントの検証
            if let Some(endpoint) = &dep.endpoint {
                ServiceEndpoint::from_address(endpoint)?;
            }
        }
        Ok(())
    }
}

/// 依存先設定のデフォルトタイムアウト（ミリ秒）
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// 最小タイムアウト（ミリ秒）
pub const MIN_TIMEOUT_MS: u64 = 100;

/// 最大タイムアウト（ミリ秒）
pub const MAX_TIMEOUT_MS: u64 = 300_000;

/// 依存先サービスの定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    /// 明示的なエンドポイント（指定しない場合は論理名で解決）
    #[serde(skip_serializing_if = "Option::is_none")]
    endpoint: Option<String>,
    /// TLS を使用するかどうか
    #[serde(default)]
    tls: bool,
    /// タイムアウト（ミリ秒）
    #[serde(default = "default_timeout_ms")]
    timeout_ms: u64,
    /// 説明（ドキュメント用）
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

fn default_timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

impl ServiceDependency {
    /// 新しい依存先を作成（論理名解決）
    pub fn logical() -> Self {
        Self {
            endpoint: None,
            tls: false,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            description: None,
        }
    }

    /// 明示的なエンドポイントで作成
    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: Some(endpoint.into()),
            tls: false,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            description: None,
        }
    }

    /// TLS を設定
    pub fn tls(mut self, enabled: bool) -> Self {
        self.tls = enabled;
        self
    }

    /// タイムアウトを設定
    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// 説明を設定
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// エンドポイントを取得
    pub fn get_endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }

    /// TLS が有効かどうか
    pub fn is_tls(&self) -> bool {
        self.tls
    }

    /// タイムアウトを取得
    pub fn get_timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    /// 説明を取得
    pub fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

impl Default for ServiceDependency {
    fn default() -> Self {
        Self::logical()
    }
}

/// 解決済みエンドポイント
#[derive(Debug, Clone)]
pub struct ResolvedEndpoint {
    /// サービス名
    pub service_name: String,
    /// エンドポイント
    pub endpoint: ServiceEndpoint,
    /// タイムアウト（ミリ秒）
    pub timeout_ms: u64,
}

impl ResolvedEndpoint {
    /// アドレスを取得
    pub fn address(&self) -> String {
        self.endpoint.to_address()
    }

    /// URI を取得
    pub fn uri(&self) -> String {
        self.endpoint.to_uri()
    }
}

/// 依存先設定ビルダー
#[derive(Debug, Default)]
pub struct DependenciesConfigBuilder {
    discovery: Option<ServiceDiscoveryConfig>,
    services: HashMap<String, ServiceDependency>,
}

impl DependenciesConfigBuilder {
    /// 新しいビルダーを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// ディスカバリ設定を設定
    pub fn discovery(mut self, config: ServiceDiscoveryConfig) -> Self {
        self.discovery = Some(config);
        self
    }

    /// 依存先サービスを追加
    pub fn service(mut self, name: impl Into<String>, dep: ServiceDependency) -> Self {
        self.services.insert(name.into(), dep);
        self
    }

    /// 論理名で解決するサービスを追加
    pub fn logical_service(self, name: impl Into<String>) -> Self {
        self.service(name, ServiceDependency::logical())
    }

    /// 明示的なエンドポイントを持つサービスを追加
    pub fn explicit_service(
        self,
        name: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        self.service(name, ServiceDependency::with_endpoint(endpoint))
    }

    /// 設定をビルド
    pub fn build(self) -> DependenciesConfig {
        DependenciesConfig {
            discovery: self.discovery.unwrap_or_default(),
            services: self.services,
        }
    }
}

/// k1s0 規約に準拠したサービス名かどうかを検証
///
/// - kebab-case のみ許可
/// - 英小文字、数字、ハイフンのみ使用可能
/// - 先頭と末尾はハイフン不可
pub fn validate_service_name(name: &str) -> Result<(), GrpcClientError> {
    if name.is_empty() {
        return Err(GrpcClientError::config("service name cannot be empty"));
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err(GrpcClientError::config(format!(
            "service name '{}' cannot start or end with hyphen",
            name
        )));
    }

    if name.contains("--") {
        return Err(GrpcClientError::config(format!(
            "service name '{}' cannot contain consecutive hyphens",
            name
        )));
    }

    for c in name.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
            return Err(GrpcClientError::config(format!(
                "service name '{}' contains invalid character '{}' (only lowercase letters, digits, and hyphens allowed)",
                name, c
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_endpoint_new() {
        let endpoint = ServiceEndpoint::new("localhost", 50051);
        assert_eq!(endpoint.host(), "localhost");
        assert_eq!(endpoint.port(), 50051);
        assert!(!endpoint.is_tls());
        assert_eq!(endpoint.to_address(), "localhost:50051");
    }

    #[test]
    fn test_service_endpoint_with_tls() {
        let endpoint = ServiceEndpoint::with_tls("secure.example.com", 443);
        assert!(endpoint.is_tls());
        assert_eq!(endpoint.to_uri(), "https://secure.example.com:443");
    }

    #[test]
    fn test_service_endpoint_from_address() {
        let endpoint = ServiceEndpoint::from_address("localhost:50051").unwrap();
        assert_eq!(endpoint.host(), "localhost");
        assert_eq!(endpoint.port(), 50051);

        let endpoint = ServiceEndpoint::from_address("https://secure.example.com:443").unwrap();
        assert!(endpoint.is_tls());
        assert_eq!(endpoint.host(), "secure.example.com");
        assert_eq!(endpoint.port(), 443);

        let endpoint = ServiceEndpoint::from_address("http://localhost:8080").unwrap();
        assert!(!endpoint.is_tls());
        assert_eq!(endpoint.port(), 8080);
    }

    #[test]
    fn test_service_endpoint_from_address_default_port() {
        let endpoint = ServiceEndpoint::from_address("localhost").unwrap();
        assert_eq!(endpoint.port(), default_port());

        let endpoint = ServiceEndpoint::from_address("https://secure.example.com").unwrap();
        assert_eq!(endpoint.port(), 443);
    }

    #[test]
    fn test_service_discovery_config_explicit() {
        let config = ServiceDiscoveryConfig::builder()
            .service("auth-service", ServiceEndpoint::new("auth.example.com", 50051))
            .build();

        let address = config.resolve("auth-service").unwrap();
        assert_eq!(address, "auth.example.com:50051");
    }

    #[test]
    fn test_service_discovery_config_k8s_dns() {
        let config = ServiceDiscoveryConfig::builder()
            .default_namespace("default")
            .cluster_domain("svc.cluster.local")
            .default_port(50051)
            .build();

        // service のみ
        let address = config.resolve("auth-service").unwrap();
        assert_eq!(address, "auth-service.default.svc.cluster.local:50051");

        // service.namespace
        let address = config.resolve("auth-service.production").unwrap();
        assert_eq!(
            address,
            "auth-service.production.svc.cluster.local:50051"
        );
    }

    #[test]
    fn test_service_discovery_config_no_namespace() {
        let config = ServiceDiscoveryConfig::builder().build();

        // namespace なしでサービス名のみは解決できない
        let result = config.resolve("auth-service");
        assert!(result.is_err());
    }

    #[test]
    fn test_service_discovery_config_fqdn() {
        let config = ServiceDiscoveryConfig::builder()
            .default_port(50051)
            .build();

        // FQDN はそのまま使用
        let address = config
            .resolve("auth-service.default.svc.cluster.local")
            .unwrap();
        assert_eq!(
            address,
            "auth-service.default.svc.cluster.local:50051"
        );
    }

    #[test]
    fn test_service_resolver() {
        let config = ServiceDiscoveryConfig::builder()
            .service("auth-service", ServiceEndpoint::new("auth.example.com", 50051))
            .default_namespace("default")
            .build();

        let resolver = ServiceResolver::new(config);

        // 明示的に登録されたサービス
        let endpoint = resolver.resolve_endpoint("auth-service").unwrap();
        assert_eq!(endpoint.host(), "auth.example.com");

        // K8s DNS 解決
        let address = resolver.resolve("config-service").unwrap();
        assert!(address.contains("config-service.default"));
    }

    #[test]
    fn test_service_discovery_config_register() {
        let mut config = ServiceDiscoveryConfig::new();
        config.register("my-service", ServiceEndpoint::new("localhost", 8080));

        assert_eq!(config.service_count(), 1);
        assert!(config.get("my-service").is_some());
    }

    // DependenciesConfig テスト

    #[test]
    fn test_dependencies_config_default() {
        let config = DependenciesConfig::new();
        assert_eq!(config.service_count(), 0);
    }

    #[test]
    fn test_dependencies_config_builder() {
        let config = DependenciesConfig::builder()
            .discovery(
                ServiceDiscoveryConfig::builder()
                    .default_namespace("production")
                    .build(),
            )
            .service(
                "auth-service",
                ServiceDependency::logical().timeout_ms(5000),
            )
            .explicit_service("external-api", "https://api.example.com:443")
            .build();

        assert_eq!(config.service_count(), 2);
        assert!(config.get_service("auth-service").is_some());
        assert!(config.get_service("external-api").is_some());
    }

    #[test]
    fn test_dependencies_config_resolve_logical() {
        let config = DependenciesConfig::builder()
            .discovery(
                ServiceDiscoveryConfig::builder()
                    .default_namespace("default")
                    .default_port(50051)
                    .build(),
            )
            .logical_service("auth-service")
            .build();

        let resolved = config.resolve_endpoint("auth-service").unwrap();
        assert_eq!(resolved.service_name, "auth-service");
        assert!(resolved.address().contains("auth-service.default"));
    }

    #[test]
    fn test_dependencies_config_resolve_explicit() {
        let config = DependenciesConfig::builder()
            .service(
                "external-api",
                ServiceDependency::with_endpoint("api.example.com:8080").timeout_ms(10000),
            )
            .build();

        let resolved = config.resolve_endpoint("external-api").unwrap();
        assert_eq!(resolved.endpoint.host(), "api.example.com");
        assert_eq!(resolved.endpoint.port(), 8080);
        assert_eq!(resolved.timeout_ms, 10000);
    }

    #[test]
    fn test_dependencies_config_resolve_with_tls() {
        let config = DependenciesConfig::builder()
            .discovery(
                ServiceDiscoveryConfig::builder()
                    .default_namespace("default")
                    .build(),
            )
            .service(
                "secure-service",
                ServiceDependency::logical().tls(true),
            )
            .build();

        let resolved = config.resolve_endpoint("secure-service").unwrap();
        // TLS フラグは endpoint に反映される（論理名解決の場合）
        assert!(resolved.endpoint.is_tls());
    }

    #[test]
    fn test_dependencies_config_validate_timeout() {
        let config = DependenciesConfig::builder()
            .service(
                "fast-service",
                ServiceDependency::with_endpoint("localhost:8080").timeout_ms(50), // 下限未満
            )
            .build();

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_dependencies_config_service_names() {
        let config = DependenciesConfig::builder()
            .logical_service("service-a")
            .logical_service("service-b")
            .logical_service("service-c")
            .build();

        let names: Vec<_> = config.service_names().collect();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"service-a"));
        assert!(names.contains(&"service-b"));
        assert!(names.contains(&"service-c"));
    }

    #[test]
    fn test_service_dependency_builder() {
        let dep = ServiceDependency::with_endpoint("localhost:8080")
            .tls(true)
            .timeout_ms(5000)
            .description("Test service");

        assert_eq!(dep.get_endpoint(), Some("localhost:8080"));
        assert!(dep.is_tls());
        assert_eq!(dep.get_timeout_ms(), 5000);
        assert_eq!(dep.get_description(), Some("Test service"));
    }

    #[test]
    fn test_resolved_endpoint() {
        let endpoint = ServiceEndpoint::new("localhost", 50051);
        let resolved = ResolvedEndpoint {
            service_name: "my-service".to_string(),
            endpoint,
            timeout_ms: 5000,
        };

        assert_eq!(resolved.address(), "localhost:50051");
        assert_eq!(resolved.uri(), "http://localhost:50051");
    }

    // validate_service_name テスト

    #[test]
    fn test_validate_service_name_valid() {
        assert!(validate_service_name("auth-service").is_ok());
        assert!(validate_service_name("config-service").is_ok());
        assert!(validate_service_name("api").is_ok());
        assert!(validate_service_name("service-v2").is_ok());
        assert!(validate_service_name("my-long-service-name").is_ok());
    }

    #[test]
    fn test_validate_service_name_empty() {
        assert!(validate_service_name("").is_err());
    }

    #[test]
    fn test_validate_service_name_uppercase() {
        assert!(validate_service_name("Auth-Service").is_err());
        assert!(validate_service_name("AUTH").is_err());
    }

    #[test]
    fn test_validate_service_name_underscore() {
        assert!(validate_service_name("auth_service").is_err());
    }

    #[test]
    fn test_validate_service_name_leading_hyphen() {
        assert!(validate_service_name("-service").is_err());
    }

    #[test]
    fn test_validate_service_name_trailing_hyphen() {
        assert!(validate_service_name("service-").is_err());
    }

    #[test]
    fn test_validate_service_name_consecutive_hyphens() {
        assert!(validate_service_name("auth--service").is_err());
    }

    #[test]
    fn test_validate_service_name_special_chars() {
        assert!(validate_service_name("auth.service").is_err());
        assert!(validate_service_name("auth@service").is_err());
        assert!(validate_service_name("auth service").is_err());
    }

    #[test]
    fn test_dependencies_config_yaml_serialization() {
        let config = DependenciesConfig::builder()
            .discovery(
                ServiceDiscoveryConfig::builder()
                    .default_namespace("default")
                    .default_port(50051)
                    .build(),
            )
            .service(
                "auth-service",
                ServiceDependency::logical()
                    .timeout_ms(5000)
                    .description("認証サービス"),
            )
            .explicit_service("external-api", "https://api.example.com:443")
            .build();

        // YAML にシリアライズできることを確認
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("auth-service"));
        assert!(yaml.contains("external-api"));

        // デシリアライズして元に戻せることを確認
        let deserialized: DependenciesConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.service_count(), 2);
    }
}
