#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use domain::entity::file::FileMetadata;
use domain::repository::{FileMetadataRepository, FileStorageRepository};
use infrastructure::config::Config;
use infrastructure::kafka_producer::{
    FileEventPublisher, FileKafkaProducer, NoopFileEventPublisher,
};
use proto::k1s0::system::file::v1::file_service_server::FileServiceServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-file-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: cfg.app.environment.clone(),
        trace_endpoint: cfg
            .observability
            .trace
            .enabled
            .then(|| cfg.observability.trace.endpoint.clone()),
        sample_rate: cfg.observability.trace.sample_rate,
        log_level: cfg.observability.log.level.clone(),
        log_format: cfg.observability.log.format.clone(),
    };
    k1s0_telemetry::init_telemetry(&telemetry_cfg).expect("failed to init telemetry");

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting file server"
    );

    // Metadata repository (PostgreSQL or InMemory)
    let metadata_repo: Arc<dyn FileMetadataRepository> = if let Some(ref db_cfg) = cfg.database {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_cfg.url.clone());
        info!(
            schema = %db_cfg.schema,
            max_connections = db_cfg.max_connections,
            min_connections = db_cfg.min_connections,
            "initializing PostgreSQL metadata repository"
        );
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_cfg.max_connections)
            .min_connections(db_cfg.min_connections)
            .acquire_timeout(Duration::from_secs(db_cfg.connect_timeout_seconds))
            .connect(&database_url)
            .await?;
        Arc::new(
            infrastructure::file_metadata_postgres::FileMetadataPostgresRepository::new(
                pool,
                &db_cfg.schema,
            )?,
        )
    } else if let Ok(database_url) = std::env::var("DATABASE_URL") {
        info!("DATABASE_URL is set, using PostgreSQL metadata repository");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&database_url)
            .await?;
        Arc::new(
            infrastructure::file_metadata_postgres::FileMetadataPostgresRepository::new(
                pool, "file",
            )?,
        )
    } else {
        info!("no database configured, using in-memory metadata repository");
        Arc::new(InMemoryFileMetadataRepository::new())
    };

    // Storage backend (S3 or InMemory)

    let storage_repo: Arc<dyn FileStorageRepository> = if let Some(ref storage_cfg) = cfg.storage {
        if storage_cfg.backend == "s3" {
            let bucket = storage_cfg
                .bucket
                .clone()
                .unwrap_or_else(|| "k1s0-files".to_string());
            info!(bucket = %bucket, "initializing S3 storage backend");
            Arc::new(
                infrastructure::s3_storage::S3StorageRepository::new(
                    bucket,
                    storage_cfg.region.clone(),
                    storage_cfg.endpoint.clone(),
                )
                .await?,
            )
        } else {
            info!("using in-memory storage backend");
            Arc::new(InMemoryFileStorageRepository::new())
        }
    } else {
        info!("no storage configured, using in-memory storage backend");
        Arc::new(InMemoryFileStorageRepository::new())
    };

    // Kafka publisher
    let publisher: Arc<dyn FileEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        match FileKafkaProducer::new(kafka_cfg) {
            Ok(p) => {
                info!("Kafka file event publisher enabled");
                Arc::new(p)
            }
            Err(e) => {
                tracing::warn!("Failed to create Kafka publisher, using noop: {}", e);
                Arc::new(NoopFileEventPublisher)
            }
        }
    } else {
        info!("no kafka configured, using noop publisher");
        Arc::new(NoopFileEventPublisher)
    };

    // Use cases
    let list_files_uc = Arc::new(usecase::ListFilesUseCase::new(metadata_repo.clone()));
    let generate_upload_url_uc = Arc::new(usecase::GenerateUploadUrlUseCase::new(
        metadata_repo.clone(),
        storage_repo.clone(),
    ));
    let complete_upload_uc = Arc::new(usecase::CompleteUploadUseCase::new(
        metadata_repo.clone(),
        publisher.clone(),
    ));
    let get_file_metadata_uc =
        Arc::new(usecase::GetFileMetadataUseCase::new(metadata_repo.clone()));
    let generate_download_url_uc = Arc::new(usecase::GenerateDownloadUrlUseCase::new(
        metadata_repo.clone(),
        storage_repo.clone(),
    ));
    let delete_file_uc = Arc::new(usecase::DeleteFileUseCase::new(
        metadata_repo.clone(),
        storage_repo.clone(),
        publisher.clone(),
    ));
    let update_file_tags_uc = Arc::new(usecase::UpdateFileTagsUseCase::new(metadata_repo.clone()));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-file-server"));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for file-server");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(adapter::middleware::auth::FileAuthState {
            verifier: jwks_verifier,
        })
    } else {
        info!("no auth configured, file-server running without authentication");
        None
    };

    // REST app state
    let mut state = adapter::handler::AppState {
        list_files_uc: list_files_uc.clone(),
        generate_upload_url_uc: generate_upload_url_uc.clone(),
        complete_upload_uc: complete_upload_uc.clone(),
        get_file_metadata_uc: get_file_metadata_uc.clone(),
        generate_download_url_uc: generate_download_url_uc.clone(),
        delete_file_uc: delete_file_uc.clone(),
        update_file_tags_uc: update_file_tags_uc.clone(),
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    let app =
        adapter::handler::router(state).layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC service
    let grpc_svc = Arc::new(adapter::grpc::FileGrpcService::new(
        get_file_metadata_uc,
        list_files_uc,
        generate_upload_url_uc,
        complete_upload_uc,
        generate_download_url_uc,
        delete_file_uc,
        update_file_tags_uc,
    ));
    let tonic_svc = adapter::grpc::FileServiceTonic::new(grpc_svc);

    // gRPC server
    let grpc_port = cfg.server.grpc_port;
    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", grpc_port).parse()?;
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(FileServiceServer::new(tonic_svc))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app);

    // REST 縺ｨ gRPC 繧剃ｸｦ陦瑚ｵｷ蜍・
    tokio::select! {
        result = rest_future => {
            if let Err(e) = result {
                tracing::error!("REST server error: {}", e);
            }
        }
        result = grpc_future => {
            if let Err(e) = result {
                tracing::error!("gRPC server error: {}", e);
            }
        }
    }

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
        uploaded_by: Option<String>,
        content_type: Option<String>,
        tag: Option<(String, String)>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<FileMetadata>, u64)> {
        let files = self.files.read().await;
        let mut filtered: Vec<FileMetadata> = files
            .values()
            .filter(|f| {
                if let Some(ref tid) = tenant_id {
                    if f.tenant_id != *tid {
                        return false;
                    }
                }
                if let Some(ref uploaded_by) = uploaded_by {
                    if f.uploaded_by != *uploaded_by {
                        return false;
                    }
                }
                if let Some(ref content_type) = content_type {
                    if !f.content_type.starts_with(content_type) {
                        return false;
                    }
                }
                if let Some((ref key, ref value)) = tag {
                    match f.tags.get(key) {
                        Some(v) if v == value => {}
                        _ => return false,
                    }
                }
                true
            })
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let start = page.saturating_sub(1) as usize * page_size as usize;
        filtered = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
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
        _content_type: &str,
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
