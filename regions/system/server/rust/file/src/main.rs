#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod usecase;

use domain::entity::file::FileMetadata;
use domain::repository::{FileMetadataRepository, FileStorageRepository};

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
    8098
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
        "starting file server"
    );

    let metadata_repo: Arc<dyn FileMetadataRepository> =
        Arc::new(InMemoryFileMetadataRepository::new());
    let storage_repo: Arc<dyn FileStorageRepository> =
        Arc::new(InMemoryFileStorageRepository::new());

    let _list_files_uc = Arc::new(usecase::ListFilesUseCase::new(metadata_repo.clone()));
    let _generate_upload_url_uc = Arc::new(usecase::GenerateUploadUrlUseCase::new(
        metadata_repo.clone(),
        storage_repo.clone(),
    ));
    let _complete_upload_uc =
        Arc::new(usecase::CompleteUploadUseCase::new(metadata_repo.clone()));
    let _get_file_metadata_uc =
        Arc::new(usecase::GetFileMetadataUseCase::new(metadata_repo.clone()));
    let _generate_download_url_uc = Arc::new(usecase::GenerateDownloadUrlUseCase::new(
        metadata_repo.clone(),
        storage_repo.clone(),
    ));
    let _delete_file_uc = Arc::new(usecase::DeleteFileUseCase::new(
        metadata_repo.clone(),
        storage_repo.clone(),
    ));
    let _update_file_tags_uc =
        Arc::new(usecase::UpdateFileTagsUseCase::new(metadata_repo.clone()));

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

struct InMemoryFileMetadataRepository {
    files: tokio::sync::RwLock<HashMap<String, FileMetadata>>,
}

impl InMemoryFileMetadataRepository {
    fn new() -> Self {
        Self {
            files: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl FileMetadataRepository for InMemoryFileMetadataRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<FileMetadata>> {
        let files = self.files.read().await;
        Ok(files.get(id).cloned())
    }

    async fn find_all(
        &self,
        tenant_id: Option<String>,
        _owner_id: Option<String>,
        _mime_type: Option<String>,
        _tag: Option<(String, String)>,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<FileMetadata>, u64)> {
        let files = self.files.read().await;
        let filtered: Vec<FileMetadata> = files
            .values()
            .filter(|f| {
                if let Some(ref tid) = tenant_id {
                    f.tenant_id == *tid
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        Ok((filtered, total))
    }

    async fn create(&self, file: &FileMetadata) -> anyhow::Result<()> {
        let mut files = self.files.write().await;
        files.insert(file.id.clone(), file.clone());
        Ok(())
    }

    async fn update(&self, file: &FileMetadata) -> anyhow::Result<()> {
        let mut files = self.files.write().await;
        files.insert(file.id.clone(), file.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let mut files = self.files.write().await;
        Ok(files.remove(id).is_some())
    }
}

struct InMemoryFileStorageRepository;

impl InMemoryFileStorageRepository {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl FileStorageRepository for InMemoryFileStorageRepository {
    async fn generate_upload_url(
        &self,
        storage_key: &str,
        _mime_type: &str,
        _expires_in_seconds: u32,
    ) -> anyhow::Result<String> {
        Ok(format!(
            "https://storage.example.com/upload/{}?sig=mock",
            storage_key
        ))
    }

    async fn generate_download_url(
        &self,
        storage_key: &str,
        _expires_in_seconds: u32,
    ) -> anyhow::Result<String> {
        Ok(format!(
            "https://storage.example.com/download/{}?sig=mock",
            storage_key
        ))
    }

    async fn delete_object(&self, _storage_key: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_object_metadata(
        &self,
        _storage_key: &str,
    ) -> anyhow::Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }
}
