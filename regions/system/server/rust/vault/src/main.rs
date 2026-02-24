#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod usecase;

use adapter::grpc::VaultGrpcService;
use adapter::handler::{self, AppState};
use domain::entity::secret::Secret;
use domain::repository::{AccessLogRepository, SecretStore};

/// Application configuration.
#[derive(Debug, Clone, serde::Deserialize)]
struct Config {
    app: AppConfig,
    server: ServerConfig,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AppConfig {
    name: String,
    #[serde(default = "default_version")]
    version: String,
    #[serde(default = "default_environment")]
    environment: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_environment() -> String {
    "dev".to_string()
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ServerConfig {
    #[serde(default = "default_host")]
    host: String,
    #[serde(default = "default_port")]
    port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8090
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-vault-server".to_string(),
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
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting vault server"
    );

    // InMemory implementations for dev mode
    let secret_store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
    let audit_repo: Arc<dyn AccessLogRepository> = Arc::new(NoopAccessLogRepository);

    // Use cases
    let get_secret_uc = Arc::new(usecase::GetSecretUseCase::new(
        secret_store.clone(),
        audit_repo.clone(),
    ));
    let set_secret_uc = Arc::new(usecase::SetSecretUseCase::new(
        secret_store.clone(),
        audit_repo.clone(),
    ));
    let delete_secret_uc = Arc::new(usecase::DeleteSecretUseCase::new(
        secret_store.clone(),
        audit_repo.clone(),
    ));
    let list_secrets_uc = Arc::new(usecase::ListSecretsUseCase::new(secret_store));

    // gRPC service
    let _vault_grpc_svc = Arc::new(VaultGrpcService::new(
        get_secret_uc.clone(),
        set_secret_uc.clone(),
        delete_secret_uc.clone(),
        list_secrets_uc.clone(),
    ));

    // AppState (REST)
    let state = AppState {
        get_secret_uc,
        set_secret_uc,
        delete_secret_uc,
        list_secrets_uc,
        db_pool: None,
    };

    // REST Router
    let app = handler::router(state);

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// --- InMemory SecretStore ---

struct InMemorySecretStore {
    secrets: tokio::sync::RwLock<HashMap<String, Secret>>,
}

impl InMemorySecretStore {
    fn new() -> Self {
        Self {
            secrets: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl domain::repository::SecretStore for InMemorySecretStore {
    async fn get(&self, path: &str, version: Option<i64>) -> anyhow::Result<Secret> {
        let store = self.secrets.read().await;
        let secret = store
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("secret not found: {}", path))?;
        if let Some(v) = version {
            secret
                .versions
                .iter()
                .find(|sv| sv.version == v && !sv.destroyed)
                .ok_or_else(|| anyhow::anyhow!("version {} not found", v))?;
        }
        Ok(secret.clone())
    }

    async fn set(&self, path: &str, data: HashMap<String, String>) -> anyhow::Result<i64> {
        let mut store = self.secrets.write().await;
        let version = if let Some(existing) = store.get_mut(path) {
            let updated = existing.clone().update(data);
            let v = updated.current_version;
            *existing = updated;
            v
        } else {
            let secret = Secret::new(path.to_string(), data);
            store.insert(path.to_string(), secret);
            1
        };
        Ok(version)
    }

    async fn delete(&self, path: &str, versions: Vec<i64>) -> anyhow::Result<()> {
        let mut store = self.secrets.write().await;
        if let Some(secret) = store.get_mut(path) {
            for sv in &mut secret.versions {
                if versions.is_empty() || versions.contains(&sv.version) {
                    sv.destroyed = true;
                }
            }
        }
        Ok(())
    }

    async fn list(&self, path_prefix: &str) -> anyhow::Result<Vec<String>> {
        let store = self.secrets.read().await;
        Ok(store
            .keys()
            .filter(|k| k.starts_with(path_prefix))
            .cloned()
            .collect())
    }

    async fn exists(&self, path: &str) -> anyhow::Result<bool> {
        Ok(self.secrets.read().await.contains_key(path))
    }
}

// --- Noop AccessLogRepository ---

struct NoopAccessLogRepository;

#[async_trait::async_trait]
impl domain::repository::AccessLogRepository for NoopAccessLogRepository {
    async fn record(
        &self,
        _log: &domain::entity::access_log::SecretAccessLog,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
