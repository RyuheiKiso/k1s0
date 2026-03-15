use std::sync::Arc;

use axum::Router;

use k1s0_correlation::layer::CorrelationLayer;
use k1s0_health::CompositeHealthChecker;
use k1s0_telemetry::metrics::Metrics;
use k1s0_telemetry::MetricsLayer;

use super::request_id::RequestIdLayer;
use super::standard_routes;

/// Profile はデプロイ環境を表す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Profile {
    Dev,
    Staging,
    Prod,
}

impl Profile {
    pub fn from_env(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "prod" | "production" => Self::Prod,
            "staging" | "stg" => Self::Staging,
            _ => Self::Dev,
        }
    }
}

/// K1s0Stack は標準ミドルウェアスタックを構築するビルダー。
pub struct K1s0Stack {
    #[allow(dead_code)]
    service_name: String,
    profile: Profile,
    metrics: Option<Arc<Metrics>>,
    health_checker: Option<Arc<CompositeHealthChecker>>,
    correlation: bool,
    request_id: bool,
}

impl K1s0Stack {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            profile: Profile::Dev,
            metrics: None,
            health_checker: None,
            correlation: true,
            request_id: true,
        }
    }

    pub fn profile(mut self, profile: Profile) -> Self {
        self.profile = profile;
        self
    }

    pub fn metrics(mut self, metrics: Arc<Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn health_checker(mut self, checker: Arc<CompositeHealthChecker>) -> Self {
        self.health_checker = Some(checker);
        self
    }

    pub fn without_correlation(mut self) -> Self {
        self.correlation = false;
        self
    }

    pub fn without_request_id(mut self) -> Self {
        self.request_id = false;
        self
    }

    /// wrap はRouterにミドルウェアスタックと標準エンドポイントを適用する。
    ///
    /// レイヤー適用順序（外→内）:
    /// 1. MetricsLayer（metrics設定時）
    /// 2. CorrelationLayer（correlation有効時）
    /// 3. RequestIdLayer（request_id有効時）
    pub fn wrap(self, router: Router) -> Router {
        // 標準エンドポイントをマージ
        let router = standard_routes::merge_standard_routes(
            router,
            self.metrics.clone(),
            self.health_checker,
        );

        // レイヤー適用（内→外の順で .layer() を呼ぶ）
        // axumでは最後にlayerしたものが最も外側になる
        let router = if self.request_id {
            router.layer(RequestIdLayer::new())
        } else {
            router
        };

        let router = if self.correlation {
            router.layer(CorrelationLayer::new())
        } else {
            router
        };

        if let Some(metrics) = self.metrics {
            router.layer(MetricsLayer::new(metrics))
        } else {
            router
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    // /healthz エンドポイントが200を返すことを確認する。
    #[tokio::test]
    async fn test_healthz_returns_ok() {
        let stack = K1s0Stack::new("test-server");
        let app = stack.wrap(Router::new());

        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    // メトリクス設定付きで /metrics エンドポイントが200を返すことを確認する。
    #[tokio::test]
    async fn test_metrics_endpoint() {
        let metrics = Arc::new(Metrics::new("test-server"));
        let stack = K1s0Stack::new("test-server").metrics(metrics);
        let app = stack.wrap(Router::new());

        let req = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    // without_correlation を指定したとき x-correlation-id ヘッダーが付与されないことを確認する。
    #[tokio::test]
    async fn test_without_correlation_disables_layer() {
        let stack = K1s0Stack::new("test-server").without_correlation();
        let app = stack.wrap(Router::new());

        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);
        // CorrelationLayer無効なのでx-correlation-idヘッダーなし
        assert!(!resp.headers().contains_key("x-correlation-id"));
    }

    // 各環境名文字列が正しい Profile に変換されることを確認する。
    #[test]
    fn test_profile_from_env() {
        assert_eq!(Profile::from_env("prod"), Profile::Prod);
        assert_eq!(Profile::from_env("production"), Profile::Prod);
        assert_eq!(Profile::from_env("staging"), Profile::Staging);
        assert_eq!(Profile::from_env("stg"), Profile::Staging);
        assert_eq!(Profile::from_env("dev"), Profile::Dev);
        assert_eq!(Profile::from_env("development"), Profile::Dev);
        assert_eq!(Profile::from_env("anything-else"), Profile::Dev);
    }
}
