use anyhow::Context;
// OpenSearch パスワードを expose_secret() で取り出すために使用する
use secrecy::ExposeSecret;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

// gRPC 認証レイヤー
use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;

use super::cache::IndexCache;
use super::config::Config;
use super::kafka_producer::{KafkaSearchProducer, NoopSearchEventPublisher, SearchEventPublisher};
use crate::adapter::grpc::SearchGrpcService;
use crate::adapter::repository::{
    CachedSearchRepository, SearchOpenSearchRepository, SearchPostgresRepository,
};
use crate::domain::entity::search_index::{
    PaginationResult, SearchDocument, SearchIndex, SearchQuery, SearchResult,
};
use crate::domain::repository::SearchRepository;

pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-search-server".to_string(),
        // Cargo.toml の package.version を使用する（M-16 監査対応: ハードコード解消）
        version: env!("CARGO_PKG_VERSION").to_string(),
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
    k1s0_telemetry::init_telemetry(&telemetry_cfg)
        .map_err(|e| anyhow::anyhow!("テレメトリの初期化に失敗: {}", e))?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting search server"
    );

    // --- Repository: OpenSearch → PostgreSQL → InMemory fallback ---
    let base_search_repo: Arc<dyn SearchRepository> = if let Some(ref os_cfg) = cfg.opensearch {
        info!(url = %os_cfg.url, prefix = %os_cfg.index_prefix, "connecting to OpenSearch for search repository");
        // HIGH-6 監査対応: エラー時にパスワードが含まれる可能性があるため context で上書きする
        // expose_secret() で取り出したパスワードがエラートレースに含まれないようにする
        let repo = SearchOpenSearchRepository::new(
            &os_cfg.url,
            &os_cfg.username,
            // expose_secret() で OpenSearch パスワードを取り出す。接続後は保持しない。
            os_cfg.password.expose_secret(),
            &os_cfg.index_prefix,
            os_cfg.tls_insecure,
        )
        .map_err(|_| anyhow::anyhow!("OpenSearch connection failed (check url/credentials)"))?;
        Arc::new(repo)
    } else {
        let db_url = std::env::var("DATABASE_URL").ok();
        if let Some(url) = db_url {
            info!("connecting to PostgreSQL for search repository");
            let pool = super::database::connect(&url, 10).await?;
            let pool = Arc::new(pool);
            Arc::new(SearchPostgresRepository::new(pool))
        } else {
            // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
            k1s0_server_common::require_infra(
                "search",
                k1s0_server_common::InfraKind::Database,
                &cfg.app.environment,
                None::<String>,
            )?;
            info!("using in-memory search repository (dev/test bypass)");
            Arc::new(InMemorySearchRepository::new())
        }
    };

    // --- Cache: moka (max 1000, TTL 30s) ---
    let index_cache = Arc::new(IndexCache::new(
        cfg.cache.max_entries,
        cfg.cache.ttl_seconds,
    ));
    info!(
        max_entries = cfg.cache.max_entries,
        ttl_seconds = cfg.cache.ttl_seconds,
        "index cache initialized"
    );
    let search_repo: Arc<dyn SearchRepository> = Arc::new(CachedSearchRepository::new(
        base_search_repo,
        index_cache.clone(),
    ));

    // --- Kafka: Producer or Noop fallback ---
    let event_publisher: Arc<dyn SearchEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        let brokers = kafka_cfg.brokers.join(",");
        let topic = "k1s0.system.search.indexed.v1".to_string();
        info!(brokers = %brokers, topic = %topic, "connecting to Kafka");
        let producer = KafkaSearchProducer::new(&brokers, &kafka_cfg.security_protocol, &topic)?;
        Arc::new(producer)
    } else {
        // infra_guard: stable サービスでは Kafka 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "search",
            k1s0_server_common::InfraKind::Kafka,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("using noop event publisher (dev/test bypass)");
        Arc::new(NoopSearchEventPublisher)
    };

    let create_index_uc = Arc::new(crate::usecase::CreateIndexUseCase::new(search_repo.clone()));
    let index_document_uc = Arc::new(crate::usecase::IndexDocumentUseCase::new(
        search_repo.clone(),
        event_publisher.clone(),
    ));
    let search_uc = Arc::new(crate::usecase::SearchUseCase::new(search_repo.clone()));
    let delete_document_uc = Arc::new(crate::usecase::DeleteDocumentUseCase::new(
        search_repo.clone(),
    ));
    let list_indices_uc = Arc::new(crate::usecase::ListIndicesUseCase::new(search_repo));

    // --- Kafka consumer (optional, background task) ---
    if let Some(ref kafka_cfg) = cfg.kafka {
        match super::kafka_consumer::SearchKafkaConsumer::new(
            kafka_cfg,
            index_document_uc.clone(),
            delete_document_uc.clone(),
        ) {
            Ok(consumer) => {
                let consumer = consumer.with_metrics(Arc::new(
                    k1s0_telemetry::metrics::Metrics::new("k1s0-search-server"),
                ));
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
        create_index_uc.clone(),
        list_indices_uc.clone(),
        index_document_uc.clone(),
        search_uc.clone(),
        delete_document_uc.clone(),
    ));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-search-server"));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "search-server",
        &cfg.app.environment,
        cfg.auth.as_ref().map(|auth_cfg| -> anyhow::Result<_> {
            info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for search-server");
            let jwks_verifier = Arc::new(
                k1s0_auth::JwksVerifier::new(
                    &auth_cfg.jwks_url,
                    &auth_cfg.issuer,
                    &auth_cfg.audience,
                    std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
                )
                .context("JWKS 検証器の作成に失敗")?,
            );
            Ok(crate::adapter::middleware::auth::AuthState {
                verifier: jwks_verifier,
            })
        }).transpose()?,
    )?;

    let mut handler_state = crate::adapter::handler::search_handler::AppState {
        search_uc,
        index_document_uc,
        delete_document_uc,
        create_index_uc,
        list_indices_uc,
        metrics: metrics.clone(),
        auth_state: None,
    };
    // gRPC 認証レイヤー用に auth_state を REST への移動前にクローンしておく。
    let grpc_auth_layer = GrpcAuthLayer::new(auth_state.clone(), Tier::System, search_grpc_action);
    if let Some(auth_st) = auth_state {
        handler_state = handler_state.with_auth(auth_st);
    }

    let public_routes = axum::Router::new()
        .route(
            "/healthz",
            axum::routing::get(crate::adapter::handler::health::healthz),
        )
        .route(
            "/readyz",
            axum::routing::get(crate::adapter::handler::health::readyz),
        )
        .route("/metrics", axum::routing::get(metrics_handler));

    let api_routes = if let Some(ref auth_st) = handler_state.auth_state {
        use crate::adapter::middleware::auth::auth_middleware;
        use crate::adapter::middleware::rbac::require_permission;

        // GET indices -> search/read
        let read_routes = axum::Router::new()
            .route(
                "/api/v1/search/indices",
                axum::routing::get(crate::adapter::handler::search_handler::list_indices),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "search", "read",
            )));

        // POST /search -> search/read, POST /index -> search/write, POST /indices -> search/admin
        let search_read_routes = axum::Router::new()
            .route(
                "/api/v1/search",
                axum::routing::post(crate::adapter::handler::search_handler::search),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "search", "read",
            )));

        // POST index -> search/write
        let write_routes = axum::Router::new()
            .route(
                "/api/v1/search/index",
                axum::routing::post(crate::adapter::handler::search_handler::index_document),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "search", "write",
            )));

        // POST /indices -> search/admin
        let index_admin_routes = axum::Router::new()
            .route(
                "/api/v1/search/indices",
                axum::routing::post(crate::adapter::handler::search_handler::create_index),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "search", "admin",
            )));

        // DELETE doc -> search/write
        let admin_routes = axum::Router::new()
            .route(
                "/api/v1/search/index/{index_name}/{id}",
                axum::routing::delete(
                    crate::adapter::handler::search_handler::delete_document_from_index,
                ),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "search", "write",
            )));

        axum::Router::new()
            .merge(read_routes)
            .merge(search_read_routes)
            .merge(write_routes)
            .merge(index_admin_routes)
            .merge(admin_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_st.clone(),
                auth_middleware,
            ))
    } else {
        axum::Router::new()
            .route(
                "/api/v1/search",
                axum::routing::post(crate::adapter::handler::search_handler::search),
            )
            .route(
                "/api/v1/search/index",
                axum::routing::post(crate::adapter::handler::search_handler::index_document),
            )
            .route(
                "/api/v1/search/index/{index_name}/{id}",
                axum::routing::delete(
                    crate::adapter::handler::search_handler::delete_document_from_index,
                ),
            )
            .route(
                "/api/v1/search/indices",
                axum::routing::post(crate::adapter::handler::search_handler::create_index),
            )
            .route(
                "/api/v1/search/indices",
                axum::routing::get(crate::adapter::handler::search_handler::list_indices),
            )
    };

    let app = public_routes
        .merge(api_routes)
        .with_state(handler_state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC server
    use crate::proto::k1s0::system::search::v1::search_service_server::SearchServiceServer;

    let search_tonic = crate::adapter::grpc::SearchServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    // gRPCサーバーのグレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(SearchServiceServer::new(search_tonic))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    // RESTサーバーのグレースフルシャットダウン設定
    let rest_future = axum::serve(listener, app).with_graceful_shutdown(async {
        let _ = k1s0_server_common::shutdown::shutdown_signal().await;
    });

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

    // テレメトリのシャットダウン処理
    k1s0_telemetry::shutdown();

    Ok(())
}

/// gRPC メソッド名から必要な RBAC アクション文字列を返す。
/// CreateIndex / IndexDocument / DeleteDocument は write、それ以外は read。
fn search_grpc_action(method: &str) -> &'static str {
    match method {
        "CreateIndex" | "IndexDocument" | "DeleteDocument" => "write",
        _ => "read",
    }
}

async fn metrics_handler(
    axum::extract::State(state): axum::extract::State<
        crate::adapter::handler::search_handler::AppState,
    >,
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
        let docs = documents
            .get(&query.index_name)
            .cloned()
            .unwrap_or_default();

        let matched: Vec<SearchDocument> =
            docs.into_iter()
                .filter(|doc| {
                    let content_str = doc.content.to_string();
                    let query_ok = query.query.is_empty() || content_str.contains(&query.query);
                    if !query_ok {
                        return false;
                    }
                    query.filters.iter().all(|(k, v)| {
                        doc.content.get(k).and_then(|x| x.as_str()) == Some(v.as_str())
                    })
                })
                .map(|mut doc| {
                    doc.score = 1.0;
                    doc
                })
                .collect();

        let total = matched.len() as u64;
        let hits: Vec<SearchDocument> = matched
            .into_iter()
            .skip(query.from as usize)
            .take(query.size as usize)
            .collect();
        let page_size = query.size.max(1);
        let page = (query.from / page_size) + 1;
        let has_next = total > (query.from as u64 + hits.len() as u64);

        Ok(SearchResult {
            total,
            hits,
            facets: HashMap::new(),
            pagination: PaginationResult {
                total_count: total,
                page,
                page_size,
                has_next,
            },
        })
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
