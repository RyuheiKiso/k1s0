//! サービスディスカバリ
//!
//! 論理名からアドレスを解決する仕組みを提供する。
//! K8s DNS ベースの解決を前提とし、環境差は設定で吸収する。

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
}
