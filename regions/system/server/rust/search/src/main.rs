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
use adapter::repository::{SearchOpenSearchRepository, SearchPostgresRepository};
use domain::entity::search_index::{SearchDocument, SearchIndex, SearchQuery, SearchResult};
use domain::repository::SearchRepository;
use infrastructure::cache::IndexCache;
use infrastructure::config::Config;
use infrastructure::kafka_producer::{
    KafkaSearchProducer, NoopSearchEventPublisher, SearchEventPublisher,
};

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

    // --- Repository: OpenSearch → PostgreSQL → InMemory fallback ---
    let search_repo: Arc<dyn SearchRepository> = if let Some(ref os_cfg) = cfg.opensearch {
        info!(url = %os_cfg.url, prefix = %os_cfg.index_prefix, "connecting to OpenSearch for search repository");
        let repo = SearchOpenSearchRepository::new(
            &os_cfg.url,
            &os_cfg.username,
            &os_cfg.password,
            &os_cfg.index_prefix,
        )?;
        Arc::new(repo)
    } else {
        let db_url = std::env::var("DATABASE_URL").ok();
        if let Some(url) = db_url {
            info!("connecting to PostgreSQL for search repository");
            let pool = infrastructure::database::connect(&url, 10).await?;
            let pool = Arc::new(pool);
            Arc::new(SearchPostgresRepository::new(pool))
        } else {
            info!("using in-memory search repository (DATABASE_URL not set)");
            Arc::new(InMemorySearchRepository::new())
        }
    };

    // --- Cache: moka (max 1000, TTL 30s) ---
    let _index_cache = Arc::new(IndexCache::new(
        cfg.cache.max_entries,
        cfg.cache.ttl_seconds,
    ));
    info!(
        max_entries = cfg.cache.max_entries,
        ttl_seconds = cfg.cache.ttl_seconds,
        "index cache initialized"
    );

    // --- Kafka: Producer or Noop fallback ---
    let _event_publisher: Arc<dyn SearchEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        let brokers = kafka_cfg.brokers.join(",");
        let topic = "k1s0.system.search.indexed.v1".to_string();
        info!(brokers = %brokers, topic = %topic, "connecting to Kafka");
        let producer = KafkaSearchProducer::new(
            &brokers,
            &kafka_cfg.security_protocol,
            &topic,
        )?;
        Arc::new(producer)
    } else {
        info!("using noop event publisher (kafka not configured)");
        Arc::new(NoopSearchEventPublisher)
    };

    let create_index_uc = Arc::new(usecase::CreateIndexUseCase::new(search_repo.clone()));
    let index_document_uc = Arc::new(usecase::IndexDocumentUseCase::new(search_repo.clone()));
    let search_uc = Arc::new(usecase::SearchUseCase::new(search_repo.clone()));
    let delete_document_uc = Arc::new(usecase::DeleteDocumentUseCase::new(search_repo.clone()));
    let list_indices_uc = Arc::new(usecase::ListIndicesUseCase::new(search_repo));

    // --- Kafka consumer (optional, background task) ---
    if let Some(ref kafka_cfg) = cfg.kafka {
        match infrastructure::kafka_consumer::SearchKafkaConsumer::new(
            kafka_cfg,
            index_document_uc.clone(),
        ) {
            Ok(consumer) => {
                let consumer = consumer.with_metrics(
                    Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-search-server")),
                );
                info!("kafka consumer initialized, starting background ingestion");
                tokio::spawn(async move {
                    if let Err(e) = consumer.run().await {
                        tracing::error!(error = %e, "kafka consumer stopped with error");
                    }
                });
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to create kafka consumer");
            }
        }
    }

    let grpc_svc = Arc::new(SearchGrpcService::new(
        index_document_uc.clone(),
        search_uc.clone(),
        delete_document_uc.clone(),
    ));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-search-server",
    ));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for search-server");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(adapter::middleware::auth::SearchAuthState {
            verifier: jwks_verifier,
        })
    } else {
        info!("no auth configured, search-server running without authentication");
        None
    };

    let mut handler_state = adapter::handler::search_handler::AppState {
        search_uc,
        index_document_uc,
        delete_document_uc,
        create_index_uc,
        list_indices_uc,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        handler_state = handler_state.with_auth(auth_st);
    }

    let public_routes = axum::Router::new()
        .route("/healthz", axum::routing::get(adapter::handler::health::healthz))
        .route("/readyz", axum::routing::get(adapter::handler::health::readyz))
        .route("/metrics", axum::routing::get(metrics_handler));

    let api_routes = if let Some(ref auth_st) = handler_state.auth_state {
        use adapter::middleware::auth::auth_middleware;
        use adapter::middleware::rbac::require_permission;

        // GET indices -> search/read
        let read_routes = axum::Router::new()
            .route(
                "/api/v1/search/indices",
                axum::routing::get(adapter::handler::search_handler::list_indices),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "search", "read",
            )));

        // POST search/index -> search/write
        let write_routes = axum::Router::new()
            .route(
                "/api/v1/search",
                axum::routing::post(adapter::handler::search_handler::search),
            )
            .route(
                "/api/v1/search/index",
                axum::routing::post(adapter::handler::search_handler::index_document),
            )
            .route(
                "/api/v1/search/indices",
                axum::routing::post(adapter::handler::search_handler::create_index),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "search", "write",
            )));

        // DELETE doc -> search/admin
        let admin_routes = axum::Router::new()
            .route(
                "/api/v1/search/index/:index_name/:id",
                axum::routing::delete(adapter::handler::search_handler::delete_document_from_index),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "search", "admin",
            )));

        axum::Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .merge(admin_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_st.clone(),
                auth_middleware,
            ))
    } else {
        axum::Router::new()
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
            .route(
                "/api/v1/search/indices",
                axum::routing::post(adapter::handler::search_handler::create_index),
            )
            .route(
                "/api/v1/search/indices",
                axum::routing::get(adapter::handler::search_handler::list_indices),
            )
    };

    let app = public_routes
        .merge(api_routes)
        .with_state(handler_state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC server
    use proto::k1s0::system::search::v1::search_service_server::SearchServiceServer;

    let search_tonic = adapter::grpc::SearchServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 50051).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
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

async fn metrics_handler(
    axum::extract::State(state): axum::extract::State<adapter::handler::search_handler::AppState>,
) -> impl axum::response::IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        axum::http::StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
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

    async fn list_indices(&self) -> anyhow::Result<Vec<SearchIndex>> {
        let indices = self.indices.read().await;
        Ok(indices.values().cloned().collect())
    }
}
