#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

mod adapter;
mod domain;
mod usecase;

use domain::entity::search_index::{SearchDocument, SearchIndex, SearchQuery, SearchResult};
use domain::repository::SearchRepository;

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
    8094
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-search-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string()),
        trace_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        sample_rate: 1.0,
        log_level: "info".to_string(),
    };
    k1s0_telemetry::init_telemetry(&telemetry_cfg).expect("failed to init telemetry");

    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting search server"
    );

    let search_repo: Arc<dyn SearchRepository> = Arc::new(InMemorySearchRepository::new());

    let _create_index_uc = Arc::new(usecase::CreateIndexUseCase::new(search_repo.clone()));
    let _index_document_uc = Arc::new(usecase::IndexDocumentUseCase::new(search_repo.clone()));
    let _search_uc = Arc::new(usecase::SearchUseCase::new(search_repo.clone()));
    let _delete_document_uc = Arc::new(usecase::DeleteDocumentUseCase::new(search_repo));

    let app = axum::Router::new()
        .route("/healthz", axum::routing::get(adapter::handler::health::healthz))
        .route("/readyz", axum::routing::get(adapter::handler::health::readyz));

    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// --- InMemory Repository ---

struct InMemorySearchRepository {
    indices: tokio::sync::RwLock<HashMap<String, SearchIndex>>,
    documents: tokio::sync::RwLock<HashMap<String, Vec<SearchDocument>>>,
}

impl InMemorySearchRepository {
    fn new() -> Self {
        Self {
            indices: tokio::sync::RwLock::new(HashMap::new()),
            documents: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl SearchRepository for InMemorySearchRepository {
    async fn create_index(&self, index: &SearchIndex) -> anyhow::Result<()> {
        let mut indices = self.indices.write().await;
        indices.insert(index.name.clone(), index.clone());
        Ok(())
    }

    async fn find_index(&self, name: &str) -> anyhow::Result<Option<SearchIndex>> {
        let indices = self.indices.read().await;
        Ok(indices.get(name).cloned())
    }

    async fn index_document(&self, doc: &SearchDocument) -> anyhow::Result<()> {
        let mut documents = self.documents.write().await;
        documents
            .entry(doc.index_name.clone())
            .or_default()
            .push(doc.clone());
        Ok(())
    }

    async fn search(&self, query: &SearchQuery) -> anyhow::Result<SearchResult> {
        let documents = self.documents.read().await;
        let docs = documents.get(&query.index_name).cloned().unwrap_or_default();

        let hits: Vec<SearchDocument> = docs
            .into_iter()
            .filter(|doc| {
                let content_str = doc.content.to_string();
                content_str.contains(&query.query)
            })
            .skip(query.from as usize)
            .take(query.size as usize)
            .collect();

        let total = hits.len() as u64;
        Ok(SearchResult { total, hits })
    }

    async fn delete_document(&self, index_name: &str, doc_id: &str) -> anyhow::Result<bool> {
        let mut documents = self.documents.write().await;
        if let Some(docs) = documents.get_mut(index_name) {
            let len_before = docs.len();
            docs.retain(|d| d.id != doc_id);
            Ok(docs.len() < len_before)
        } else {
            Ok(false)
        }
    }
}
