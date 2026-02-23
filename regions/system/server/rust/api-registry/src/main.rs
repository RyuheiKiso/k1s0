#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod usecase;

use domain::entity::api_registration::{ApiSchema, ApiSchemaVersion, SchemaType};
use domain::repository::{ApiSchemaRepository, ApiSchemaVersionRepository};

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
    8101
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting api-registry server"
    );

    let schema_repo: Arc<dyn ApiSchemaRepository> =
        Arc::new(InMemoryApiSchemaRepository::new());
    let version_repo: Arc<dyn ApiSchemaVersionRepository> =
        Arc::new(InMemoryApiSchemaVersionRepository::new());

    let _list_schemas_uc = Arc::new(usecase::ListSchemasUseCase::new(schema_repo.clone()));
    let _register_schema_uc = Arc::new(usecase::RegisterSchemaUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));
    let _get_schema_uc = Arc::new(usecase::GetSchemaUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));
    let _list_versions_uc = Arc::new(usecase::ListVersionsUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));
    let _get_schema_version_uc =
        Arc::new(usecase::GetSchemaVersionUseCase::new(version_repo.clone()));
    let _register_version_uc = Arc::new(usecase::RegisterVersionUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));
    let _delete_version_uc = Arc::new(usecase::DeleteVersionUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));
    let _check_compatibility_uc = Arc::new(usecase::CheckCompatibilityUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));
    let _get_diff_uc = Arc::new(usecase::GetDiffUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));

    let app = axum::Router::new()
        .route(
            "/healthz",
            axum::routing::get(adapter::handler::health::healthz),
        )
        .route(
            "/readyz",
            axum::routing::get(adapter::handler::health::readyz),
        );

    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// --- InMemory Repositories ---

struct InMemoryApiSchemaRepository {
    schemas: tokio::sync::RwLock<HashMap<String, ApiSchema>>,
}

impl InMemoryApiSchemaRepository {
    fn new() -> Self {
        Self {
            schemas: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl ApiSchemaRepository for InMemoryApiSchemaRepository {
    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<ApiSchema>> {
        let schemas = self.schemas.read().await;
        Ok(schemas.get(name).cloned())
    }

    async fn find_all(
        &self,
        schema_type: Option<String>,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchema>, u64)> {
        let schemas = self.schemas.read().await;
        let filtered: Vec<ApiSchema> = schemas
            .values()
            .filter(|s| {
                schema_type
                    .as_ref()
                    .map(|t| s.schema_type.to_string() == *t)
                    .unwrap_or(true)
            })
            .cloned()
            .collect();
        let count = filtered.len() as u64;
        Ok((filtered, count))
    }

    async fn create(&self, schema: &ApiSchema) -> anyhow::Result<()> {
        let mut schemas = self.schemas.write().await;
        schemas.insert(schema.name.clone(), schema.clone());
        Ok(())
    }

    async fn update(&self, schema: &ApiSchema) -> anyhow::Result<()> {
        let mut schemas = self.schemas.write().await;
        schemas.insert(schema.name.clone(), schema.clone());
        Ok(())
    }
}

struct InMemoryApiSchemaVersionRepository {
    versions: tokio::sync::RwLock<Vec<ApiSchemaVersion>>,
}

impl InMemoryApiSchemaVersionRepository {
    fn new() -> Self {
        Self {
            versions: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl ApiSchemaVersionRepository for InMemoryApiSchemaVersionRepository {
    async fn find_by_name_and_version(
        &self,
        name: &str,
        version: u32,
    ) -> anyhow::Result<Option<ApiSchemaVersion>> {
        let versions = self.versions.read().await;
        Ok(versions
            .iter()
            .find(|v| v.name == name && v.version == version)
            .cloned())
    }

    async fn find_latest_by_name(&self, name: &str) -> anyhow::Result<Option<ApiSchemaVersion>> {
        let versions = self.versions.read().await;
        Ok(versions
            .iter()
            .filter(|v| v.name == name)
            .max_by_key(|v| v.version)
            .cloned())
    }

    async fn find_all_by_name(
        &self,
        name: &str,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchemaVersion>, u64)> {
        let versions = self.versions.read().await;
        let filtered: Vec<ApiSchemaVersion> = versions
            .iter()
            .filter(|v| v.name == name)
            .cloned()
            .collect();
        let count = filtered.len() as u64;
        Ok((filtered, count))
    }

    async fn create(&self, version: &ApiSchemaVersion) -> anyhow::Result<()> {
        let mut versions = self.versions.write().await;
        versions.push(version.clone());
        Ok(())
    }

    async fn delete(&self, name: &str, version: u32) -> anyhow::Result<bool> {
        let mut versions = self.versions.write().await;
        let len_before = versions.len();
        versions.retain(|v| !(v.name == name && v.version == version));
        Ok(versions.len() < len_before)
    }

    async fn count_by_name(&self, name: &str) -> anyhow::Result<u64> {
        let versions = self.versions.read().await;
        Ok(versions.iter().filter(|v| v.name == name).count() as u64)
    }
}
