use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tracing::{info, warn};

use crate::domain::entity::health::{HealthState, HealthStatus};
use crate::domain::repository::{HealthRepository, ServiceRepository};
use crate::domain::repository::service_repository::ServiceListFilters;

/// HealthCollectorConfig はヘルスコレクターの設定を表す。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HealthCollectorConfig {
    #[serde(default = "default_interval_secs")]
    pub interval_secs: u64,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for HealthCollectorConfig {
    fn default() -> Self {
        Self {
            interval_secs: default_interval_secs(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

fn default_interval_secs() -> u64 {
    60
}

fn default_timeout_secs() -> u64 {
    5
}

/// HealthCollector はサービスの /healthz エンドポイントを定期的にポーリングするバックグラウンドタスク。
pub struct HealthCollector {
    service_repo: Arc<dyn ServiceRepository>,
    health_repo: Arc<dyn HealthRepository>,
    http_client: reqwest::Client,
    config: HealthCollectorConfig,
}

impl HealthCollector {
    pub fn new(
        service_repo: Arc<dyn ServiceRepository>,
        health_repo: Arc<dyn HealthRepository>,
        config: HealthCollectorConfig,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("failed to build HTTP client");

        Self {
            service_repo,
            health_repo,
            http_client,
            config,
        }
    }

    /// バックグラウンドタスクとしてヘルスチェックループを開始する。
    pub async fn run(&self) {
        info!(
            interval_secs = self.config.interval_secs,
            "starting health collector"
        );

        loop {
            self.collect().await;
            tokio::time::sleep(Duration::from_secs(self.config.interval_secs)).await;
        }
    }

    async fn collect(&self) {
        let services = match self.service_repo.list(ServiceListFilters::default()).await {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, "failed to list services for health check");
                return;
            }
        };

        for service in services {
            let healthcheck_url = match &service.healthcheck_url {
                Some(url) if !url.is_empty() => url.clone(),
                _ => continue,
            };

            let start = std::time::Instant::now();
            let (state, message, response_time_ms) = match self
                .http_client
                .get(&healthcheck_url)
                .send()
                .await
            {
                Ok(resp) => {
                    let elapsed = start.elapsed().as_millis() as i64;
                    if resp.status().is_success() {
                        (HealthState::Healthy, None, Some(elapsed))
                    } else {
                        (
                            HealthState::Degraded,
                            Some(format!("HTTP {}", resp.status())),
                            Some(elapsed),
                        )
                    }
                }
                Err(e) => {
                    let elapsed = start.elapsed().as_millis() as i64;
                    (
                        HealthState::Unhealthy,
                        Some(e.to_string()),
                        Some(elapsed),
                    )
                }
            };

            let health = HealthStatus {
                service_id: service.id,
                status: state,
                message,
                response_time_ms,
                checked_at: Utc::now(),
            };

            if let Err(e) = self.health_repo.upsert(&health).await {
                warn!(
                    service_id = %service.id,
                    error = %e,
                    "failed to upsert health status"
                );
            }
        }
    }
}
