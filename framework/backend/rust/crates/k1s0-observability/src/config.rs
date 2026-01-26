//! 観測性の設定
//!
//! サービス名、環境名等の必須設定を管理する。

use thiserror::Error;

use crate::context::RequestContext;

/// 観測性設定エラー
#[derive(Debug, Error)]
pub enum ConfigError {
    /// サービス名が未設定
    #[error("service_name は必須です")]
    MissingServiceName,

    /// 環境名が未設定
    #[error("env は必須です")]
    MissingEnv,

    /// 環境名が不正
    #[error("env は dev, stg, prod のいずれかである必要があります: {0}")]
    InvalidEnv(String),
}

/// 観測性設定
///
/// サービスの観測性に必要な設定を保持する。
/// `service_name` と `env` は必須。
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// サービス名
    service_name: String,
    /// 環境名（dev, stg, prod）
    env: String,
    /// サービスバージョン（オプション）
    version: Option<String>,
    /// サービスインスタンス ID（オプション）
    instance_id: Option<String>,
    /// OTel エンドポイント（オプション）
    otel_endpoint: Option<String>,
    /// ログレベル
    log_level: String,
    /// サンプリングレート（0.0 - 1.0）
    sampling_rate: f64,
}

impl ObservabilityConfig {
    /// ビルダーを作成
    pub fn builder() -> ObservabilityBuilder {
        ObservabilityBuilder::new()
    }

    /// サービス名を取得
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// 環境名を取得
    pub fn env(&self) -> &str {
        &self.env
    }

    /// バージョンを取得
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// インスタンス ID を取得
    pub fn instance_id(&self) -> Option<&str> {
        self.instance_id.as_deref()
    }

    /// OTel エンドポイントを取得
    pub fn otel_endpoint(&self) -> Option<&str> {
        self.otel_endpoint.as_deref()
    }

    /// ログレベルを取得
    pub fn log_level(&self) -> &str {
        &self.log_level
    }

    /// サンプリングレートを取得
    pub fn sampling_rate(&self) -> f64 {
        self.sampling_rate
    }

    /// 新しいリクエストコンテキストを作成
    pub fn new_request_context(&self) -> RequestContext {
        RequestContext::new()
    }

    /// 指定したトレース ID でリクエストコンテキストを作成
    pub fn new_request_context_with_trace(&self, trace_id: impl Into<String>) -> RequestContext {
        RequestContext::with_trace_id(trace_id)
    }

    /// 本番環境かどうか
    pub fn is_production(&self) -> bool {
        self.env == "prod"
    }
}

/// 観測性設定ビルダー
///
/// 必須フィールド欠落を起こさない初期化 API を提供。
#[derive(Debug, Default)]
pub struct ObservabilityBuilder {
    service_name: Option<String>,
    env: Option<String>,
    version: Option<String>,
    instance_id: Option<String>,
    otel_endpoint: Option<String>,
    log_level: Option<String>,
    sampling_rate: Option<f64>,
}

impl ObservabilityBuilder {
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

    /// バージョンを設定
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// インスタンス ID を設定
    pub fn instance_id(mut self, id: impl Into<String>) -> Self {
        self.instance_id = Some(id.into());
        self
    }

    /// OTel エンドポイントを設定
    pub fn otel_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.otel_endpoint = Some(endpoint.into());
        self
    }

    /// ログレベルを設定
    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.log_level = Some(level.into());
        self
    }

    /// サンプリングレートを設定（0.0 - 1.0）
    pub fn sampling_rate(mut self, rate: f64) -> Self {
        self.sampling_rate = Some(rate.clamp(0.0, 1.0));
        self
    }

    /// 設定をビルド
    ///
    /// 必須フィールドが不足している場合はエラーを返す。
    pub fn build(self) -> Result<ObservabilityConfig, ConfigError> {
        let service_name = self.service_name.ok_or(ConfigError::MissingServiceName)?;
        let env = self.env.ok_or(ConfigError::MissingEnv)?;

        // 環境名の検証
        if !matches!(env.as_str(), "dev" | "stg" | "prod") {
            return Err(ConfigError::InvalidEnv(env));
        }

        Ok(ObservabilityConfig {
            service_name,
            env,
            version: self.version,
            instance_id: self.instance_id,
            otel_endpoint: self.otel_endpoint,
            log_level: self.log_level.unwrap_or_else(|| "INFO".to_string()),
            sampling_rate: self.sampling_rate.unwrap_or(1.0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_success() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .version("1.0.0")
            .build()
            .unwrap();

        assert_eq!(config.service_name(), "test-service");
        assert_eq!(config.env(), "dev");
        assert_eq!(config.version(), Some("1.0.0"));
    }

    #[test]
    fn test_builder_missing_service_name() {
        let result = ObservabilityConfig::builder()
            .env("dev")
            .build();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::MissingServiceName));
    }

    #[test]
    fn test_builder_missing_env() {
        let result = ObservabilityConfig::builder()
            .service_name("test")
            .build();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::MissingEnv));
    }

    #[test]
    fn test_builder_invalid_env() {
        let result = ObservabilityConfig::builder()
            .service_name("test")
            .env("invalid")
            .build();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::InvalidEnv(_)));
    }

    #[test]
    fn test_default_values() {
        let config = ObservabilityConfig::builder()
            .service_name("test")
            .env("dev")
            .build()
            .unwrap();

        assert_eq!(config.log_level(), "INFO");
        assert_eq!(config.sampling_rate(), 1.0);
    }

    #[test]
    fn test_sampling_rate_clamped() {
        let config = ObservabilityConfig::builder()
            .service_name("test")
            .env("dev")
            .sampling_rate(1.5)
            .build()
            .unwrap();

        assert_eq!(config.sampling_rate(), 1.0);
    }

    #[test]
    fn test_is_production() {
        let dev = ObservabilityConfig::builder()
            .service_name("test")
            .env("dev")
            .build()
            .unwrap();
        assert!(!dev.is_production());

        let prod = ObservabilityConfig::builder()
            .service_name("test")
            .env("prod")
            .build()
            .unwrap();
        assert!(prod.is_production());
    }

    #[test]
    fn test_new_request_context() {
        let config = ObservabilityConfig::builder()
            .service_name("test")
            .env("dev")
            .build()
            .unwrap();

        let ctx = config.new_request_context();
        assert!(!ctx.trace_id().is_empty());
        assert!(!ctx.request_id().is_empty());
    }
}
