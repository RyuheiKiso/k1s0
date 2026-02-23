use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;

use http::{Request, Response};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::metrics::Metrics;
use crate::middleware::TelemetryMiddleware;

/// MetricsLayer は axum Router に適用する Tower Layer で、
/// HTTP リクエストのメトリクスを自動記録する。
///
/// # 使用例
///
/// ```ignore
/// use k1s0_telemetry::middleware::MetricsLayer;
///
/// let app = Router::new()
///     .route("/healthz", get(healthz))
///     .layer(MetricsLayer::new(metrics.clone()));
/// ```
#[derive(Clone)]
pub struct MetricsLayer {
    mw: TelemetryMiddleware,
}

impl MetricsLayer {
    pub fn new(metrics: Arc<Metrics>) -> Self {
        Self {
            mw: TelemetryMiddleware::new(metrics),
        }
    }
}

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsService {
            inner,
            mw: self.mw.clone(),
        }
    }
}

/// MetricsService は MetricsLayer が生成する Tower Service で、
/// リクエストの method/path を記録し、レスポンス完了時にメトリクスを記録する。
#[derive(Clone)]
pub struct MetricsService<S> {
    inner: S,
    mw: TelemetryMiddleware,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for MetricsService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = MetricsResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let method = req.method().to_string();
        let path = req.uri().path().to_string();

        MetricsResponseFuture {
            inner: self.inner.call(req),
            method,
            path,
            start: Instant::now(),
            mw: self.mw.clone(),
        }
    }
}

pin_project! {
    /// MetricsResponseFuture はレスポンス完了を待ち、メトリクスを記録する Future。
    pub struct MetricsResponseFuture<F> {
        #[pin]
        inner: F,
        method: String,
        path: String,
        start: Instant,
        mw: TelemetryMiddleware,
    }
}

impl<F, ResBody, E> std::future::Future for MetricsResponseFuture<F>
where
    F: std::future::Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Ready(Ok(response)) => {
                let duration = this.start.elapsed().as_secs_f64();
                let status = response.status().as_u16();
                this.mw
                    .on_response(this.method, this.path, status, duration);
                Poll::Ready(Ok(response))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}
