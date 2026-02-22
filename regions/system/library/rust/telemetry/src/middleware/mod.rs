use std::sync::Arc;

use crate::metrics::Metrics;

#[cfg(any(feature = "axum-layer", test))]
mod http_layer;
#[cfg(any(feature = "grpc-layer", test))]
mod grpc_layer;

#[cfg(any(feature = "axum-layer", test))]
pub use http_layer::MetricsLayer;
#[cfg(any(feature = "grpc-layer", test))]
pub use grpc_layer::GrpcMetricsLayer;

/// TelemetryMiddleware は HTTP リクエストの分散トレーシングとメトリクス記録を提供する。
/// Go の HTTPMiddleware と同等の機能を持つ。
///
/// # axum での使用例
///
/// ```ignore
/// use axum::{Router, middleware};
/// use tower_http::trace::TraceLayer;
/// use std::sync::Arc;
/// use k1s0_telemetry::middleware::TelemetryMiddleware;
/// use k1s0_telemetry::metrics::Metrics;
///
/// let metrics = Arc::new(Metrics::new("order-server"));
/// let mw = TelemetryMiddleware::new(metrics);
///
/// let app = Router::new()
///     .route("/api/v1/orders", post(create_order))
///     .layer(TraceLayer::new_for_http());
/// ```
#[derive(Clone)]
pub struct TelemetryMiddleware {
    pub metrics: Arc<Metrics>,
}

impl TelemetryMiddleware {
    /// new は TelemetryMiddleware を生成する。
    pub fn new(metrics: Arc<Metrics>) -> Self {
        Self { metrics }
    }

    /// on_request はリクエスト開始時にトレーシングスパンを作成する。
    /// axum の middleware::from_fn 等から呼び出す。
    pub fn on_request(&self, method: &str, path: &str) {
        tracing::info_span!("http_request", http.method = method, http.path = path,);
    }

    /// on_response はレスポンス完了時にメトリクスを記録する。
    /// ステータスコードとレイテンシを記録し、構造化ログを出力する。
    pub fn on_response(&self, method: &str, path: &str, status: u16, duration_secs: f64) {
        let status_str = status.to_string();
        self.metrics.record_http_request(method, path, &status_str);
        self.metrics
            .record_http_duration(method, path, duration_secs);

        tracing::info!(
            http.method = method,
            http.path = path,
            http.status_code = status,
            duration_secs = duration_secs,
            "Request completed"
        );
    }
}

/// GrpcInterceptor は gRPC Unary RPC のトレーシングとメトリクス記録を提供する。
/// Go の GRPCUnaryInterceptor と同等の機能を持つ。
///
/// # tonic での使用例
///
/// ```ignore
/// use tonic::transport::Server;
/// use std::sync::Arc;
/// use k1s0_telemetry::middleware::GrpcInterceptor;
/// use k1s0_telemetry::metrics::Metrics;
///
/// let metrics = Arc::new(Metrics::new("order-server"));
/// let interceptor = GrpcInterceptor::new(metrics);
/// ```
#[derive(Clone)]
pub struct GrpcInterceptor {
    pub metrics: Arc<Metrics>,
}

impl GrpcInterceptor {
    /// new は GrpcInterceptor を生成する。
    pub fn new(metrics: Arc<Metrics>) -> Self {
        Self { metrics }
    }

    /// on_request は gRPC リクエスト開始時にトレーシングスパンを作成する。
    pub fn on_request(&self, service: &str, method: &str) {
        tracing::info_span!("grpc_call", rpc.service = service, rpc.method = method,);
    }

    /// on_response は gRPC レスポンス完了時にメトリクスを記録する。
    /// gRPC ステータスコードとレイテンシを記録し、構造化ログを出力する。
    pub fn on_response(&self, service: &str, method: &str, code: &str, duration_secs: f64) {
        self.metrics.record_grpc_request(service, method, code);
        self.metrics
            .record_grpc_duration(service, method, duration_secs);

        if code == "OK" {
            tracing::info!(
                rpc.service = service,
                rpc.method = method,
                rpc.grpc_status_code = code,
                duration_secs = duration_secs,
                "gRPC call completed"
            );
        } else {
            tracing::error!(
                rpc.service = service,
                rpc.method = method,
                rpc.grpc_status_code = code,
                duration_secs = duration_secs,
                "gRPC call failed"
            );
        }
    }
}

/// trace_request マクロは axum/tonic ハンドラにトレーシング情報を付与する。
///
/// # 使用例
///
/// ```ignore
/// use k1s0_telemetry::trace_request;
///
/// let result = trace_request!("GET", "/health", { 42 });
/// ```
#[macro_export]
macro_rules! trace_request {
    ($method:expr, $path:expr, $body:block) => {{
        let span = tracing::info_span!("http_request", http.method = $method, http.path = $path,);
        let _enter = span.enter();
        let start = std::time::Instant::now();
        let result = $body;
        let duration = start.elapsed();
        tracing::info!(
            duration_ms = duration.as_millis() as u64,
            "Request completed"
        );
        result
    }};
}

/// trace_grpc_call マクロは gRPC メソッド呼び出しにトレーシング情報を付与する。
///
/// # 使用例
///
/// ```ignore
/// use k1s0_telemetry::trace_grpc_call;
///
/// let result = trace_grpc_call!("OrderService.CreateOrder", { "ok" });
/// ```
#[macro_export]
macro_rules! trace_grpc_call {
    ($method:expr, $body:block) => {{
        let span = tracing::info_span!("grpc_call", rpc.method = $method,);
        let _enter = span.enter();
        let start = std::time::Instant::now();
        let result = $body;
        let duration = start.elapsed();
        tracing::info!(
            duration_ms = duration.as_millis() as u64,
            "gRPC call completed"
        );
        result
    }};
}
