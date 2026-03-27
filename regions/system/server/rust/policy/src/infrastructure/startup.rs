use anyhow::Context;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

// gRPC 認証レイヤー
use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;

use crate::adapter;
use crate::infrastructure;
use crate::proto;
use crate::usecase;

use super::cache::PolicyCache;
use super::config::Config;
use super::kafka_producer::{KafkaPolicyProducer, NoopPolicyEventPublisher, PolicyEventPublisher};
use super::opa_client::OpaClient;
use crate::adapter::grpc::PolicyGrpcService;
use crate::adapter::repository::bundle_postgres::BundlePostgresRepository;
use crate::adapter::repository::cached_policy_repository::CachedPolicyRepository;
use crate::adapter::repository::policy_postgres::PolicyPostgresRepository;
use crate::domain::entity::policy::Policy;
use crate::domain::entity::policy_bundle::PolicyBundle;
use crate::domain::repository::PolicyBundleRepository;
use crate::domain::repository::PolicyRepository;

pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-policy-server".to_string(),
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
        "starting policy server"
    );

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-policy-server"));

    // Cache
    let cache = Arc::new(PolicyCache::new(
        cfg.cache.max_entries,
        cfg.cache.ttl_seconds,
    ));

    // Repositories: PostgreSQL or InMemory fallback
    let (policy_repo, bundle_repo): (Arc<dyn PolicyRepository>, Arc<dyn PolicyBundleRepository>) =
        if let Some(ref db_cfg) = cfg.database {
            info!(
                "connecting to PostgreSQL: {}:{}/{}",
                db_cfg.host, db_cfg.port, db_cfg.name
            );
            let pool = Arc::new(infrastructure::database::connect(db_cfg).await?);
            info!("PostgreSQL connection established");

            let pg_policy_repo: Arc<dyn PolicyRepository> =
                Arc::new(PolicyPostgresRepository::new(pool.clone()));
            let cached_policy_repo: Arc<dyn PolicyRepository> =
                Arc::new(CachedPolicyRepository::new(pg_policy_repo, cache.clone()));

            let bundle_repo: Arc<dyn PolicyBundleRepository> =
                Arc::new(BundlePostgresRepository::new(pool));

            (cached_policy_repo, bundle_repo)
        } else {
            // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
            k1s0_server_common::require_infra(
                "policy",
                k1s0_server_common::InfraKind::Database,
                &cfg.app.environment,
                None::<String>,
            )?;
            info!("no database configured, using in-memory repositories (dev/test bypass)");
            let policy_repo: Arc<dyn PolicyRepository> = Arc::new(InMemoryPolicyRepository::new());
            let bundle_repo: Arc<dyn PolicyBundleRepository> =
                Arc::new(InMemoryPolicyBundleRepository::new());
            (policy_repo, bundle_repo)
        };

    // Kafka event publisher
    let event_publisher: Arc<dyn PolicyEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        info!(
            brokers = %kafka_cfg.brokers.join(","),
            topic = %kafka_cfg.topic,
            "initializing Kafka policy event publisher"
        );
        let producer = KafkaPolicyProducer::new(kafka_cfg)?.with_metrics(metrics.clone());
        Arc::new(producer)
    } else {
        info!("no Kafka configured, using no-op event publisher");
        Arc::new(NoopPolicyEventPublisher)
    };

    // OPA client
    let opa_client: Option<Arc<OpaClient>> = if let Some(ref opa_cfg) = cfg.opa {
        info!(url = %opa_cfg.url, timeout_ms = %opa_cfg.timeout_ms, "initializing OPA client");
        Some(Arc::new(OpaClient::new(opa_cfg)?))
    } else {
        info!("no OPA configured, using policy.enabled fallback");
        None
    };

    let create_policy_uc = Arc::new(usecase::CreatePolicyUseCase::with_publisher(
        policy_repo.clone(),
        event_publisher.clone(),
    ));
    let get_policy_uc = Arc::new(usecase::GetPolicyUseCase::new(policy_repo.clone()));
    let update_policy_uc = Arc::new(usecase::UpdatePolicyUseCase::with_publisher(
        policy_repo.clone(),
        event_publisher.clone(),
    ));
    let delete_policy_uc = Arc::new(usecase::DeletePolicyUseCase::with_publisher(
        policy_repo.clone(),
        event_publisher,
    ));
    let list_policies_uc = Arc::new(usecase::ListPoliciesUseCase::new(policy_repo.clone()));
    let evaluate_policy_uc = Arc::new(usecase::EvaluatePolicyUseCase::new(
        policy_repo.clone(),
        opa_client,
    ));
    let create_bundle_uc = Arc::new(usecase::CreateBundleUseCase::new(bundle_repo.clone()));
    let get_bundle_uc = Arc::new(usecase::GetBundleUseCase::new(bundle_repo.clone()));
    let list_bundles_uc = Arc::new(usecase::ListBundlesUseCase::new(bundle_repo));

    let grpc_svc = Arc::new(PolicyGrpcService::new(
        create_policy_uc.clone(),
        get_policy_uc.clone(),
        update_policy_uc.clone(),
        delete_policy_uc.clone(),
        list_policies_uc.clone(),
        evaluate_policy_uc.clone(),
        create_bundle_uc.clone(),
        get_bundle_uc.clone(),
        list_bundles_uc.clone(),
    ));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "policy-server",
        &cfg.app.environment,
        cfg.auth.as_ref().map(|auth_cfg| -> anyhow::Result<_> {
            info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for policy-server");
            let jwks_verifier = Arc::new(
                k1s0_auth::JwksVerifier::new(
                    &auth_cfg.jwks_url,
                    &auth_cfg.issuer,
                    &auth_cfg.audience,
                    std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
                )
                .context("JWKS 検証器の作成に失敗")?,
            );
            Ok(adapter::middleware::auth::AuthState {
                verifier: jwks_verifier,
            })
        }).transpose()?,
    )?;

    // バックエンド種別を health エンドポイントで返すために判定
    let backend_kind = if cfg.database.is_some() {
        "postgres".to_string()
    } else {
        "in-memory".to_string()
    };

    let mut state = adapter::handler::AppState {
        create_policy_uc,
        get_policy_uc,
        list_policies_uc,
        update_policy_uc,
        delete_policy_uc,
        evaluate_policy_uc,
        create_bundle_uc,
        get_bundle_uc,
        list_bundles_uc,
        metrics: metrics.clone(),
        auth_state: None,
        backend_kind,
    };
    // gRPC 認証レイヤー用に auth_state を REST への移動前にクローンしておく。
    let grpc_auth_layer = GrpcAuthLayer::new(auth_state.clone(), Tier::System, policy_grpc_action);
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    let app = adapter::handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC server
    use proto::k1s0::system::policy::v1::policy_service_server::PolicyServiceServer;

    let policy_tonic = adapter::grpc::PolicyServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    // gRPC グレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(PolicyServiceServer::new(policy_tonic))
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
    // REST グレースフルシャットダウンを設定
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

    // テレメトリのシャットダウン処理を実行
    k1s0_telemetry::shutdown();

    Ok(())
}

/// gRPC メソッド名から必要な RBAC アクション文字列を返す。
/// CreatePolicy / UpdatePolicy / DeletePolicy / CreateBundle は write、それ以外は read。
fn policy_grpc_action(method: &str) -> &'static str {
    match method {
        "CreatePolicy" | "UpdatePolicy" | "DeletePolicy" | "CreateBundle" => "write",
        _ => "read",
    }
}

// --- InMemoryPolicyRepository ---

struct InMemoryPolicyRepository {
    policies: tokio::sync::RwLock<HashMap<Uuid, Policy>>,
}

impl InMemoryPolicyRepository {
    fn new() -> Self {
        Self {
            policies: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl PolicyRepository for InMemoryPolicyRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Policy>> {
        let policies = self.policies.read().await;
        Ok(policies.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Policy>> {
        let policies = self.policies.read().await;
        Ok(policies.values().cloned().collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        bundle_id: Option<Uuid>,
        enabled_only: bool,
    ) -> anyhow::Result<(Vec<Policy>, u64)> {
        let policies = self.policies.read().await;
        let mut filtered: Vec<Policy> = policies
            .values()
            .filter(|p| {
                if enabled_only && !p.enabled {
                    return false;
                }
                if let Some(ref bid) = bundle_id {
                    if p.bundle_id.as_ref() != Some(bid) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let total = filtered.len() as u64;
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let items: Vec<Policy> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((items, total))
    }

    async fn create(&self, policy: &Policy) -> anyhow::Result<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id, policy.clone());
        Ok(())
    }

    async fn update(&self, policy: &Policy) -> anyhow::Result<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id, policy.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut policies = self.policies.write().await;
        Ok(policies.remove(id).is_some())
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        let policies = self.policies.read().await;
        Ok(policies.values().any(|p| p.name == name))
    }
}

// --- InMemoryPolicyBundleRepository ---

struct InMemoryPolicyBundleRepository {
    bundles: tokio::sync::RwLock<HashMap<Uuid, PolicyBundle>>,
}

impl InMemoryPolicyBundleRepository {
    fn new() -> Self {
        Self {
            bundles: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl PolicyBundleRepository for InMemoryPolicyBundleRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<PolicyBundle>> {
        let bundles = self.bundles.read().await;
        Ok(bundles.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<PolicyBundle>> {
        let bundles = self.bundles.read().await;
        Ok(bundles.values().cloned().collect())
    }

    async fn create(&self, bundle: &PolicyBundle) -> anyhow::Result<()> {
        let mut bundles = self.bundles.write().await;
        bundles.insert(bundle.id, bundle.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut bundles = self.bundles.write().await;
        Ok(bundles.remove(id).is_some())
    }
}
