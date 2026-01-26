//! HTTP ミドルウェア
//!
//! HTTP リクエストの観測性を提供する。
//!
//! # 機能
//!
//! - リクエスト/レスポンスのログ出力
//! - メトリクス収集
//! - トレースコンテキストの伝播
//! - axum Tower Layer としての統合
//!
//! # 使用例（axum）
//!
//! ```ignore
//! use axum::Router;
//! use k1s0_observability::middleware::http::ObservabilityLayer;
//!
//! let config = ObservabilityConfig::builder()
//!     .service_name("my-service")
//!     .env("dev")
//!     .build()
//!     .unwrap();
//!
//! let app = Router::new()
//!     .route("/", get(handler))
//!     .layer(ObservabilityLayer::from_config(&config));
//! ```

use crate::config::ObservabilityConfig;
use crate::context::RequestContext;
use crate::log_fields::{LogEntry, LogLevel};
use crate::logging::RequestLog;
use crate::metrics::MetricLabels;

/// HTTP リクエスト情報
#[derive(Debug, Clone)]
pub struct HttpRequestInfo {
    /// HTTP メソッド
    pub method: String,
    /// リクエストパス
    pub path: String,
    /// ホスト
    pub host: Option<String>,
    /// User-Agent
    pub user_agent: Option<String>,
    /// リクエストサイズ（バイト）
    pub content_length: Option<u64>,
}

impl HttpRequestInfo {
    /// 新しいリクエスト情報を作成
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            host: None,
            user_agent: None,
            content_length: None,
        }
    }

    /// ホストを設定
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// User-Agent を設定
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Content-Length を設定
    pub fn with_content_length(mut self, length: u64) -> Self {
        self.content_length = Some(length);
        self
    }
}

/// HTTP レスポンス情報
#[derive(Debug, Clone)]
pub struct HttpResponseInfo {
    /// ステータスコード
    pub status_code: u16,
    /// レスポンスサイズ（バイト）
    pub content_length: Option<u64>,
    /// エラーの種類（エラーの場合）
    pub error_kind: Option<String>,
    /// エラーコード（エラーの場合）
    pub error_code: Option<String>,
}

impl HttpResponseInfo {
    /// 新しいレスポンス情報を作成
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            content_length: None,
            error_kind: None,
            error_code: None,
        }
    }

    /// 成功かどうか
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 400
    }

    /// クライアントエラーかどうか
    pub fn is_client_error(&self) -> bool {
        self.status_code >= 400 && self.status_code < 500
    }

    /// サーバーエラーかどうか
    pub fn is_server_error(&self) -> bool {
        self.status_code >= 500
    }

    /// Content-Length を設定
    pub fn with_content_length(mut self, length: u64) -> Self {
        self.content_length = Some(length);
        self
    }

    /// エラー情報を設定
    pub fn with_error(mut self, kind: impl Into<String>, code: impl Into<String>) -> Self {
        self.error_kind = Some(kind.into());
        self.error_code = Some(code.into());
        self
    }
}

/// HTTP 観測性
///
/// HTTP リクエストのログ、メトリクス、トレースを統合する。
#[derive(Debug, Clone)]
pub struct HttpObservability {
    service_name: String,
    service_env: String,
}

impl HttpObservability {
    /// ObservabilityConfig から作成
    pub fn from_config(config: &ObservabilityConfig) -> Self {
        Self {
            service_name: config.service_name().to_string(),
            service_env: config.env().to_string(),
        }
    }

    /// リクエストコンテキストを作成または取得
    ///
    /// traceparent ヘッダがあれば引き継ぎ、なければ新規作成。
    pub fn extract_or_create_context(&self, traceparent: Option<&str>) -> RequestContext {
        traceparent
            .and_then(RequestContext::from_traceparent)
            .unwrap_or_else(RequestContext::new)
    }

    /// リクエスト完了時の観測性出力を生成
    pub fn on_request_complete(
        &self,
        ctx: &RequestContext,
        request: &HttpRequestInfo,
        response: &HttpResponseInfo,
        latency_ms: f64,
    ) -> HttpObservabilityOutput {
        // ログレベルの決定
        let log_level = if response.is_server_error() {
            LogLevel::Error
        } else if response.is_client_error() {
            LogLevel::Warn
        } else {
            LogLevel::Info
        };

        // ログメッセージ
        let message = format!(
            "{} {} {} {:.2}ms",
            request.method, request.path, response.status_code, latency_ms
        );

        // ログエントリ
        let mut entry = LogEntry::new(log_level, &message);
        entry.service_name = Some(self.service_name.clone());
        entry.service_env = Some(self.service_env.clone());
        entry.trace_id = Some(ctx.trace_id().to_string());
        entry.span_id = Some(ctx.span_id().to_string());
        entry.request_id = Some(ctx.request_id().to_string());

        let log = RequestLog::new(entry)
            .with_http(&request.method, &request.path, response.status_code)
            .with_latency(latency_ms);

        // メトリクスラベル
        let labels = MetricLabels::new()
            .service(&self.service_name)
            .env(&self.service_env)
            .http(&request.method, &request.path, response.status_code);

        HttpObservabilityOutput {
            log,
            labels,
            log_level,
            latency_ms,
            success: response.is_success(),
            request: request.clone(),
            response: response.clone(),
        }
    }
}

/// HTTP 観測性出力
#[derive(Debug)]
pub struct HttpObservabilityOutput {
    /// ログ
    pub log: RequestLog,
    /// メトリクスラベル
    pub labels: MetricLabels,
    /// ログレベル
    pub log_level: LogLevel,
    /// レイテンシ
    pub latency_ms: f64,
    /// 成功かどうか
    pub success: bool,
    /// リクエスト情報
    pub request: HttpRequestInfo,
    /// レスポンス情報
    pub response: HttpResponseInfo,
}

impl HttpObservabilityOutput {
    /// JSON ログを出力
    pub fn log_json(&self) -> Result<String, serde_json::Error> {
        self.log.to_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_info() {
        let info = HttpRequestInfo::new("GET", "/api/users")
            .with_host("localhost")
            .with_content_length(100);

        assert_eq!(info.method, "GET");
        assert_eq!(info.path, "/api/users");
        assert_eq!(info.host, Some("localhost".to_string()));
        assert_eq!(info.content_length, Some(100));
    }

    #[test]
    fn test_http_response_info() {
        let success = HttpResponseInfo::new(200);
        assert!(success.is_success());
        assert!(!success.is_client_error());
        assert!(!success.is_server_error());

        let client_error = HttpResponseInfo::new(404);
        assert!(!client_error.is_success());
        assert!(client_error.is_client_error());

        let server_error = HttpResponseInfo::new(500);
        assert!(server_error.is_server_error());
    }

    #[test]
    fn test_http_observability() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let obs = HttpObservability::from_config(&config);
        let ctx = RequestContext::new();
        let request = HttpRequestInfo::new("GET", "/api/users");
        let response = HttpResponseInfo::new(200);

        let output = obs.on_request_complete(&ctx, &request, &response, 42.5);

        assert!(output.success);
        assert_eq!(output.log_level, LogLevel::Info);
        assert_eq!(output.latency_ms, 42.5);
    }

    #[test]
    fn test_http_observability_error() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let obs = HttpObservability::from_config(&config);
        let ctx = RequestContext::new();
        let request = HttpRequestInfo::new("GET", "/api/users/123");
        let response = HttpResponseInfo::new(500)
            .with_error("INTERNAL", "INTERNAL_ERROR");

        let output = obs.on_request_complete(&ctx, &request, &response, 100.0);

        assert!(!output.success);
        assert_eq!(output.log_level, LogLevel::Error);
    }

    #[test]
    fn test_extract_or_create_context() {
        let config = ObservabilityConfig::builder()
            .service_name("test")
            .env("dev")
            .build()
            .unwrap();

        let obs = HttpObservability::from_config(&config);

        // traceparent なし
        let ctx1 = obs.extract_or_create_context(None);
        assert!(!ctx1.trace_id().is_empty());

        // traceparent あり
        let traceparent = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let ctx2 = obs.extract_or_create_context(Some(traceparent));
        assert_eq!(ctx2.trace_id(), "0af7651916cd43dd8448eb211c80319c");
    }

    #[test]
    fn test_log_json() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let obs = HttpObservability::from_config(&config);
        let ctx = RequestContext::new();
        let request = HttpRequestInfo::new("GET", "/api/users");
        let response = HttpResponseInfo::new(200);

        let output = obs.on_request_complete(&ctx, &request, &response, 42.5);
        let json = output.log_json().unwrap();

        assert!(json.contains("test-service"));
        assert!(json.contains("GET"));
        assert!(json.contains("/api/users"));
        assert!(json.contains("200"));
    }
}

// ============================================================================
// axum Tower Layer 実装
// ============================================================================

#[cfg(feature = "axum-layer")]
pub use axum_layer::*;

#[cfg(feature = "axum-layer")]
mod axum_layer {
    use super::*;
    use axum::{
        body::Body,
        extract::Request,
        response::Response,
    };
    use futures::future::BoxFuture;
    use http::header::HeaderValue;
    use pin_project_lite::pin_project;
    use std::{
        future::Future,
        pin::Pin,
        sync::Arc,
        task::{Context, Poll},
        time::Instant,
    };
    use tower::{Layer, Service};

    /// 観測性レイヤー
    ///
    /// axum の Router に追加して、全リクエストの観測性を自動化する。
    ///
    /// # 例
    ///
    /// ```ignore
    /// use axum::Router;
    /// use k1s0_observability::middleware::http::ObservabilityLayer;
    ///
    /// let config = ObservabilityConfig::builder()
    ///     .service_name("my-service")
    ///     .env("dev")
    ///     .build()
    ///     .unwrap();
    ///
    /// let app = Router::new()
    ///     .route("/", get(handler))
    ///     .layer(ObservabilityLayer::from_config(&config));
    /// ```
    #[derive(Clone)]
    pub struct ObservabilityLayer {
        inner: Arc<ObservabilityLayerInner>,
    }

    struct ObservabilityLayerInner {
        observability: HttpObservability,
        skip_paths: Vec<String>,
    }

    impl ObservabilityLayer {
        /// 新しいレイヤーを作成
        pub fn new(service_name: impl Into<String>, service_env: impl Into<String>) -> Self {
            Self {
                inner: Arc::new(ObservabilityLayerInner {
                    observability: HttpObservability {
                        service_name: service_name.into(),
                        service_env: service_env.into(),
                    },
                    skip_paths: Vec::new(),
                }),
            }
        }

        /// ObservabilityConfig から作成
        pub fn from_config(config: &ObservabilityConfig) -> Self {
            Self::new(config.service_name(), config.env())
        }

        /// スキップするパスを設定
        ///
        /// ヘルスチェックやメトリクスエンドポイントなど、
        /// ログに記録したくないパスを指定する。
        pub fn skip_paths(mut self, paths: Vec<String>) -> Self {
            if let Some(inner) = Arc::get_mut(&mut self.inner) {
                inner.skip_paths = paths;
            }
            self
        }
    }

    impl<S> Layer<S> for ObservabilityLayer {
        type Service = ObservabilityMiddleware<S>;

        fn layer(&self, inner: S) -> Self::Service {
            ObservabilityMiddleware {
                inner,
                layer: self.clone(),
            }
        }
    }

    /// 観測性ミドルウェア
    #[derive(Clone)]
    pub struct ObservabilityMiddleware<S> {
        inner: S,
        layer: ObservabilityLayer,
    }

    impl<S> Service<Request> for ObservabilityMiddleware<S>
    where
        S: Service<Request, Response = Response> + Clone + Send + 'static,
        S::Future: Send + 'static,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = ObservabilityFuture<S::Future>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, request: Request) -> Self::Future {
            let start = Instant::now();
            let method = request.method().to_string();
            let path = request.uri().path().to_string();

            // スキップパスのチェック
            let skip = self.layer.inner.skip_paths.iter().any(|p| path.starts_with(p));

            // traceparent ヘッダの取得
            let traceparent = request
                .headers()
                .get("traceparent")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            // リクエストコンテキストの作成
            let ctx = self
                .layer
                .inner
                .observability
                .extract_or_create_context(traceparent.as_deref());

            let layer = self.layer.clone();
            let future = self.inner.call(request);

            ObservabilityFuture {
                inner: future,
                state: Some(FutureState {
                    layer,
                    ctx,
                    start,
                    method,
                    path,
                    skip,
                }),
            }
        }
    }

    struct FutureState {
        layer: ObservabilityLayer,
        ctx: RequestContext,
        start: Instant,
        method: String,
        path: String,
        skip: bool,
    }

    pin_project! {
        /// 観測性フューチャー
        pub struct ObservabilityFuture<F> {
            #[pin]
            inner: F,
            state: Option<FutureState>,
        }
    }

    impl<F, E> Future for ObservabilityFuture<F>
    where
        F: Future<Output = Result<Response, E>>,
    {
        type Output = F::Output;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.project();

            match this.inner.poll(cx) {
                Poll::Ready(result) => {
                    if let Some(state) = this.state.take() {
                        if !state.skip {
                            let elapsed = state.start.elapsed();
                            let latency_ms = elapsed.as_secs_f64() * 1000.0;

                            let status_code = result
                                .as_ref()
                                .map(|r| r.status().as_u16())
                                .unwrap_or(500);

                            let request = HttpRequestInfo::new(&state.method, &state.path);
                            let response = HttpResponseInfo::new(status_code);

                            let output = state.layer.inner.observability.on_request_complete(
                                &state.ctx,
                                &request,
                                &response,
                                latency_ms,
                            );

                            // JSON ログを出力
                            if let Ok(json) = output.log_json() {
                                tracing::info!(target: "http_access", "{}", json);
                            }
                        }
                    }

                    Poll::Ready(result)
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }

    /// リクエストコンテキスト抽出エクステンション
    ///
    /// axum のリクエストエクステンションから RequestContext を取得する。
    pub fn extract_request_context(request: &Request) -> Option<RequestContext> {
        request.extensions().get::<RequestContext>().cloned()
    }

    /// コンテキスト挿入レイヤー
    ///
    /// リクエストエクステンションに RequestContext を挿入するレイヤー。
    #[derive(Clone)]
    pub struct ContextLayer;

    impl<S> Layer<S> for ContextLayer {
        type Service = ContextMiddleware<S>;

        fn layer(&self, inner: S) -> Self::Service {
            ContextMiddleware { inner }
        }
    }

    /// コンテキスト挿入ミドルウェア
    #[derive(Clone)]
    pub struct ContextMiddleware<S> {
        inner: S,
    }

    impl<S> Service<Request> for ContextMiddleware<S>
    where
        S: Service<Request, Response = Response> + Clone + Send + 'static,
        S::Future: Send + 'static,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = S::Future;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, mut request: Request) -> Self::Future {
            // traceparent ヘッダからコンテキストを取得または作成
            let ctx = request
                .headers()
                .get("traceparent")
                .and_then(|v| v.to_str().ok())
                .and_then(RequestContext::from_traceparent)
                .unwrap_or_else(RequestContext::new);

            request.extensions_mut().insert(ctx);

            self.inner.call(request)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_observability_layer_creation() {
            let layer = ObservabilityLayer::new("test-service", "dev");
            assert!(Arc::strong_count(&layer.inner) >= 1);
        }

        #[test]
        fn test_skip_paths() {
            let layer = ObservabilityLayer::new("test", "dev")
                .skip_paths(vec!["/health".to_string(), "/metrics".to_string()]);

            assert!(layer.inner.skip_paths.contains(&"/health".to_string()));
            assert!(layer.inner.skip_paths.contains(&"/metrics".to_string()));
        }
    }
}
