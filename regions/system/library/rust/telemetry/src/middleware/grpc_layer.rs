use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;

use http::{Request, Response};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::metrics::Metrics;
use crate::middleware::GrpcInterceptor;

/// GrpcMetricsLayer は tonic::transport::Server に適用する Tower Layer で、
/// gRPC リクエストのメトリクスを自動記録する。
///
/// # 使用例
///
/// ```ignore
/// use k1s0_telemetry::middleware::GrpcMetricsLayer;
///
/// tonic::transport::Server::builder()
///     .layer(GrpcMetricsLayer::new(metrics.clone()))
///     .add_service(my_service)
///     .serve(addr)
///     .await?;
/// ```
#[derive(Clone)]
pub struct GrpcMetricsLayer {
    interceptor: GrpcInterceptor,
}

impl GrpcMetricsLayer {
    pub fn new(metrics: Arc<Metrics>) -> Self {
        Self {
            interceptor: GrpcInterceptor::new(metrics),
        }
    }
}

impl<S> Layer<S> for GrpcMetricsLayer {
    type Service = GrpcMetricsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        GrpcMetricsService {
            inner,
            interceptor: self.interceptor.clone(),
        }
    }
}

/// GrpcMetricsService は GrpcMetricsLayer が生成する Tower Service。
/// URI パスから `/package.Service/Method` を抽出し、レスポンスの grpc-status ヘッダから
/// ステータスコードを取得してメトリクスを記録する。
#[derive(Clone)]
pub struct GrpcMetricsService<S> {
    inner: S,
    interceptor: GrpcInterceptor,
}

/// URI パスから gRPC service 名と method 名を抽出する。
/// 入力例: `/package.ServiceName/MethodName`
/// 戻り値: `("package.ServiceName", "MethodName")`
fn extract_grpc_service_method(path: &str) -> (&str, &str) {
    let trimmed = path.strip_prefix('/').unwrap_or(path);
    match trimmed.rsplit_once('/') {
        Some((service, method)) => (service, method),
        None => ("unknown", trimmed),
    }
}

/// gRPC レスポンスヘッダから grpc-status を取得してステータスコード文字列に変換する。
fn grpc_status_code<B>(response: &Response<B>) -> &'static str {
    let status = response
        .headers()
        .get("grpc-status")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0); // grpc-status が無い場合は OK (0) とみなす

    match status {
        0 => "OK",
        1 => "CANCELLED",
        2 => "UNKNOWN",
        3 => "INVALID_ARGUMENT",
        4 => "DEADLINE_EXCEEDED",
        5 => "NOT_FOUND",
        6 => "ALREADY_EXISTS",
        7 => "PERMISSION_DENIED",
        8 => "RESOURCE_EXHAUSTED",
        9 => "FAILED_PRECONDITION",
        10 => "ABORTED",
        11 => "OUT_OF_RANGE",
        12 => "UNIMPLEMENTED",
        13 => "INTERNAL",
        14 => "UNAVAILABLE",
        15 => "DATA_LOSS",
        16 => "UNAUTHENTICATED",
        _ => "UNKNOWN",
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for GrpcMetricsService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = GrpcMetricsResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let path = req.uri().path().to_string();
        let (service, method) = extract_grpc_service_method(&path);

        GrpcMetricsResponseFuture {
            inner: self.inner.call(req),
            service: service.to_string(),
            method: method.to_string(),
            start: Instant::now(),
            interceptor: self.interceptor.clone(),
        }
    }
}

pin_project! {
    /// GrpcMetricsResponseFuture はレスポンス完了を待ち、gRPC メトリクスを記録する Future。
    pub struct GrpcMetricsResponseFuture<F> {
        #[pin]
        inner: F,
        service: String,
        method: String,
        start: Instant,
        interceptor: GrpcInterceptor,
    }
}

impl<F, ResBody, E> std::future::Future for GrpcMetricsResponseFuture<F>
where
    F: std::future::Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Ready(Ok(response)) => {
                let duration = this.start.elapsed().as_secs_f64();
                let code = grpc_status_code(&response);
                this.interceptor
                    .on_response(this.service, this.method, code, duration);
                Poll::Ready(Ok(response))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_grpc_service_method() {
        let (svc, method) = extract_grpc_service_method("/k1s0.system.auth.v1.AuthService/ValidateToken");
        assert_eq!(svc, "k1s0.system.auth.v1.AuthService");
        assert_eq!(method, "ValidateToken");
    }

    #[test]
    fn test_extract_grpc_service_method_no_slash() {
        let (svc, method) = extract_grpc_service_method("NoSlash");
        assert_eq!(svc, "unknown");
        assert_eq!(method, "NoSlash");
    }

    #[test]
    fn test_extract_grpc_service_method_leading_slash_only() {
        let (svc, method) = extract_grpc_service_method("/Service/Method");
        assert_eq!(svc, "Service");
        assert_eq!(method, "Method");
    }
}
