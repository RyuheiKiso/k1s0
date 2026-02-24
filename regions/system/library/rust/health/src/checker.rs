use std::collections::HashMap;

use async_trait::async_trait;

use crate::error::HealthError;
use crate::response::{CheckResult, HealthResponse, HealthStatus, HealthzResponse};

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> Result<(), HealthError>;
}

pub trait HealthChecker: Send + Sync {
    fn add_check(&mut self, check: Box<dyn HealthCheck>);
}

pub struct CompositeHealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl CompositeHealthChecker {
    pub fn new() -> Self {
        Self { checks: vec![] }
    }

    pub fn add_check(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }

    pub async fn run_all(&self) -> HealthResponse {
        let mut results = HashMap::new();
        let mut overall = HealthStatus::Healthy;

        for check in &self.checks {
            let (status, message) = match check.check().await {
                Ok(()) => (HealthStatus::Healthy, None),
                Err(e) => {
                    overall = HealthStatus::Unhealthy;
                    (HealthStatus::Unhealthy, Some(e.to_string()))
                }
            };
            results.insert(check.name().to_string(), CheckResult { status, message });
        }

        HealthResponse {
            status: overall,
            checks: results,
            timestamp: chrono_now(),
        }
    }

    /// readyz は全ヘルスチェッカーを実行し、トラフィック受け入れ可否を返す。
    /// run_all() と同等。
    pub async fn readyz(&self) -> HealthResponse {
        self.run_all().await
    }

    /// healthz は死活確認用エンドポイント。常に ok を返す。
    pub fn healthz(&self) -> HealthzResponse {
        HealthzResponse {
            status: "ok".to_string(),
        }
    }
}

impl Default for CompositeHealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysHealthy;

    #[async_trait]
    impl HealthCheck for AlwaysHealthy {
        fn name(&self) -> &str {
            "always-healthy"
        }
        async fn check(&self) -> Result<(), HealthError> {
            Ok(())
        }
    }

    struct AlwaysUnhealthy;

    #[async_trait]
    impl HealthCheck for AlwaysUnhealthy {
        fn name(&self) -> &str {
            "always-unhealthy"
        }
        async fn check(&self) -> Result<(), HealthError> {
            Err(HealthError::CheckFailed("down".to_string()))
        }
    }

    #[tokio::test]
    async fn test_all_healthy() {
        let mut checker = CompositeHealthChecker::new();
        checker.add_check(Box::new(AlwaysHealthy));

        let response = checker.run_all().await;
        assert_eq!(response.status, HealthStatus::Healthy);
        assert_eq!(response.checks.len(), 1);
    }

    #[tokio::test]
    async fn test_one_unhealthy() {
        let mut checker = CompositeHealthChecker::new();
        checker.add_check(Box::new(AlwaysHealthy));
        checker.add_check(Box::new(AlwaysUnhealthy));

        let response = checker.run_all().await;
        assert_eq!(response.status, HealthStatus::Unhealthy);
        assert_eq!(response.checks.len(), 2);
        assert_eq!(
            response.checks["always-healthy"].status,
            HealthStatus::Healthy
        );
        assert_eq!(
            response.checks["always-unhealthy"].status,
            HealthStatus::Unhealthy
        );
    }

    #[tokio::test]
    async fn test_empty_checker() {
        let checker = CompositeHealthChecker::new();
        let response = checker.run_all().await;
        assert_eq!(response.status, HealthStatus::Healthy);
        assert!(response.checks.is_empty());
    }

    #[tokio::test]
    async fn test_readyz_is_alias_of_run_all() {
        let mut checker = CompositeHealthChecker::new();
        checker.add_check(Box::new(AlwaysHealthy));

        let readyz = checker.readyz().await;
        assert_eq!(readyz.status, HealthStatus::Healthy);
        assert_eq!(readyz.checks.len(), 1);
    }

    #[test]
    fn test_healthz_always_ok() {
        let checker = CompositeHealthChecker::new();
        let response = checker.healthz();
        assert_eq!(response.status, "ok");
    }
}
