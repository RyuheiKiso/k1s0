//! トレーシング
//!
//! OpenTelemetry トレーシングの初期化と基本操作を提供する。
//!
//! # 機能
//!
//! - OpenTelemetry SDK の初期化
//! - OTLP エクスポーターによるトレース送信
//! - サンプリング設定
//! - リソース属性の設定
//!
//! # 使用例
//!
//! ```ignore
//! use k1s0_observability::{ObservabilityConfig, tracing::init_tracing};
//!
//! let config = ObservabilityConfig::builder()
//!     .service_name("my-service")
//!     .env("dev")
//!     .otel_endpoint("http://localhost:4317")
//!     .sampling_rate(1.0)
//!     .build()
//!     .unwrap();
//!
//! // トレーシング初期化
//! let _guard = init_tracing(&config).expect("failed to init tracing");
//! ```

use crate::config::ObservabilityConfig;
use crate::context::RequestContext;

/// トレーサー設定
///
/// OTel トレーサーの初期化に必要な設定を保持する。
#[derive(Debug, Clone)]
pub struct TracerConfig {
    /// サービス名（service.name）
    pub service_name: String,
    /// 環境名（deployment.environment）
    pub env: String,
    /// サービスバージョン（service.version）
    pub version: Option<String>,
    /// OTel エンドポイント
    pub endpoint: Option<String>,
    /// サンプリングレート（0.0 - 1.0）
    pub sampling_rate: f64,
}

impl TracerConfig {
    /// ObservabilityConfig から作成
    pub fn from_config(config: &ObservabilityConfig) -> Self {
        Self {
            service_name: config.service_name().to_string(),
            env: config.env().to_string(),
            version: config.version().map(|s| s.to_string()),
            endpoint: config.otel_endpoint().map(|s| s.to_string()),
            sampling_rate: config.sampling_rate(),
        }
    }

    /// OTel リソース属性を取得
    pub fn resource_attributes(&self) -> Vec<(&'static str, String)> {
        let mut attrs = vec![
            ("service.name", self.service_name.clone()),
            ("deployment.environment", self.env.clone()),
        ];

        if let Some(ref version) = self.version {
            attrs.push(("service.version", version.clone()));
        }

        attrs
    }
}

/// スパン種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    /// 内部処理
    Internal,
    /// サーバー（リクエスト受信側）
    Server,
    /// クライアント（リクエスト送信側）
    Client,
    /// プロデューサー（メッセージ送信）
    Producer,
    /// コンシューマー（メッセージ受信）
    Consumer,
}

impl SpanKind {
    /// OTel 標準の文字列表現
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Internal => "INTERNAL",
            Self::Server => "SERVER",
            Self::Client => "CLIENT",
            Self::Producer => "PRODUCER",
            Self::Consumer => "CONSUMER",
        }
    }
}

/// スパン属性
///
/// スパンに付与する属性を構築するビルダー。
#[derive(Debug, Clone, Default)]
pub struct SpanAttributes {
    attributes: Vec<(String, SpanAttributeValue)>,
}

/// スパン属性値
#[derive(Debug, Clone)]
pub enum SpanAttributeValue {
    /// 文字列
    String(String),
    /// 整数
    Int(i64),
    /// 浮動小数点
    Float(f64),
    /// 真偽値
    Bool(bool),
}

impl SpanAttributes {
    /// 新しいスパン属性を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 文字列属性を追加
    pub fn string(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes
            .push((key.into(), SpanAttributeValue::String(value.into())));
        self
    }

    /// 整数属性を追加
    pub fn int(mut self, key: impl Into<String>, value: i64) -> Self {
        self.attributes
            .push((key.into(), SpanAttributeValue::Int(value)));
        self
    }

    /// 浮動小数点属性を追加
    pub fn float(mut self, key: impl Into<String>, value: f64) -> Self {
        self.attributes
            .push((key.into(), SpanAttributeValue::Float(value)));
        self
    }

    /// 真偽値属性を追加
    pub fn bool(mut self, key: impl Into<String>, value: bool) -> Self {
        self.attributes
            .push((key.into(), SpanAttributeValue::Bool(value)));
        self
    }

    /// HTTP リクエスト属性を追加
    pub fn http_request(self, method: &str, path: &str, host: &str) -> Self {
        self.string("http.method", method)
            .string("http.target", path)
            .string("http.host", host)
    }

    /// HTTP レスポンス属性を追加
    pub fn http_response(self, status_code: u16) -> Self {
        self.int("http.status_code", status_code as i64)
    }

    /// gRPC 属性を追加
    pub fn grpc(self, service: &str, method: &str) -> Self {
        self.string("rpc.system", "grpc")
            .string("rpc.service", service)
            .string("rpc.method", method)
    }

    /// 属性のリストを取得
    pub fn into_vec(self) -> Vec<(String, SpanAttributeValue)> {
        self.attributes
    }
}

/// スパンビルダー
///
/// 新しいスパンを作成するためのビルダー。
#[derive(Debug)]
pub struct SpanBuilder {
    name: String,
    kind: SpanKind,
    parent_context: Option<RequestContext>,
    attributes: SpanAttributes,
}

impl SpanBuilder {
    /// 新しいスパンビルダーを作成
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: SpanKind::Internal,
            parent_context: None,
            attributes: SpanAttributes::new(),
        }
    }

    /// スパン種別を設定
    pub fn kind(mut self, kind: SpanKind) -> Self {
        self.kind = kind;
        self
    }

    /// 親コンテキストを設定
    pub fn parent(mut self, ctx: &RequestContext) -> Self {
        self.parent_context = Some(ctx.clone());
        self
    }

    /// 属性を設定
    pub fn attributes(mut self, attrs: SpanAttributes) -> Self {
        self.attributes = attrs;
        self
    }

    /// スパン情報を取得（実際の OTel スパン作成は別途）
    pub fn build(self) -> SpanInfo {
        let context = self
            .parent_context
            .map(|ctx| ctx.child())
            .unwrap_or_else(RequestContext::new);

        SpanInfo {
            name: self.name,
            kind: self.kind,
            context,
            attributes: self.attributes,
        }
    }
}

/// スパン情報
///
/// 作成されたスパンの情報を保持する。
#[derive(Debug)]
pub struct SpanInfo {
    /// スパン名
    pub name: String,
    /// スパン種別
    pub kind: SpanKind,
    /// リクエストコンテキスト
    pub context: RequestContext,
    /// 属性
    pub attributes: SpanAttributes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracer_config_from_config() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .version("1.0.0")
            .sampling_rate(0.5)
            .build()
            .unwrap();

        let tracer_config = TracerConfig::from_config(&config);

        assert_eq!(tracer_config.service_name, "test-service");
        assert_eq!(tracer_config.env, "dev");
        assert_eq!(tracer_config.version, Some("1.0.0".to_string()));
        assert_eq!(tracer_config.sampling_rate, 0.5);
    }

    #[test]
    fn test_resource_attributes() {
        let tracer_config = TracerConfig {
            service_name: "test".to_string(),
            env: "dev".to_string(),
            version: Some("1.0.0".to_string()),
            endpoint: None,
            sampling_rate: 1.0,
        };

        let attrs = tracer_config.resource_attributes();
        assert!(attrs.contains(&("service.name", "test".to_string())));
        assert!(attrs.contains(&("deployment.environment", "dev".to_string())));
        assert!(attrs.contains(&("service.version", "1.0.0".to_string())));
    }

    #[test]
    fn test_span_kind() {
        assert_eq!(SpanKind::Server.as_str(), "SERVER");
        assert_eq!(SpanKind::Client.as_str(), "CLIENT");
    }

    #[test]
    fn test_span_attributes() {
        let attrs = SpanAttributes::new()
            .string("key1", "value1")
            .int("key2", 42)
            .bool("key3", true);

        let vec = attrs.into_vec();
        assert_eq!(vec.len(), 3);
    }

    #[test]
    fn test_span_attributes_http() {
        let attrs = SpanAttributes::new()
            .http_request("GET", "/api/users", "localhost")
            .http_response(200);

        let vec = attrs.into_vec();
        assert!(vec.iter().any(|(k, _)| k == "http.method"));
        assert!(vec.iter().any(|(k, _)| k == "http.status_code"));
    }

    #[test]
    fn test_span_builder() {
        let ctx = RequestContext::new();
        let span = SpanBuilder::new("my-span")
            .kind(SpanKind::Server)
            .parent(&ctx)
            .attributes(SpanAttributes::new().string("foo", "bar"))
            .build();

        assert_eq!(span.name, "my-span");
        assert_eq!(span.kind, SpanKind::Server);
        assert_eq!(span.context.trace_id(), ctx.trace_id());
    }
}

// ============================================================================
// OpenTelemetry OTLP 初期化
// ============================================================================

/// トレーシング初期化ガード
///
/// ドロップ時に OpenTelemetry のシャットダウンを行う。
#[cfg(feature = "otel")]
pub struct TracingGuard {
    _private: (),
}

#[cfg(feature = "otel")]
impl TracingGuard {
    fn new() -> Self {
        Self { _private: () }
    }
}

#[cfg(feature = "otel")]
impl Drop for TracingGuard {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}

/// トレーシングエラー
#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    /// OTLP エクスポーターの設定に失敗
    #[error("failed to configure OTLP exporter: {0}")]
    OtlpConfigFailed(String),

    /// トレーサープロバイダーの設定に失敗
    #[error("failed to configure tracer provider: {0}")]
    ProviderConfigFailed(String),

    /// subscriber の設定に失敗
    #[error("failed to set subscriber: {0}")]
    SubscriberFailed(String),
}

/// OpenTelemetry トレーシングを初期化
///
/// OTLP エクスポーターを使用してトレースを送信する。
///
/// # 引数
///
/// * `config` - ObservabilityConfig
///
/// # 戻り値
///
/// * `Ok(TracingGuard)` - 初期化成功
/// * `Err(TracingError)` - 初期化失敗
///
/// # 機能フラグ
///
/// この関数は `otel` feature が有効な場合のみ利用可能。
///
/// ```toml
/// [dependencies]
/// k1s0-observability = { version = "0.1", features = ["otel"] }
/// ```
#[cfg(feature = "otel")]
pub fn init_tracing(config: &ObservabilityConfig) -> Result<TracingGuard, TracingError> {
    use opentelemetry::trace::TracerProvider;
    use opentelemetry_sdk::{
        trace::{RandomIdGenerator, Sampler, TracerProvider as SdkTracerProvider},
        Resource,
    };
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let tracer_config = TracerConfig::from_config(config);

    // リソース属性の設定
    let resource = create_resource(&tracer_config);

    // サンプラーの設定
    let sampler = create_sampler(tracer_config.sampling_rate);

    // OTLP エクスポーターの設定
    let exporter = create_otlp_exporter(&tracer_config)?;

    // TracerProvider の構築
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(sampler)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource)
        .build();

    let tracer = provider.tracer(tracer_config.service_name.clone());

    // グローバルプロバイダーとして設定
    opentelemetry::global::set_tracer_provider(provider);

    // tracing-opentelemetry レイヤーの設定
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(telemetry_layer)
        .try_init()
        .map_err(|e| TracingError::SubscriberFailed(e.to_string()))?;

    Ok(TracingGuard::new())
}

/// リソースを作成
#[cfg(feature = "otel")]
fn create_resource(config: &TracerConfig) -> opentelemetry_sdk::Resource {
    use opentelemetry::KeyValue;

    let mut attrs = vec![
        KeyValue::new("service.name", config.service_name.clone()),
        KeyValue::new("deployment.environment", config.env.clone()),
    ];

    if let Some(ref version) = config.version {
        attrs.push(KeyValue::new("service.version", version.clone()));
    }

    opentelemetry_sdk::Resource::builder()
        .with_attributes(attrs)
        .build()
}

/// サンプラーを作成
#[cfg(feature = "otel")]
fn create_sampler(sampling_rate: f64) -> opentelemetry_sdk::trace::Sampler {
    use opentelemetry_sdk::trace::Sampler;

    if sampling_rate >= 1.0 {
        Sampler::AlwaysOn
    } else if sampling_rate <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::TraceIdRatioBased(sampling_rate)
    }
}

/// OTLP エクスポーターを作成
#[cfg(feature = "otel")]
fn create_otlp_exporter(
    config: &TracerConfig,
) -> Result<opentelemetry_otlp::SpanExporter, TracingError> {
    use opentelemetry_otlp::WithExportConfig;

    let endpoint = config
        .endpoint
        .clone()
        .unwrap_or_else(|| "http://localhost:4317".to_string());

    opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .map_err(|e| TracingError::OtlpConfigFailed(e.to_string()))
}

/// ロギングと統合したトレーシングを初期化
///
/// tracing-subscriber の JSON レイヤーと OpenTelemetry レイヤーを
/// 両方設定する。
#[cfg(feature = "otel")]
pub fn init_tracing_with_logging(
    config: &ObservabilityConfig,
) -> Result<TracingGuard, TracingError> {
    use opentelemetry::trace::TracerProvider;
    use opentelemetry_sdk::{
        trace::{RandomIdGenerator, TracerProvider as SdkTracerProvider},
        Resource,
    };
    use tracing_subscriber::{
        fmt::{self, format::JsonFields, time::UtcTime},
        layer::SubscriberExt,
        util::SubscriberInitExt,
        EnvFilter,
    };

    let tracer_config = TracerConfig::from_config(config);

    // リソース属性の設定
    let resource = create_resource(&tracer_config);

    // サンプラーの設定
    let sampler = create_sampler(tracer_config.sampling_rate);

    // OTLP エクスポーターの設定
    let exporter = create_otlp_exporter(&tracer_config)?;

    // TracerProvider の構築
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(sampler)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource)
        .build();

    let tracer = provider.tracer(tracer_config.service_name.clone());

    // グローバルプロバイダーとして設定
    opentelemetry::global::set_tracer_provider(provider);

    // tracing-opentelemetry レイヤー
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // JSON ロギングレイヤー
    let json_layer = fmt::layer()
        .json()
        .with_timer(UtcTime::rfc_3339())
        .with_current_span(true)
        .with_span_list(false)
        .with_target(true)
        .flatten_event(true);

    // 環境フィルター
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.log_level()));

    tracing_subscriber::registry()
        .with(filter)
        .with(json_layer)
        .with(telemetry_layer)
        .try_init()
        .map_err(|e| TracingError::SubscriberFailed(e.to_string()))?;

    Ok(TracingGuard::new())
}

/// トレーサー（OTel 統合なし）
///
/// otel feature が無効な場合でも使用可能なスタブ実装。
#[cfg(not(feature = "otel"))]
pub struct TracingGuard {
    _private: (),
}

#[cfg(not(feature = "otel"))]
impl TracingGuard {
    fn new() -> Self {
        Self { _private: () }
    }
}

/// トレーシングを初期化（OTel 統合なし）
///
/// otel feature が無効な場合は、tracing のみ設定する。
#[cfg(not(feature = "otel"))]
pub fn init_tracing(config: &ObservabilityConfig) -> Result<TracingGuard, TracingError> {
    use tracing_subscriber::{
        fmt::{self, time::UtcTime},
        layer::SubscriberExt,
        util::SubscriberInitExt,
        EnvFilter,
    };

    let json_layer = fmt::layer()
        .json()
        .with_timer(UtcTime::rfc_3339())
        .with_current_span(true)
        .with_span_list(false)
        .with_target(true)
        .flatten_event(true);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.log_level()));

    tracing_subscriber::registry()
        .with(filter)
        .with(json_layer)
        .try_init()
        .map_err(|e: tracing_subscriber::util::TryInitError| {
            TracingError::SubscriberFailed(e.to_string())
        })?;

    Ok(TracingGuard::new())
}
