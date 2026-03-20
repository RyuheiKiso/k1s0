use anyhow::Context;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tracing::info;
use uuid::Uuid;

use crate::adapter;
use crate::infrastructure;
use crate::proto;
use crate::usecase;

use super::config::Config;
use super::kafka_producer::{KafkaRuleProducer, NoopRuleEventPublisher, RuleEventPublisher};
use crate::adapter::grpc::RuleEngineGrpcService;
use crate::domain::entity::rule::{EvaluationLog, Rule, RuleSet, RuleSetVersion};
use crate::domain::repository::{
    EvaluationLogRepository, RuleRepository, RuleSetRepository, RuleSetVersionRepository,
};

pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-rule-engine-server".to_string(),
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
    k1s0_telemetry::init_telemetry(&telemetry_cfg)
        .map_err(|e| anyhow::anyhow!("テレメトリの初期化に失敗: {}", e))?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting rule-engine server"
    );

    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-rule-engine-server",
    ));

    let _cache = Arc::new(infrastructure::cache::RuleCache::new(
        cfg.cache.max_entries,
        cfg.cache.ttl_seconds,
    ));

    // Repositories: InMemory fallback (PostgreSQL would be similar to policy-server)
    // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
    k1s0_server_common::require_infra(
        "rule-engine",
        k1s0_server_common::InfraKind::Database,
        &cfg.app.environment,
        None::<String>,
    )?;
    let rule_repo: Arc<dyn RuleRepository> = Arc::new(InMemoryRuleRepository::new());
    let rule_set_repo: Arc<dyn RuleSetRepository> = Arc::new(InMemoryRuleSetRepository::new());
    let version_repo: Arc<dyn RuleSetVersionRepository> =
        Arc::new(InMemoryRuleSetVersionRepository::new());
    let eval_log_repo: Arc<dyn EvaluationLogRepository> =
        Arc::new(InMemoryEvaluationLogRepository::new());

    // Kafka event publisher
    let event_publisher: Arc<dyn RuleEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        info!(
            brokers = %kafka_cfg.brokers.join(","),
            topic = %kafka_cfg.topic,
            "initializing Kafka rule event publisher"
        );
        let producer = KafkaRuleProducer::new(kafka_cfg)?.with_metrics(metrics.clone());
        Arc::new(producer)
    } else {
        info!("no Kafka configured, using no-op event publisher");
        Arc::new(NoopRuleEventPublisher)
    };

    // Token verifier
    let auth_state = k1s0_server_common::require_auth_state(
        "rule-engine-server",
        &cfg.app.environment,
        cfg.auth.as_ref().map(|auth_cfg| -> anyhow::Result<_> {
            info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for rule-engine-server");
            let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
                &auth_cfg.jwks_url,
                &auth_cfg.issuer,
                &auth_cfg.audience,
                std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
            ).context("JWKS 検証器の作成に失敗")?);
            Ok(adapter::middleware::auth::AuthState {
                verifier: jwks_verifier,
            })
        }).transpose()?,
    )?;

    let create_rule_uc = Arc::new(usecase::CreateRuleUseCase::with_publisher(
        rule_repo.clone(),
        event_publisher.clone(),
    ));
    let get_rule_uc = Arc::new(usecase::GetRuleUseCase::new(rule_repo.clone()));
    let update_rule_uc = Arc::new(usecase::UpdateRuleUseCase::with_publisher(
        rule_repo.clone(),
        event_publisher.clone(),
    ));
    let delete_rule_uc = Arc::new(usecase::DeleteRuleUseCase::with_publisher(
        rule_repo.clone(),
        event_publisher.clone(),
    ));
    let list_rules_uc = Arc::new(usecase::ListRulesUseCase::new(rule_repo.clone()));

    let create_rule_set_uc = Arc::new(usecase::CreateRuleSetUseCase::with_publisher(
        rule_set_repo.clone(),
        event_publisher.clone(),
    ));
    let get_rule_set_uc = Arc::new(usecase::GetRuleSetUseCase::new(rule_set_repo.clone()));
    let update_rule_set_uc = Arc::new(usecase::UpdateRuleSetUseCase::with_publisher(
        rule_set_repo.clone(),
        event_publisher.clone(),
    ));
    let delete_rule_set_uc = Arc::new(usecase::DeleteRuleSetUseCase::with_publisher(
        rule_set_repo.clone(),
        event_publisher.clone(),
    ));
    let list_rule_sets_uc = Arc::new(usecase::ListRuleSetsUseCase::new(rule_set_repo.clone()));

    let publish_rule_set_uc = Arc::new(usecase::PublishRuleSetUseCase::with_publisher(
        rule_set_repo.clone(),
        version_repo.clone(),
        event_publisher.clone(),
    ));
    let rollback_rule_set_uc = Arc::new(usecase::RollbackRuleSetUseCase::with_publisher(
        rule_set_repo.clone(),
        version_repo.clone(),
        event_publisher,
    ));

    let evaluate_uc = Arc::new(usecase::EvaluateUseCase::new(
        rule_set_repo.clone(),
        rule_repo.clone(),
        eval_log_repo.clone(),
    ));
    let list_evaluation_logs_uc = Arc::new(usecase::ListEvaluationLogsUseCase::new(eval_log_repo));

    let grpc_svc = Arc::new(RuleEngineGrpcService::new(
        create_rule_uc.clone(),
        get_rule_uc.clone(),
        update_rule_uc.clone(),
        delete_rule_uc.clone(),
        list_rules_uc.clone(),
        create_rule_set_uc.clone(),
        get_rule_set_uc.clone(),
        update_rule_set_uc.clone(),
        delete_rule_set_uc.clone(),
        list_rule_sets_uc.clone(),
        publish_rule_set_uc.clone(),
        rollback_rule_set_uc.clone(),
        evaluate_uc.clone(),
    ));

    // バックエンド種別を health エンドポイントで返すために設定
    // 現在は in-memory リポジトリのみ。database 構成追加時に "postgres" へ変更すること。
    let backend_kind = "in-memory".to_string();

    let mut state = adapter::handler::AppState {
        create_rule_uc,
        get_rule_uc,
        list_rules_uc,
        update_rule_uc,
        delete_rule_uc,
        create_rule_set_uc,
        get_rule_set_uc,
        list_rule_sets_uc,
        update_rule_set_uc,
        delete_rule_set_uc,
        publish_rule_set_uc,
        rollback_rule_set_uc,
        evaluate_uc,
        list_evaluation_logs_uc,
        metrics: metrics.clone(),
        auth_state: None,
        backend_kind,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    let app = adapter::handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC server
    use proto::k1s0::system::rule_engine::v1::rule_engine_service_server::RuleEngineServiceServer;

    let grpc_tonic = adapter::grpc::RuleEngineServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    // gRPCサーバーのグレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(RuleEngineServiceServer::new(grpc_tonic))
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

// --- InMemoryRuleRepository ---

struct InMemoryRuleRepository {
    rules: tokio::sync::RwLock<HashMap<Uuid, Rule>>,
}

impl InMemoryRuleRepository {
    fn new() -> Self {
        Self {
            rules: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl RuleRepository for InMemoryRuleRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Rule>> {
        let rules = self.rules.read().await;
        Ok(rules.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Rule>> {
        let rules = self.rules.read().await;
        Ok(rules.values().cloned().collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        _rule_set_id: Option<Uuid>,
        _domain: Option<String>,
    ) -> anyhow::Result<(Vec<Rule>, u64)> {
        let rules = self.rules.read().await;
        let mut all: Vec<Rule> = rules.values().cloned().collect();
        all.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let total = all.len() as u64;
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let items: Vec<Rule> = all
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((items, total))
    }

    async fn create(&self, rule: &Rule) -> anyhow::Result<()> {
        let mut rules = self.rules.write().await;
        rules.insert(rule.id, rule.clone());
        Ok(())
    }

    async fn update(&self, rule: &Rule) -> anyhow::Result<()> {
        let mut rules = self.rules.write().await;
        rules.insert(rule.id, rule.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut rules = self.rules.write().await;
        Ok(rules.remove(id).is_some())
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        let rules = self.rules.read().await;
        Ok(rules.values().any(|r| r.name == name))
    }

    async fn find_by_ids(&self, ids: &[Uuid]) -> anyhow::Result<Vec<Rule>> {
        let rules = self.rules.read().await;
        Ok(ids.iter().filter_map(|id| rules.get(id).cloned()).collect())
    }
}

// --- InMemoryRuleSetRepository ---

struct InMemoryRuleSetRepository {
    rule_sets: tokio::sync::RwLock<HashMap<Uuid, RuleSet>>,
}

impl InMemoryRuleSetRepository {
    fn new() -> Self {
        Self {
            rule_sets: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl RuleSetRepository for InMemoryRuleSetRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<RuleSet>> {
        let sets = self.rule_sets.read().await;
        Ok(sets.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<RuleSet>> {
        let sets = self.rule_sets.read().await;
        Ok(sets.values().cloned().collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        domain: Option<String>,
    ) -> anyhow::Result<(Vec<RuleSet>, u64)> {
        let sets = self.rule_sets.read().await;
        let mut all: Vec<RuleSet> = sets
            .values()
            .filter(|rs| {
                if let Some(ref d) = domain {
                    rs.domain == *d
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        all.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let total = all.len() as u64;
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let items: Vec<RuleSet> = all
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((items, total))
    }

    async fn find_by_domain_and_name(
        &self,
        domain: &str,
        name: &str,
    ) -> anyhow::Result<Option<RuleSet>> {
        let sets = self.rule_sets.read().await;
        Ok(sets
            .values()
            .find(|rs| rs.domain == domain && rs.name == name)
            .cloned())
    }

    async fn create(&self, rule_set: &RuleSet) -> anyhow::Result<()> {
        let mut sets = self.rule_sets.write().await;
        sets.insert(rule_set.id, rule_set.clone());
        Ok(())
    }

    async fn update(&self, rule_set: &RuleSet) -> anyhow::Result<()> {
        let mut sets = self.rule_sets.write().await;
        sets.insert(rule_set.id, rule_set.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut sets = self.rule_sets.write().await;
        Ok(sets.remove(id).is_some())
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        let sets = self.rule_sets.read().await;
        Ok(sets.values().any(|rs| rs.name == name))
    }
}

// --- InMemoryRuleSetVersionRepository ---

struct InMemoryRuleSetVersionRepository {
    versions: tokio::sync::RwLock<Vec<RuleSetVersion>>,
}

impl InMemoryRuleSetVersionRepository {
    fn new() -> Self {
        Self {
            versions: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl RuleSetVersionRepository for InMemoryRuleSetVersionRepository {
    async fn find_by_rule_set_id_and_version(
        &self,
        rule_set_id: &Uuid,
        version: u32,
    ) -> anyhow::Result<Option<RuleSetVersion>> {
        let versions = self.versions.read().await;
        Ok(versions
            .iter()
            .find(|v| v.rule_set_id == *rule_set_id && v.version == version)
            .cloned())
    }

    async fn find_latest_by_rule_set_id(
        &self,
        rule_set_id: &Uuid,
    ) -> anyhow::Result<Option<RuleSetVersion>> {
        let versions = self.versions.read().await;
        Ok(versions
            .iter()
            .filter(|v| v.rule_set_id == *rule_set_id)
            .max_by_key(|v| v.version)
            .cloned())
    }

    async fn create(&self, version: &RuleSetVersion) -> anyhow::Result<()> {
        let mut versions = self.versions.write().await;
        versions.push(version.clone());
        Ok(())
    }
}

// --- InMemoryEvaluationLogRepository ---

struct InMemoryEvaluationLogRepository {
    logs: tokio::sync::RwLock<Vec<EvaluationLog>>,
}

impl InMemoryEvaluationLogRepository {
    fn new() -> Self {
        Self {
            logs: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl EvaluationLogRepository for InMemoryEvaluationLogRepository {
    async fn create(&self, log: &EvaluationLog) -> anyhow::Result<()> {
        let mut logs = self.logs.write().await;
        logs.push(log.clone());
        Ok(())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        rule_set_name: Option<String>,
        _domain: Option<String>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> anyhow::Result<(Vec<EvaluationLog>, u64)> {
        let logs = self.logs.read().await;
        let mut filtered: Vec<EvaluationLog> = logs
            .iter()
            .filter(|l| {
                if let Some(ref name) = rule_set_name {
                    if !l.rule_set_name.contains(name.as_str()) {
                        return false;
                    }
                }
                if let Some(ref f) = from {
                    if l.evaluated_at < *f {
                        return false;
                    }
                }
                if let Some(ref t) = to {
                    if l.evaluated_at > *t {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.evaluated_at.cmp(&a.evaluated_at));
        let total = filtered.len() as u64;
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let items: Vec<EvaluationLog> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((items, total))
    }
}
