use std::sync::Arc;

use axum::Router;

use k1s0_health::{CompositeHealthChecker, HealthCheck};
use k1s0_telemetry::metrics::Metrics;
use k1s0_telemetry::TelemetryConfig;

use super::stack::{K1s0Stack, Profile};

/// K1s0App はサーバー初期化のボイラープレートを削減する上位ビルダー。
///
/// Config → Telemetry → Metrics → HealthCheck → K1s0Stack の初期化を一括で行い、
/// 初期化順序ミス・設定漏れを構造的に排除する。
pub struct K1s0App {
    telemetry_config: TelemetryConfig,
    profile: Option<Profile>,
    health_checks: Vec<Box<dyn HealthCheck>>,
    skip_correlation: bool,
    skip_request_id: bool,
}

impl K1s0App {
    pub fn new(telemetry_config: TelemetryConfig) -> Self {
        Self {
            telemetry_config,
            profile: None,
            health_checks: Vec::new(),
            skip_correlation: false,
            skip_request_id: false,
        }
    }

    pub fn profile(mut self, profile: Profile) -> Self {
        self.profile = Some(profile);
        self
    }

    pub fn add_health_check(mut self, check: Box<dyn HealthCheck>) -> Self {
        self.health_checks.push(check);
        self
    }

    pub fn without_correlation(mut self) -> Self {
        self.skip_correlation = true;
        self
    }

    pub fn without_request_id(mut self) -> Self {
        self.skip_request_id = true;
        self
    }

    /// Telemetry初期化 → Metrics生成 → HealthChecker構築 → K1s0AppReady返却。
    pub async fn build(self) -> Result<K1s0AppReady, Box<dyn std::error::Error>> {
        k1s0_telemetry::init_telemetry(&self.telemetry_config)?;
        self.build_inner()
    }

    /// テスト用: Telemetry初期化をスキップしてビルドする。
    #[cfg(test)]
    pub(crate) fn build_for_test(self) -> Result<K1s0AppReady, Box<dyn std::error::Error>> {
        self.build_inner()
    }

    fn build_inner(self) -> Result<K1s0AppReady, Box<dyn std::error::Error>> {
        let service_name = self.telemetry_config.service_name.clone();
        let profile = self.profile.unwrap_or_else(|| {
            Profile::from_env(&self.telemetry_config.environment)
        });

        let metrics = Arc::new(Metrics::new(&service_name));

        let mut health_checker = CompositeHealthChecker::new();
        for check in self.health_checks {
            health_checker.add_check(check);
        }
        let health_checker = Arc::new(health_checker);

        Ok(K1s0AppReady {
            service_name,
            profile,
            metrics,
            health_checker,
            skip_correlation: self.skip_correlation,
            skip_request_id: self.skip_request_id,
        })
    }
}

/// K1s0AppReady はビルド完了後の不変な状態を保持し、wrap() でRouterに適用する。
pub struct K1s0AppReady {
    service_name: String,
    profile: Profile,
    metrics: Arc<Metrics>,
    health_checker: Arc<CompositeHealthChecker>,
    skip_correlation: bool,
    skip_request_id: bool,
}

impl K1s0AppReady {
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub fn metrics(&self) -> Arc<Metrics> {
        self.metrics.clone()
    }

    pub fn health_checker(&self) -> Arc<CompositeHealthChecker> {
        self.health_checker.clone()
    }

    /// K1s0Stackを内部構築し、Routerにミドルウェアスタック + 標準エンドポイントを適用する。
    pub fn wrap(&self, router: Router) -> Router {
        let mut stack = K1s0Stack::new(&self.service_name)
            .profile(self.profile.clone())
            .metrics(self.metrics.clone())
            .health_checker(self.health_checker.clone());

        if self.skip_correlation {
            stack = stack.without_correlation();
        }
        if self.skip_request_id {
            stack = stack.without_request_id();
        }

        stack.wrap(router)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use k1s0_health::HealthError;
    use tower::ServiceExt;

    fn test_telemetry_config(service_name: &str) -> TelemetryConfig {
        TelemetryConfig {
            service_name: service_name.to_string(),
            version: "0.1.0".to_string(),
            tier: "system".to_string(),
            environment: "dev".to_string(),
            trace_endpoint: None,
            sample_rate: 0.0,
            log_level: "info".to_string(),
            log_format: "text".to_string(),
        }
    }

    struct AlwaysHealthy;

    #[async_trait::async_trait]
    impl HealthCheck for AlwaysHealthy {
        fn name(&self) -> &str {
            "always-healthy"
        }
        async fn check(&self) -> Result<(), HealthError> {
            Ok(())
        }
    }

    #[test]
    fn test_build_returns_ready() {
        let app = K1s0App::new(test_telemetry_config("test-server"))
            .build_for_test()
            .unwrap();

        assert_eq!(app.service_name(), "test-server");
    }

    #[tokio::test]
    async fn test_health_checks_added() {
        let app = K1s0App::new(test_telemetry_config("test-server"))
            .add_health_check(Box::new(AlwaysHealthy))
            .build_for_test()
            .unwrap();

        let resp = app.health_checker().run_all().await;
        assert!(resp.checks.contains_key("always-healthy"));
    }

    #[tokio::test]
    async fn test_wrap_adds_healthz() {
        let app = K1s0App::new(test_telemetry_config("test-server"))
            .build_for_test()
            .unwrap();

        let router = app.wrap(Router::new());

        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_wrap_adds_metrics() {
        let app = K1s0App::new(test_telemetry_config("test-server"))
            .build_for_test()
            .unwrap();

        let router = app.wrap(Router::new());

        let req = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[test]
    fn test_profile_auto_detected() {
        let cfg = TelemetryConfig {
            environment: "prod".to_string(),
            ..test_telemetry_config("test-server")
        };

        let app = K1s0App::new(cfg).build_for_test().unwrap();
        assert_eq!(app.profile(), &Profile::Prod);
    }

    #[test]
    fn test_profile_explicit_overrides_env() {
        let cfg = TelemetryConfig {
            environment: "prod".to_string(),
            ..test_telemetry_config("test-server")
        };

        let app = K1s0App::new(cfg)
            .profile(Profile::Dev)
            .build_for_test()
            .unwrap();
        assert_eq!(app.profile(), &Profile::Dev);
    }

    #[tokio::test]
    async fn test_without_options_propagate() {
        let app = K1s0App::new(test_telemetry_config("test-server"))
            .without_correlation()
            .without_request_id()
            .build_for_test()
            .unwrap();

        let router = app.wrap(Router::new());

        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);
        // CorrelationLayer無効なのでx-correlation-idヘッダーなし
        assert!(!resp.headers().contains_key("x-correlation-id"));
        // RequestIdLayer無効なのでx-request-idヘッダーなし
        assert!(!resp.headers().contains_key("x-request-id"));
    }
}
