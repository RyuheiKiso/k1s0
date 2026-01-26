//! gRPC サーバ設定
//!
//! サーバ初期化、インターセプタ設定を管理する。

use crate::error::GrpcServerError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// デフォルトのシャットダウンタイムアウト（秒）
pub const DEFAULT_SHUTDOWN_TIMEOUT_SECS: u64 = 30;

/// デフォルトのリクエストタイムアウト（秒）
pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 60;

/// gRPC サーバ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcServerConfig {
    /// サービス名（必須）
    service_name: String,
    /// 環境名（必須）
    env: String,
    /// バインドアドレス
    #[serde(default = "default_bind_address")]
    bind_address: String,
    /// ポート
    #[serde(default = "default_port")]
    port: u16,
    /// シャットダウンタイムアウト（秒）
    #[serde(default = "default_shutdown_timeout_secs")]
    shutdown_timeout_secs: u64,
    /// インターセプタ設定
    #[serde(default)]
    interceptors: InterceptorConfig,
    /// TLS 設定
    #[serde(default)]
    tls: TlsConfig,
}

fn default_bind_address() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    50051
}

fn default_shutdown_timeout_secs() -> u64 {
    DEFAULT_SHUTDOWN_TIMEOUT_SECS
}

impl GrpcServerConfig {
    /// ビルダーを作成
    pub fn builder() -> GrpcServerConfigBuilder {
        GrpcServerConfigBuilder::new()
    }

    /// 設定をバリデーション
    pub fn validate(&self) -> Result<(), GrpcServerError> {
        if self.service_name.is_empty() {
            return Err(GrpcServerError::config("service_name is required"));
        }

        if self.env.is_empty() {
            return Err(GrpcServerError::config("env is required"));
        }

        if self.port == 0 {
            return Err(GrpcServerError::config("port cannot be 0"));
        }

        Ok(())
    }

    /// サービス名を取得
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// 環境名を取得
    pub fn env(&self) -> &str {
        &self.env
    }

    /// バインドアドレスを取得
    pub fn bind_address(&self) -> &str {
        &self.bind_address
    }

    /// ポートを取得
    pub fn port(&self) -> u16 {
        self.port
    }

    /// ソケットアドレスを取得
    pub fn socket_address(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }

    /// シャットダウンタイムアウトを取得
    pub fn shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.shutdown_timeout_secs)
    }

    /// インターセプタ設定を取得
    pub fn interceptors(&self) -> &InterceptorConfig {
        &self.interceptors
    }

    /// TLS 設定を取得
    pub fn tls(&self) -> &TlsConfig {
        &self.tls
    }
}

/// gRPC サーバ設定ビルダー
#[derive(Debug, Default)]
pub struct GrpcServerConfigBuilder {
    service_name: Option<String>,
    env: Option<String>,
    bind_address: Option<String>,
    port: Option<u16>,
    shutdown_timeout_secs: Option<u64>,
    interceptors: Option<InterceptorConfig>,
    tls: Option<TlsConfig>,
}

impl GrpcServerConfigBuilder {
    /// 新しいビルダーを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// サービス名を設定（必須）
    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = Some(name.into());
        self
    }

    /// 環境名を設定（必須）
    pub fn env(mut self, env: impl Into<String>) -> Self {
        self.env = Some(env.into());
        self
    }

    /// バインドアドレスを設定
    pub fn bind_address(mut self, address: impl Into<String>) -> Self {
        self.bind_address = Some(address.into());
        self
    }

    /// ポートを設定
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// シャットダウンタイムアウトを設定（秒）
    pub fn shutdown_timeout_secs(mut self, secs: u64) -> Self {
        self.shutdown_timeout_secs = Some(secs);
        self
    }

    /// インターセプタ設定を設定
    pub fn interceptors(mut self, config: InterceptorConfig) -> Self {
        self.interceptors = Some(config);
        self
    }

    /// TLS 設定を設定
    pub fn tls(mut self, config: TlsConfig) -> Self {
        self.tls = Some(config);
        self
    }

    /// 設定をビルド
    pub fn build(self) -> Result<GrpcServerConfig, GrpcServerError> {
        let config = GrpcServerConfig {
            service_name: self
                .service_name
                .ok_or_else(|| GrpcServerError::config("service_name is required"))?,
            env: self
                .env
                .ok_or_else(|| GrpcServerError::config("env is required"))?,
            bind_address: self.bind_address.unwrap_or_else(default_bind_address),
            port: self.port.unwrap_or_else(default_port),
            shutdown_timeout_secs: self
                .shutdown_timeout_secs
                .unwrap_or_else(default_shutdown_timeout_secs),
            interceptors: self.interceptors.unwrap_or_default(),
            tls: self.tls.unwrap_or_default(),
        };

        config.validate()?;
        Ok(config)
    }
}

/// インターセプタ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptorConfig {
    /// トレースコンテキスト伝播を有効にする
    #[serde(default = "default_true")]
    pub trace_propagation: bool,
    /// error_code 付与を有効にする
    #[serde(default = "default_true")]
    pub error_code_attachment: bool,
    /// request_id 相関を有効にする
    #[serde(default = "default_true")]
    pub request_id_correlation: bool,
    /// テナント情報読み取りを有効にする
    #[serde(default = "default_true")]
    pub tenant_extraction: bool,
    /// デッドライン検知を有効にする
    #[serde(default = "default_true")]
    pub deadline_detection: bool,
    /// デッドライン未指定時の動作
    #[serde(default)]
    pub deadline_policy: DeadlinePolicy,
    /// メトリクス出力を有効にする
    #[serde(default = "default_true")]
    pub metrics: bool,
    /// リクエストログを有効にする
    #[serde(default = "default_true")]
    pub request_logging: bool,
}

fn default_true() -> bool {
    true
}

impl Default for InterceptorConfig {
    fn default() -> Self {
        Self {
            trace_propagation: true,
            error_code_attachment: true,
            request_id_correlation: true,
            tenant_extraction: true,
            deadline_detection: true,
            deadline_policy: DeadlinePolicy::default(),
            metrics: true,
            request_logging: true,
        }
    }
}

/// デッドライン未指定時のポリシー
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeadlinePolicy {
    /// 許可（ログ/メトリクス出力のみ）
    Allow,
    /// 警告（ログ/メトリクス出力、ヘッダで通知）
    Warn,
    /// 拒否（INVALID_ARGUMENT で返す）
    Reject,
}

impl Default for DeadlinePolicy {
    fn default() -> Self {
        Self::Warn
    }
}

impl DeadlinePolicy {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Warn => "warn",
            Self::Reject => "reject",
        }
    }
}

/// TLS 設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlsConfig {
    /// TLS 有効
    #[serde(default)]
    enabled: bool,
    /// サーバ証明書パス
    #[serde(skip_serializing_if = "Option::is_none")]
    cert_path: Option<String>,
    /// サーバ秘密鍵パス
    #[serde(skip_serializing_if = "Option::is_none")]
    key_path: Option<String>,
    /// クライアント CA 証明書パス（mTLS 用）
    #[serde(skip_serializing_if = "Option::is_none")]
    client_ca_path: Option<String>,
}

impl TlsConfig {
    /// TLS 無効の設定を作成
    pub fn disabled() -> Self {
        Self::default()
    }

    /// TLS 有効の設定ビルダーを作成
    pub fn enabled(cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        Self {
            enabled: true,
            cert_path: Some(cert_path.into()),
            key_path: Some(key_path.into()),
            client_ca_path: None,
        }
    }

    /// mTLS 有効の設定を作成
    pub fn mtls(
        cert_path: impl Into<String>,
        key_path: impl Into<String>,
        client_ca_path: impl Into<String>,
    ) -> Self {
        Self {
            enabled: true,
            cert_path: Some(cert_path.into()),
            key_path: Some(key_path.into()),
            client_ca_path: Some(client_ca_path.into()),
        }
    }

    /// TLS が有効かどうか
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 証明書パスを取得
    pub fn cert_path(&self) -> Option<&str> {
        self.cert_path.as_deref()
    }

    /// 秘密鍵パスを取得
    pub fn key_path(&self) -> Option<&str> {
        self.key_path.as_deref()
    }

    /// クライアント CA パスを取得
    pub fn client_ca_path(&self) -> Option<&str> {
        self.client_ca_path.as_deref()
    }

    /// mTLS が有効かどうか
    pub fn is_mtls(&self) -> bool {
        self.enabled && self.client_ca_path.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_server_config_builder() {
        let config = GrpcServerConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        assert_eq!(config.service_name(), "test-service");
        assert_eq!(config.env(), "dev");
        assert_eq!(config.bind_address(), "0.0.0.0");
        assert_eq!(config.port(), 50051);
    }

    #[test]
    fn test_grpc_server_config_requires_service_name() {
        let result = GrpcServerConfig::builder().env("dev").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_grpc_server_config_requires_env() {
        let result = GrpcServerConfig::builder()
            .service_name("test-service")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_grpc_server_config_socket_address() {
        let config = GrpcServerConfig::builder()
            .service_name("test-service")
            .env("dev")
            .bind_address("127.0.0.1")
            .port(8080)
            .build()
            .unwrap();

        assert_eq!(config.socket_address(), "127.0.0.1:8080");
    }

    #[test]
    fn test_interceptor_config_default() {
        let config = InterceptorConfig::default();

        assert!(config.trace_propagation);
        assert!(config.error_code_attachment);
        assert!(config.request_id_correlation);
        assert!(config.tenant_extraction);
        assert!(config.deadline_detection);
        assert!(config.metrics);
        assert!(config.request_logging);
        assert_eq!(config.deadline_policy, DeadlinePolicy::Warn);
    }

    #[test]
    fn test_deadline_policy() {
        assert_eq!(DeadlinePolicy::Allow.as_str(), "allow");
        assert_eq!(DeadlinePolicy::Warn.as_str(), "warn");
        assert_eq!(DeadlinePolicy::Reject.as_str(), "reject");
    }

    #[test]
    fn test_tls_config_disabled() {
        let config = TlsConfig::disabled();
        assert!(!config.is_enabled());
        assert!(!config.is_mtls());
    }

    #[test]
    fn test_tls_config_enabled() {
        let config = TlsConfig::enabled("/etc/ssl/server.crt", "/etc/ssl/server.key");
        assert!(config.is_enabled());
        assert!(!config.is_mtls());
        assert_eq!(config.cert_path(), Some("/etc/ssl/server.crt"));
        assert_eq!(config.key_path(), Some("/etc/ssl/server.key"));
    }

    #[test]
    fn test_tls_config_mtls() {
        let config = TlsConfig::mtls(
            "/etc/ssl/server.crt",
            "/etc/ssl/server.key",
            "/etc/ssl/ca.crt",
        );
        assert!(config.is_enabled());
        assert!(config.is_mtls());
        assert_eq!(config.client_ca_path(), Some("/etc/ssl/ca.crt"));
    }
}
