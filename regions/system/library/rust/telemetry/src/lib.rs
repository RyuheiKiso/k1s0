pub mod logger;
pub mod metrics;
pub mod middleware;

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

/// TelemetryConfig は telemetry ライブラリの初期化設定を保持する。
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

/// init_telemetry は OpenTelemetry TracerProvider と tracing-subscriber を初期化する。
/// trace_endpoint が指定されている場合、OTLP gRPC エクスポータを設定する。
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

/// shutdown は OpenTelemetry TracerProvider をシャットダウンする。
pub fn shutdown() {
    global::shutdown_tracer_provider();
}
