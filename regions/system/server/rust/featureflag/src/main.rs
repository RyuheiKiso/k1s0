#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

mod adapter;
mod domain;
mod infrastructure;
mod usecase;

use adapter::grpc::FeatureFlagGrpcService;
use domain::entity::feature_flag::FeatureFlag;
use domain::repository::FeatureFlagRepository;
use infrastructure::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-featureflag-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string()),
        trace_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        sample_rate: 1.0,
        log_level: "info".to_string(),
    };
    k1s0_telemetry::init_telemetry(&telemetry_cfg).expect("failed to init telemetry");

    // Config
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting featureflag server"
    );

    // In-memory flag repository (default stub)
    let flag_repo: Arc<dyn FeatureFlagRepository> = Arc::new(InMemoryFeatureFlagRepository::new());

    // Use cases
    let evaluate_flag_uc = Arc::new(usecase::EvaluateFlagUseCase::new(flag_repo.clone()));
    let get_flag_uc = Arc::new(usecase::GetFlagUseCase::new(flag_repo.clone()));
    let create_flag_uc = Arc::new(usecase::CreateFlagUseCase::new(flag_repo.clone()));
    let update_flag_uc = Arc::new(usecase::UpdateFlagUseCase::new(flag_repo.clone()));

    let _grpc_svc = Arc::new(FeatureFlagGrpcService::new(
        evaluate_flag_uc.clone(),
        get_flag_uc.clone(),
        create_flag_uc.clone(),
        update_flag_uc.clone(),
    ));

    // AppState for REST handlers
    let state = adapter::handler::AppState {
        flag_repo: flag_repo.clone(),
        evaluate_flag_uc,
        get_flag_uc,
        create_flag_uc,
        update_flag_uc,
    };

    // REST router
    let app = adapter::handler::router(state);

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// --- InMemoryFeatureFlagRepository ---

struct InMemoryFeatureFlagRepository {
    flags: tokio::sync::RwLock<HashMap<String, FeatureFlag>>,
}

impl InMemoryFeatureFlagRepository {
    fn new() -> Self {
        Self {
            flags: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl FeatureFlagRepository for InMemoryFeatureFlagRepository {
    async fn find_by_key(&self, flag_key: &str) -> anyhow::Result<FeatureFlag> {
        let flags = self.flags.read().await;
        flags
            .get(flag_key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("flag not found: {}", flag_key))
    }

    async fn find_all(&self) -> anyhow::Result<Vec<FeatureFlag>> {
        let flags = self.flags.read().await;
        Ok(flags.values().cloned().collect())
    }

    async fn create(&self, flag: &FeatureFlag) -> anyhow::Result<()> {
        let mut flags = self.flags.write().await;
        flags.insert(flag.flag_key.clone(), flag.clone());
        Ok(())
    }

    async fn update(&self, flag: &FeatureFlag) -> anyhow::Result<()> {
        let mut flags = self.flags.write().await;
        flags.insert(flag.flag_key.clone(), flag.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut flags = self.flags.write().await;
        let key = flags
            .iter()
            .find(|(_, v)| v.id == *id)
            .map(|(k, _)| k.clone());
        if let Some(key) = key {
            flags.remove(&key);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn exists_by_key(&self, flag_key: &str) -> anyhow::Result<bool> {
        let flags = self.flags.read().await;
        Ok(flags.contains_key(flag_key))
    }
}
