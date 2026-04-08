pub mod logger;
pub mod metrics;
pub mod middleware;

pub mod error_classifier;

#[cfg(feature = "macros")]
pub use k1s0_telemetry_macros::k1s0_trace;

#[cfg(any(feature = "sqlx-instrument", feature = "kafka-instrument"))]
pub mod instrument;

#[cfg(any(feature = "grpc-layer", test))]
pub use middleware::GrpcMetricsLayer;
#[cfg(any(feature = "axum-layer", test))]
pub use middleware::MetricsLayer;

#[cfg(test)]
mod tests;

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tracing_subscriber::{
    fmt, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

/// `TelemetryConfig` は telemetry ライブラリの初期化設定を保持する。
pub struct TelemetryConfig {
    pub service_name: String,
    pub version: String,
    pub tier: String,
    pub environment: String,
    pub trace_endpoint: Option<String>,
    pub sample_rate: f64,
    pub log_level: String,
    /// ログ出力フォーマット。"text" の場合はプレーンテキスト、それ以外は JSON。
    pub log_format: String,
}

/// `TelemetryConfig` を段階的に構築するビルダー。
/// サービスの startup.rs で繰り返される10行のボイラープレートを1チェーンに圧縮する。
pub struct TelemetryConfigBuilder {
    service_name: String,
    version: String,
    tier: String,
    environment: String,
    trace_endpoint: Option<String>,
    sample_rate: f64,
    log_level: String,
    log_format: String,
}

impl TelemetryConfigBuilder {
    /// 新しいビルダーを作成する（サービス名は必須）
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            // Cargo.toml の package.version からバージョンを取得する（M-16 監査対応: ハードコード解消）
            version: env!("CARGO_PKG_VERSION").to_string(),
            tier: "system".to_string(),
            environment: "dev".to_string(),
            trace_endpoint: None,
            sample_rate: 1.0,
            log_level: "info".to_string(),
            log_format: "text".to_string(),
        }
    }

    /// バージョンを設定する
    #[must_use]
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// ティアを設定する（system / business / service）
    #[must_use]
    pub fn tier(mut self, tier: impl Into<String>) -> Self {
        self.tier = tier.into();
        self
    }

    /// 環境名を設定する
    #[must_use]
    pub fn environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = environment.into();
        self
    }

    /// トレースエンドポイントを有効フラグに基づいて条件付きで設定する
    #[must_use]
    pub fn trace(mut self, enabled: bool, endpoint: impl Into<String>, sample_rate: f64) -> Self {
        self.trace_endpoint = enabled.then(|| endpoint.into());
        self.sample_rate = sample_rate;
        self
    }

    /// ログ設定を一括で設定する
    #[must_use]
    pub fn log(mut self, level: impl Into<String>, format: impl Into<String>) -> Self {
        self.log_level = level.into();
        self.log_format = format.into();
        self
    }

    /// `TelemetryConfig` を構築する
    #[must_use]
    pub fn build(self) -> TelemetryConfig {
        TelemetryConfig {
            service_name: self.service_name,
            version: self.version,
            tier: self.tier,
            environment: self.environment,
            trace_endpoint: self.trace_endpoint,
            sample_rate: self.sample_rate,
            log_level: self.log_level,
            log_format: self.log_format,
        }
    }

    /// `TelemetryConfig` を構築し、同時に `init_telemetry` を呼び出す
    pub fn init(self) -> Result<TelemetryConfig, Box<dyn std::error::Error>> {
        let config = self.build();
        init_telemetry(&config)?;
        Ok(config)
    }
}

impl TelemetryConfig {
    /// ビルダーを作成する便利メソッド
    pub fn builder(service_name: impl Into<String>) -> TelemetryConfigBuilder {
        TelemetryConfigBuilder::new(service_name)
    }
}

/// `init_telemetry` は OpenTelemetry `TracerProvider` と tracing-subscriber を初期化する。
/// `trace_endpoint` が指定されている場合、OTLP gRPC エクスポータを設定する。
pub fn init_telemetry(cfg: &TelemetryConfig) -> Result<(), Box<dyn std::error::Error>> {
    let tracer = if let Some(ref endpoint) = cfg.trace_endpoint {
        let exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()?;
        let provider = sdktrace::TracerProvider::builder()
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .with_sampler(sdktrace::Sampler::TraceIdRatioBased(cfg.sample_rate))
            .with_resource(Resource::new(vec![
                KeyValue::new("service.name", cfg.service_name.clone()),
                KeyValue::new("service.version", cfg.version.clone()),
                KeyValue::new("tier", cfg.tier.clone()),
                KeyValue::new("environment", cfg.environment.clone()),
            ]))
            .build();
        let tracer = provider.tracer("k1s0");
        global::set_tracer_provider(provider);
        Some(tracer)
    } else {
        None
    };

    let filter = EnvFilter::new(&cfg.log_level);
    let registry = tracing_subscriber::registry().with(filter);

    if cfg.log_format == "text" {
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_span_events(FmtSpan::CLOSE);
        let subscriber = registry.with(fmt_layer);
        if let Some(t) = tracer {
            let telemetry_layer = tracing_opentelemetry::layer().with_tracer(t);
            subscriber.with(telemetry_layer).init();
        } else {
            subscriber.init();
        }
    } else {
        let fmt_layer = fmt::layer()
            .json()
            .with_target(true)
            .with_span_events(FmtSpan::CLOSE);
        let subscriber = registry.with(fmt_layer);
        if let Some(t) = tracer {
            let telemetry_layer = tracing_opentelemetry::layer().with_tracer(t);
            subscriber.with(telemetry_layer).init();
        } else {
            subscriber.init();
        }
    }

    Ok(())
}

/// shutdown は OpenTelemetry `TracerProvider` をシャットダウンする。
pub fn shutdown() {
    global::shutdown_tracer_provider();
}
