use std::task::{Context, Poll};

use http::{Request, Response};
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::context::{CorrelationContext, CorrelationHeaders};
use crate::id::{CorrelationId, TraceId};

/// CorrelationLayer は Tower Layer として、リクエストに相関IDを注入・伝播する。
///
/// # 使用例
///
/// ```ignore
/// use k1s0_correlation::layer::CorrelationLayer;
///
/// let app = Router::new()
///     .route("/healthz", get(healthz))
///     .layer(CorrelationLayer::new());
/// ```
#[derive(Clone, Default)]
pub struct CorrelationLayer;

impl CorrelationLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for CorrelationLayer {
    type Service = CorrelationService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CorrelationService { inner }
    }
}

/// CorrelationService は CorrelationLayer が生成する Tower Service で、
/// リクエストから相関ID・トレースIDを抽出し、レスポンスヘッダーに設定する。
#[derive(Clone)]
pub struct CorrelationService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CorrelationService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = CorrelationResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // x-correlation-id ヘッダーを読取（なければ UUID v4 生成）
        let correlation_id = req
            .headers()
            .get(CorrelationHeaders::CORRELATION_ID)
            .and_then(|v| v.to_str().ok())
            .map(CorrelationId::from_string)
            .unwrap_or_default();

        // x-trace-id ヘッダーを読取（あれば伝播、なければNone）
        let trace_id = req
            .headers()
            .get(CorrelationHeaders::TRACE_ID)
            .and_then(|v| v.to_str().ok())
            .and_then(TraceId::from_string);

        let mut ctx = CorrelationContext::from_correlation_id(correlation_id);
        if let Some(tid) = trace_id {
            ctx = ctx.with_trace_id(tid);
        }

        // CorrelationContext を req.extensions() に挿入
        req.extensions_mut().insert(ctx.clone());

        // tracing span に correlation_id を追加
        let span = tracing::info_span!(
            "request",
            correlation_id = %ctx.correlation_id,
        );
        let _enter = span.enter();

        CorrelationResponseFuture {
            inner: self.inner.call(req),
            ctx,
        }
    }
}

pin_project! {
    /// CorrelationResponseFuture はレスポンス完了を待ち、相関ヘッダーを設定する Future。
    pub struct CorrelationResponseFuture<F> {
        #[pin]
        inner: F,
        ctx: CorrelationContext,
    }
}

impl<F, ResBody, E> std::future::Future for CorrelationResponseFuture<F>
where
    F: std::future::Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Ready(Ok(mut response)) => {
                // レスポンスに x-correlation-id ヘッダーを設定
                if let Ok(val) = this.ctx.correlation_id.as_str().parse() {
                    response
                        .headers_mut()
                        .insert(CorrelationHeaders::CORRELATION_ID, val);
                }
                // trace_id があればレスポンスにも設定
                if let Some(ref trace_id) = this.ctx.trace_id {
                    if let Ok(val) = trace_id.as_str().parse() {
                        response
                            .headers_mut()
                            .insert(CorrelationHeaders::TRACE_ID, val);
                    }
                }
                Poll::Ready(Ok(response))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use http::{Request, Response, StatusCode};
    use std::convert::Infallible;
    use tower::{ServiceBuilder, ServiceExt};

    /// テスト用の echo サービス - extensions から CorrelationContext を取得してレスポンスに含める
    async fn echo_service(req: Request<String>) -> Result<Response<String>, Infallible> {
        let ctx = req.extensions().get::<CorrelationContext>().cloned();
        let body = match ctx {
            Some(c) => format!("correlation_id={}", c.correlation_id),
            None => "no-context".to_string(),
        };
        Ok(Response::new(body))
    }

    #[tokio::test]
    async fn test_no_correlation_header_generates_id() {
        let svc = ServiceBuilder::new()
            .layer(CorrelationLayer::new())
            .service_fn(echo_service);

        let req = Request::builder().body(String::new()).unwrap();
        let resp = svc.oneshot(req).await.unwrap();

        // レスポンスに x-correlation-id ヘッダーが設定されている
        assert!(resp.headers().contains_key("x-correlation-id"));
        let cid = resp
            .headers()
            .get("x-correlation-id")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(!cid.is_empty());
    }

    #[tokio::test]
    async fn test_existing_correlation_header_preserved() {
        let svc = ServiceBuilder::new()
            .layer(CorrelationLayer::new())
            .service_fn(echo_service);

        let req = Request::builder()
            .header("x-correlation-id", "my-corr-id")
            .body(String::new())
            .unwrap();
        let resp = svc.oneshot(req).await.unwrap();

        let cid = resp
            .headers()
            .get("x-correlation-id")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(cid, "my-corr-id");
        // body にも correlation_id が含まれる（extensions経由で渡された）
        let body = resp.into_body();
        assert!(body.contains("my-corr-id"));
    }

    #[tokio::test]
    async fn test_trace_id_propagated() {
        let svc = ServiceBuilder::new()
            .layer(CorrelationLayer::new())
            .service_fn(echo_service);

        let trace = "4bf92f3577b34da6a3ce929d0e0e4736";
        let req = Request::builder()
            .header("x-trace-id", trace)
            .body(String::new())
            .unwrap();
        let resp = svc.oneshot(req).await.unwrap();

        let tid = resp.headers().get("x-trace-id").unwrap().to_str().unwrap();
        assert_eq!(tid, trace);
    }

    #[tokio::test]
    async fn test_invalid_trace_id_ignored() {
        let svc = ServiceBuilder::new()
            .layer(CorrelationLayer::new())
            .service_fn(echo_service);

        let req = Request::builder()
            .header("x-trace-id", "invalid")
            .body(String::new())
            .unwrap();
        let resp = svc.oneshot(req).await.unwrap();

        // 不正な trace-id はレスポンスヘッダーに含まれない
        assert!(!resp.headers().contains_key("x-trace-id"));
    }

    #[tokio::test]
    async fn test_context_inserted_into_extensions() {
        let svc = ServiceBuilder::new()
            .layer(CorrelationLayer::new())
            .service_fn(|req: Request<String>| async move {
                let ctx = req.extensions().get::<CorrelationContext>().unwrap();
                assert!(!ctx.correlation_id.as_str().is_empty());
                Ok::<_, Infallible>(Response::new("ok".to_string()))
            });

        let req = Request::builder().body(String::new()).unwrap();
        let resp = svc.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
