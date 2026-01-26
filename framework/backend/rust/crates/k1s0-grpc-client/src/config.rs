//! gRPC クライアント設定
//!
//! タイムアウト、リトライ、サービスディスカバリの設定を管理する。

use crate::error::GrpcClientError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// デフォルトのタイムアウト（ミリ秒）
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000; // 30 seconds

/// 最小タイムアウト（ミリ秒）
pub const MIN_TIMEOUT_MS: u64 = 100; // 100ms

/// 最大タイムアウト（ミリ秒）
pub const MAX_TIMEOUT_MS: u64 = 300_000; // 5 minutes

/// デフォルトの接続タイムアウト（ミリ秒）
pub const DEFAULT_CONNECT_TIMEOUT_MS: u64 = 5_000; // 5 seconds

/// gRPC クライアント設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcClientConfig {
    /// リクエストタイムアウト（ミリ秒）
    #[serde(default = "default_timeout_ms")]
    timeout_ms: u64,
    /// 接続タイムアウト（ミリ秒）
    #[serde(default = "default_connect_timeout_ms")]
    connect_timeout_ms: u64,
    /// リトライポリシー（既定は無効）
    #[serde(default)]
    retry: RetryConfig,
    /// TLS 設定
    #[serde(default)]
    tls: TlsConfig,
}

fn default_timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

fn default_connect_timeout_ms() -> u64 {
    DEFAULT_CONNECT_TIMEOUT_MS
}

impl GrpcClientConfig {
    /// ビルダーを作成
    pub fn builder() -> GrpcClientConfigBuilder {
        GrpcClientConfigBuilder::new()
    }

    /// 設定をバリデーション
    pub fn validate(&self) -> Result<(), GrpcClientError> {
        // タイムアウトの下限チェック
        if self.timeout_ms < MIN_TIMEOUT_MS {
            return Err(GrpcClientError::config(format!(
                "timeout_ms {} is below minimum {}",
                self.timeout_ms, MIN_TIMEOUT_MS
            )));
        }

        // タイムアウトの上限チェック
        if self.timeout_ms > MAX_TIMEOUT_MS {
            return Err(GrpcClientError::config(format!(
                "timeout_ms {} exceeds maximum {}",
                self.timeout_ms, MAX_TIMEOUT_MS
            )));
        }

        // 接続タイムアウトがリクエストタイムアウトを超えていないかチェック
        if self.connect_timeout_ms > self.timeout_ms {
            return Err(GrpcClientError::config(format!(
                "connect_timeout_ms {} exceeds timeout_ms {}",
                self.connect_timeout_ms, self.timeout_ms
            )));
        }

        // リトライ設定のバリデーション
        self.retry.validate()?;

        Ok(())
    }

    /// リクエストタイムアウトを取得
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    /// リクエストタイムアウト（ミリ秒）を取得
    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    /// 接続タイムアウトを取得
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_millis(self.connect_timeout_ms)
    }

    /// 接続タイムアウト（ミリ秒）を取得
    pub fn connect_timeout_ms(&self) -> u64 {
        self.connect_timeout_ms
    }

    /// リトライ設定を取得
    pub fn retry(&self) -> &RetryConfig {
        &self.retry
    }

    /// TLS 設定を取得
    pub fn tls(&self) -> &TlsConfig {
        &self.tls
    }
}

impl Default for GrpcClientConfig {
    fn default() -> Self {
        Self {
            timeout_ms: DEFAULT_TIMEOUT_MS,
            connect_timeout_ms: DEFAULT_CONNECT_TIMEOUT_MS,
            retry: RetryConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

/// gRPC クライアント設定ビルダー
#[derive(Debug, Default)]
pub struct GrpcClientConfigBuilder {
    timeout_ms: Option<u64>,
    connect_timeout_ms: Option<u64>,
    retry: Option<RetryConfig>,
    tls: Option<TlsConfig>,
}

impl GrpcClientConfigBuilder {
    /// 新しいビルダーを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// リクエストタイムアウトを設定（ミリ秒）
    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// リクエストタイムアウトを設定（Duration）
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout_ms = Some(duration.as_millis() as u64);
        self
    }

    /// 接続タイムアウトを設定（ミリ秒）
    pub fn connect_timeout_ms(mut self, ms: u64) -> Self {
        self.connect_timeout_ms = Some(ms);
        self
    }

    /// 接続タイムアウトを設定（Duration）
    pub fn connect_timeout(mut self, duration: Duration) -> Self {
        self.connect_timeout_ms = Some(duration.as_millis() as u64);
        self
    }

    /// リトライ設定を設定
    pub fn retry(mut self, retry: RetryConfig) -> Self {
        self.retry = Some(retry);
        self
    }

    /// TLS 設定を設定
    pub fn tls(mut self, tls: TlsConfig) -> Self {
        self.tls = Some(tls);
        self
    }

    /// 設定をビルド
    ///
    /// バリデーションに失敗した場合はエラーを返す。
    pub fn build(self) -> Result<GrpcClientConfig, GrpcClientError> {
        let config = GrpcClientConfig {
            timeout_ms: self.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS),
            connect_timeout_ms: self.connect_timeout_ms.unwrap_or(DEFAULT_CONNECT_TIMEOUT_MS),
            retry: self.retry.unwrap_or_default(),
            tls: self.tls.unwrap_or_default(),
        };

        config.validate()?;
        Ok(config)
    }
}

/// リトライ設定
///
/// k1s0 では原則としてリトライは無効。
/// 例外的に有効にする場合は ADR での承認が必要。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// 有効/無効（既定: 無効）
    #[serde(default)]
    enabled: bool,
    /// 最大リトライ回数（既定: 0）
    #[serde(default)]
    max_attempts: u32,
    /// 初期バックオフ（ミリ秒）
    #[serde(default = "default_initial_backoff_ms")]
    initial_backoff_ms: u64,
    /// 最大バックオフ（ミリ秒）
    #[serde(default = "default_max_backoff_ms")]
    max_backoff_ms: u64,
    /// バックオフ乗数
    #[serde(default = "default_backoff_multiplier")]
    backoff_multiplier: f64,
    /// ジッター有効
    #[serde(default = "default_jitter")]
    jitter: bool,
    /// リトライ対象ステータスコード
    #[serde(default)]
    retryable_status_codes: Vec<i32>,
    /// ADR 参照（必須: opt-in の場合）
    #[serde(skip_serializing_if = "Option::is_none")]
    adr_reference: Option<String>,
}

fn default_initial_backoff_ms() -> u64 {
    100
}

fn default_max_backoff_ms() -> u64 {
    10_000
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

fn default_jitter() -> bool {
    true
}

impl RetryConfig {
    /// 無効なリトライ設定を作成
    pub fn disabled() -> Self {
        Self::default()
    }

    /// リトライを有効にしたビルダーを作成
    ///
    /// ADR 参照が必須。
    pub fn enabled(adr_reference: impl Into<String>) -> RetryConfigBuilder {
        RetryConfigBuilder::new(adr_reference.into())
    }

    /// 設定をバリデーション
    pub fn validate(&self) -> Result<(), GrpcClientError> {
        // リトライが有効な場合は ADR 参照が必須
        if self.enabled && self.adr_reference.is_none() {
            return Err(GrpcClientError::config(
                "retry enabled but adr_reference is missing (ADR approval required for retry)",
            ));
        }

        // max_attempts のチェック
        if self.enabled && self.max_attempts == 0 {
            return Err(GrpcClientError::config(
                "retry enabled but max_attempts is 0",
            ));
        }

        // バックオフ設定のチェック
        if self.initial_backoff_ms > self.max_backoff_ms {
            return Err(GrpcClientError::config(format!(
                "initial_backoff_ms {} exceeds max_backoff_ms {}",
                self.initial_backoff_ms, self.max_backoff_ms
            )));
        }

        if self.backoff_multiplier < 1.0 {
            return Err(GrpcClientError::config(
                "backoff_multiplier must be >= 1.0",
            ));
        }

        Ok(())
    }

    /// リトライが有効かどうか
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 最大リトライ回数を取得
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// 初期バックオフを取得
    pub fn initial_backoff(&self) -> Duration {
        Duration::from_millis(self.initial_backoff_ms)
    }

    /// 最大バックオフを取得
    pub fn max_backoff(&self) -> Duration {
        Duration::from_millis(self.max_backoff_ms)
    }

    /// バックオフ乗数を取得
    pub fn backoff_multiplier(&self) -> f64 {
        self.backoff_multiplier
    }

    /// ジッターが有効かどうか
    pub fn jitter(&self) -> bool {
        self.jitter
    }

    /// リトライ対象ステータスコードを取得
    pub fn retryable_status_codes(&self) -> &[i32] {
        &self.retryable_status_codes
    }

    /// ADR 参照を取得
    pub fn adr_reference(&self) -> Option<&str> {
        self.adr_reference.as_deref()
    }

    /// 指定したステータスコードがリトライ対象かどうか
    pub fn should_retry(&self, status_code: i32) -> bool {
        if !self.enabled {
            return false;
        }

        // リトライ対象が明示されていない場合は、デフォルトのリトライ可能ステータス
        if self.retryable_status_codes.is_empty() {
            return matches!(status_code, 14 | 8); // UNAVAILABLE, RESOURCE_EXHAUSTED
        }

        self.retryable_status_codes.contains(&status_code)
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_attempts: 0,
            initial_backoff_ms: default_initial_backoff_ms(),
            max_backoff_ms: default_max_backoff_ms(),
            backoff_multiplier: default_backoff_multiplier(),
            jitter: default_jitter(),
            retryable_status_codes: Vec::new(),
            adr_reference: None,
        }
    }
}

/// リトライ設定ビルダー
#[derive(Debug)]
pub struct RetryConfigBuilder {
    adr_reference: String,
    max_attempts: u32,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
    backoff_multiplier: f64,
    jitter: bool,
    retryable_status_codes: Vec<i32>,
}

impl RetryConfigBuilder {
    /// 新しいビルダーを作成
    ///
    /// ADR 参照は必須。
    pub fn new(adr_reference: String) -> Self {
        Self {
            adr_reference,
            max_attempts: 3,
            initial_backoff_ms: default_initial_backoff_ms(),
            max_backoff_ms: default_max_backoff_ms(),
            backoff_multiplier: default_backoff_multiplier(),
            jitter: default_jitter(),
            retryable_status_codes: Vec::new(),
        }
    }

    /// 最大リトライ回数を設定
    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// 初期バックオフを設定（ミリ秒）
    pub fn initial_backoff_ms(mut self, ms: u64) -> Self {
        self.initial_backoff_ms = ms;
        self
    }

    /// 最大バックオフを設定（ミリ秒）
    pub fn max_backoff_ms(mut self, ms: u64) -> Self {
        self.max_backoff_ms = ms;
        self
    }

    /// バックオフ乗数を設定
    pub fn backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// ジッターを設定
    pub fn jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// リトライ対象ステータスコードを設定
    pub fn retryable_status_codes(mut self, codes: Vec<i32>) -> Self {
        self.retryable_status_codes = codes;
        self
    }

    /// リトライ設定をビルド
    pub fn build(self) -> Result<RetryConfig, GrpcClientError> {
        let config = RetryConfig {
            enabled: true,
            max_attempts: self.max_attempts,
            initial_backoff_ms: self.initial_backoff_ms,
            max_backoff_ms: self.max_backoff_ms,
            backoff_multiplier: self.backoff_multiplier,
            jitter: self.jitter,
            retryable_status_codes: self.retryable_status_codes,
            adr_reference: Some(self.adr_reference),
        };

        config.validate()?;
        Ok(config)
    }
}

/// TLS 設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// TLS 有効（既定: false、ローカル開発を想定）
    #[serde(default)]
    enabled: bool,
    /// CA 証明書パス（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    ca_cert_path: Option<String>,
    /// クライアント証明書パス（mTLS 用）
    #[serde(skip_serializing_if = "Option::is_none")]
    client_cert_path: Option<String>,
    /// クライアント秘密鍵パス（mTLS 用）
    #[serde(skip_serializing_if = "Option::is_none")]
    client_key_path: Option<String>,
    /// サーバー名（SNI）
    #[serde(skip_serializing_if = "Option::is_none")]
    server_name: Option<String>,
}

impl TlsConfig {
    /// TLS 無効の設定を作成
    pub fn disabled() -> Self {
        Self::default()
    }

    /// TLS 有効の設定ビルダーを作成
    pub fn enabled() -> TlsConfigBuilder {
        TlsConfigBuilder::new()
    }

    /// TLS が有効かどうか
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// CA 証明書パスを取得
    pub fn ca_cert_path(&self) -> Option<&str> {
        self.ca_cert_path.as_deref()
    }

    /// クライアント証明書パスを取得
    pub fn client_cert_path(&self) -> Option<&str> {
        self.client_cert_path.as_deref()
    }

    /// クライアント秘密鍵パスを取得
    pub fn client_key_path(&self) -> Option<&str> {
        self.client_key_path.as_deref()
    }

    /// サーバー名を取得
    pub fn server_name(&self) -> Option<&str> {
        self.server_name.as_deref()
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            server_name: None,
        }
    }
}

/// TLS 設定ビルダー
#[derive(Debug, Default)]
pub struct TlsConfigBuilder {
    ca_cert_path: Option<String>,
    client_cert_path: Option<String>,
    client_key_path: Option<String>,
    server_name: Option<String>,
}

impl TlsConfigBuilder {
    /// 新しいビルダーを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// CA 証明書パスを設定
    pub fn ca_cert_path(mut self, path: impl Into<String>) -> Self {
        self.ca_cert_path = Some(path.into());
        self
    }

    /// クライアント証明書パスを設定（mTLS 用）
    pub fn client_cert_path(mut self, path: impl Into<String>) -> Self {
        self.client_cert_path = Some(path.into());
        self
    }

    /// クライアント秘密鍵パスを設定（mTLS 用）
    pub fn client_key_path(mut self, path: impl Into<String>) -> Self {
        self.client_key_path = Some(path.into());
        self
    }

    /// サーバー名を設定（SNI）
    pub fn server_name(mut self, name: impl Into<String>) -> Self {
        self.server_name = Some(name.into());
        self
    }

    /// TLS 設定をビルド
    pub fn build(self) -> TlsConfig {
        TlsConfig {
            enabled: true,
            ca_cert_path: self.ca_cert_path,
            client_cert_path: self.client_cert_path,
            client_key_path: self.client_key_path,
            server_name: self.server_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_client_config_default() {
        let config = GrpcClientConfig::default();
        assert_eq!(config.timeout_ms(), DEFAULT_TIMEOUT_MS);
        assert_eq!(config.connect_timeout_ms(), DEFAULT_CONNECT_TIMEOUT_MS);
        assert!(!config.retry().is_enabled());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_grpc_client_config_builder() {
        let config = GrpcClientConfig::builder()
            .timeout_ms(10_000)
            .connect_timeout_ms(2_000)
            .build()
            .unwrap();

        assert_eq!(config.timeout_ms(), 10_000);
        assert_eq!(config.connect_timeout_ms(), 2_000);
    }

    #[test]
    fn test_grpc_client_config_timeout_validation() {
        // 下限未満
        let result = GrpcClientConfig::builder().timeout_ms(50).build();
        assert!(result.is_err());

        // 上限超過
        let result = GrpcClientConfig::builder().timeout_ms(400_000).build();
        assert!(result.is_err());

        // 接続タイムアウトがリクエストタイムアウトを超過
        let result = GrpcClientConfig::builder()
            .timeout_ms(5_000)
            .connect_timeout_ms(10_000)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_retry_config_disabled() {
        let config = RetryConfig::disabled();
        assert!(!config.is_enabled());
        assert_eq!(config.max_attempts(), 0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_retry_config_enabled_requires_adr() {
        // ADR なしでリトライ有効はエラー
        let config = RetryConfig {
            enabled: true,
            max_attempts: 3,
            adr_reference: None,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_retry_config_enabled_with_adr() {
        let config = RetryConfig::enabled("ADR-001")
            .max_attempts(3)
            .initial_backoff_ms(100)
            .max_backoff_ms(5000)
            .build()
            .unwrap();

        assert!(config.is_enabled());
        assert_eq!(config.max_attempts(), 3);
        assert_eq!(config.adr_reference(), Some("ADR-001"));
    }

    #[test]
    fn test_retry_config_should_retry() {
        let config = RetryConfig::enabled("ADR-001")
            .max_attempts(3)
            .build()
            .unwrap();

        // デフォルトでは UNAVAILABLE と RESOURCE_EXHAUSTED がリトライ対象
        assert!(config.should_retry(14)); // UNAVAILABLE
        assert!(config.should_retry(8)); // RESOURCE_EXHAUSTED
        assert!(!config.should_retry(5)); // NOT_FOUND
    }

    #[test]
    fn test_retry_config_custom_retryable_codes() {
        let config = RetryConfig::enabled("ADR-001")
            .max_attempts(3)
            .retryable_status_codes(vec![14])
            .build()
            .unwrap();

        assert!(config.should_retry(14)); // UNAVAILABLE
        assert!(!config.should_retry(8)); // RESOURCE_EXHAUSTED (not in custom list)
    }

    #[test]
    fn test_tls_config_disabled() {
        let config = TlsConfig::disabled();
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_tls_config_enabled() {
        let config = TlsConfig::enabled()
            .ca_cert_path("/etc/ssl/ca.crt")
            .server_name("service.example.com")
            .build();

        assert!(config.is_enabled());
        assert_eq!(config.ca_cert_path(), Some("/etc/ssl/ca.crt"));
        assert_eq!(config.server_name(), Some("service.example.com"));
    }

    #[test]
    fn test_tls_config_mtls() {
        let config = TlsConfig::enabled()
            .ca_cert_path("/etc/ssl/ca.crt")
            .client_cert_path("/etc/ssl/client.crt")
            .client_key_path("/etc/ssl/client.key")
            .build();

        assert!(config.is_enabled());
        assert_eq!(config.client_cert_path(), Some("/etc/ssl/client.crt"));
        assert_eq!(config.client_key_path(), Some("/etc/ssl/client.key"));
    }

    #[test]
    fn test_config_duration() {
        let config = GrpcClientConfig::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(2))
            .build()
            .unwrap();

        assert_eq!(config.timeout(), Duration::from_secs(10));
        assert_eq!(config.connect_timeout(), Duration::from_secs(2));
    }

    #[test]
    fn test_retry_config_backoff_validation() {
        // initial > max
        let result = RetryConfig::enabled("ADR-001")
            .initial_backoff_ms(10_000)
            .max_backoff_ms(5_000)
            .build();
        assert!(result.is_err());

        // multiplier < 1.0
        let result = RetryConfig::enabled("ADR-001")
            .backoff_multiplier(0.5)
            .build();
        assert!(result.is_err());
    }
}
