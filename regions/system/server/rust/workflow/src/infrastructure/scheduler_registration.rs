use anyhow::Context;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::infrastructure::config::Config;

const OVERDUE_CHECK_JOB_NAME: &str = "workflow-overdue-check";

pub async fn register_overdue_check_job(cfg: &Config) -> anyhow::Result<()> {
    let Some(scheduler_cfg) = &cfg.scheduler else {
        return Ok(());
    };

    let client = Client::new();

    let existing_jobs: ListJobsResponse = client
        .get(format!("{}/api/v1/jobs", scheduler_cfg.internal_endpoint))
        .query(&[
            ("name_prefix", OVERDUE_CHECK_JOB_NAME),
            ("page_size", "500"),
        ])
        .send()
        .await
        .with_context(|| "failed to list scheduler jobs for workflow overdue check")?
        .error_for_status()
        .with_context(|| "failed to fetch scheduler jobs for workflow overdue check")?
        .json()
        .await
        .with_context(|| "failed to decode scheduler jobs response")?;

    if existing_jobs
        .jobs
        .iter()
        .any(|job| job.name == OVERDUE_CHECK_JOB_NAME)
    {
        info!(
            job_name = OVERDUE_CHECK_JOB_NAME,
            "workflow overdue check job is already registered"
        );
        return Ok(());
    }

    let target = overdue_check_target_url(cfg);
    let job: SchedulerJob = client
        .post(format!("{}/api/v1/jobs", scheduler_cfg.internal_endpoint))
        .json(&CreateJobRequest {
            name: OVERDUE_CHECK_JOB_NAME.to_string(),
            description: Some("Check overdue workflow tasks and publish notifications".to_string()),
            cron_expression: cfg.overdue_check.cron_expression.clone(),
            timezone: Some(cfg.overdue_check.timezone.clone()),
            target_type: "http".to_string(),
            target: Some(target.clone()),
            payload: json!({}),
        })
        .send()
        .await
        .with_context(|| "failed to create workflow overdue check job")?
        .error_for_status()
        .with_context(|| "scheduler rejected workflow overdue check job")?
        .json()
        .await
        .with_context(|| "failed to decode created workflow overdue check job")?;

    info!(
        job_id = %job.id,
        job_name = %job.name,
        target = %target,
        cron = %cfg.overdue_check.cron_expression,
        "registered workflow overdue check job"
    );
    Ok(())
}

#[must_use] 
pub fn overdue_check_target_url(cfg: &Config) -> String {
    if let Ok(base_url) = std::env::var("WORKFLOW_INTERNAL_BASE_URL") {
        return format!(
            "{}/internal/tasks/check-overdue",
            base_url.trim_end_matches('/')
        );
    }

    let base_url = match cfg.app.environment.as_str() {
        "docker" => format!("http://workflow:{}", cfg.server.port),
        "prod" | "production" | "staging" => {
            format!(
                "http://workflow.k1s0-system.svc.cluster.local:{}",
                cfg.server.port
            )
        }
        _ if cfg.server.host != "0.0.0.0" => {
            format!("http://{}:{}", cfg.server.host, cfg.server.port)
        }
        _ => format!("http://127.0.0.1:{}", cfg.server.port),
    };

    format!("{base_url}/internal/tasks/check-overdue")
}

#[derive(Debug, Deserialize)]
struct ListJobsResponse {
    jobs: Vec<SchedulerJob>,
}

#[derive(Debug, Deserialize)]
struct SchedulerJob {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct CreateJobRequest {
    name: String,
    description: Option<String>,
    cron_expression: String,
    timezone: Option<String>,
    target_type: String,
    target: Option<String>,
    payload: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::overdue_check_target_url;
    use crate::infrastructure::config::{
        AppConfig, Config, LogConfig, MetricsConfig, ObservabilityConfig, OverdueCheckConfig,
        ServerConfig, TraceConfig,
    };

    fn base_config(environment: &str, host: &str, port: u16) -> Config {
        Config {
            app: AppConfig {
                name: "workflow".to_string(),
                // Cargo.toml の package.version を使用する（M-16 監査対応: ハードコード解消）
                version: env!("CARGO_PKG_VERSION").to_string(),
                environment: environment.to_string(),
            },
            server: ServerConfig {
                host: host.to_string(),
                port,
                grpc_port: 50051,
            },
            observability: ObservabilityConfig {
                log: LogConfig::default(),
                trace: TraceConfig::default(),
                metrics: MetricsConfig::default(),
            },
            auth: None,
            database: None,
            kafka: None,
            scheduler: None,
            overdue_check: OverdueCheckConfig::default(),
        }
    }

    #[test]
    fn resolves_cluster_target_for_prod() {
        std::env::remove_var("WORKFLOW_INTERNAL_BASE_URL");
        let cfg = base_config("prod", "0.0.0.0", 8100);
        assert_eq!(
            overdue_check_target_url(&cfg),
            "http://workflow.k1s0-system.svc.cluster.local:8100/internal/tasks/check-overdue"
        );
    }

    #[test]
    fn resolves_local_target_for_dev() {
        std::env::remove_var("WORKFLOW_INTERNAL_BASE_URL");
        let cfg = base_config("dev", "0.0.0.0", 8100);
        assert_eq!(
            overdue_check_target_url(&cfg),
            "http://127.0.0.1:8100/internal/tasks/check-overdue"
        );
    }
}
