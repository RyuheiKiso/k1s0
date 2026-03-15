use std::task::{Context, Poll};

use http::{Request, Response};
use pin_project_lite::pin_project;
use tower::{Layer, Service};
use uuid::Uuid;

/// RequestIdLayer は各リクエストにユニークな x-request-id を割り当てる Tower Layer。
#[derive(Clone, Default)]
pub struct RequestIdLayer;

impl RequestIdLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdService { inner }
    }
}

#[derive(Clone)]
pub struct RequestIdService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RequestIdService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = RequestIdResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let request_id = Uuid::new_v4().to_string();

        let span = tracing::info_span!("request", request_id = %request_id);
        let _enter = span.enter();

        RequestIdResponseFuture {
            inner: self.inner.call(req),
            request_id,
        }
    }
}

pin_project! {
    pub struct RequestIdResponseFuture<F> {
        #[pin]
        inner: F,
        request_id: String,
    }
}

impl<F, ResBody, E> std::future::Future for RequestIdResponseFuture<F>
where
    F: std::future::Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Ready(Ok(mut response)) => {
                if let Ok(val) = this.request_id.parse() {
                    response.headers_mut().insert("x-request-id", val);
                }
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
    use http::{Request, Response};
    use std::convert::Infallible;
    use tower::{ServiceBuilder, ServiceExt};

    // レスポンスに x-request-id ヘッダーが付与されることを確認する。
    #[tokio::test]
    async fn test_request_id_added_to_response() {
        let svc = ServiceBuilder::new()
            .layer(RequestIdLayer::new())
            .service_fn(|_req: Request<String>| async {
                Ok::<_, Infallible>(Response::new("ok".to_string()))
            });

        let req = Request::builder().body(String::new()).unwrap();
        let resp = svc.oneshot(req).await.unwrap();

        assert!(resp.headers().contains_key("x-request-id"));
        let rid = resp
            .headers()
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(!rid.is_empty());
    }

    // リクエストごとに異なる x-request-id が生成されることを確認する。
    #[tokio::test]
    async fn test_request_id_unique_per_request() {
        let mut svc = ServiceBuilder::new()
            .layer(RequestIdLayer::new())
            .service_fn(|_req: Request<String>| async {
                Ok::<_, Infallible>(Response::new("ok".to_string()))
            });

        let req1 = Request::builder().body(String::new()).unwrap();
        let resp1 = Service::call(&mut svc, req1).await.unwrap();
        let id1 = resp1
            .headers()
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let req2 = Request::builder().body(String::new()).unwrap();
        let resp2 = Service::call(&mut svc, req2).await.unwrap();
        let id2 = resp2
            .headers()
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        assert_ne!(id1, id2);
    }
}
