#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::SearchGrpcService;
use domain::entity::search_index::{SearchDocument, SearchIndex, SearchQuery, SearchResult};
use domain::repository::SearchRepository;
use infrastructure::config::Config;

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
    let cfg = Config::load(&config_path)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting search server"
    );

    let search_repo: Arc<dyn SearchRepository> = Arc::new(InMemorySearchRepository::new());

    let _create_index_uc = Arc::new(usecase::CreateIndexUseCase::new(search_repo.clone()));
    let index_document_uc = Arc::new(usecase::IndexDocumentUseCase::new(search_repo.clone()));
    let search_uc = Arc::new(usecase::SearchUseCase::new(search_repo.clone()));
    let delete_document_uc = Arc::new(usecase::DeleteDocumentUseCase::new(search_repo));

    let grpc_svc = Arc::new(SearchGrpcService::new(
        index_document_uc.clone(),
        search_uc.clone(),
        delete_document_uc.clone(),
    ));

    let handler_state = adapter::handler::search_handler::AppState {
        search_uc,
        index_document_uc,
        delete_document_uc,
    };

    let app = axum::Router::new()
        .route("/healthz", axum::routing::get(adapter::handler::health::healthz))
        .route("/readyz", axum::routing::get(adapter::handler::health::readyz))
        .route(
            "/api/v1/search",
            axum::routing::post(adapter::handler::search_handler::search),
        )
        .route(
            "/api/v1/search/index",
            axum::routing::post(adapter::handler::search_handler::index_document),
        )
        .route(
            "/api/v1/search/index/:index_name/:id",
            axum::routing::delete(adapter::handler::search_handler::delete_document_from_index),
        )
        .with_state(handler_state);

    // gRPC server
    use proto::k1s0::system::search::v1::search_service_server::SearchServiceServer;

    let search_tonic = adapter::grpc::SearchServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 50051).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_future = async move {
        tonic::transport::Server::builder()
            .add_service(SearchServiceServer::new(search_tonic))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app);

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
