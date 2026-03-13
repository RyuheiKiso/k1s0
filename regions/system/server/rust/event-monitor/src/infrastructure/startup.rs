use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tracing::info;
use uuid::Uuid;

use crate::adapter;
use crate::adapter::grpc::EventMonitorGrpcService;
use crate::adapter::repository::event_record_postgres::EventRecordPostgresRepository;
use crate::adapter::repository::flow_definition_postgres::FlowDefinitionPostgresRepository;
use crate::adapter::repository::flow_instance_postgres::FlowInstancePostgresRepository;
use crate::domain::entity::event_record::EventRecord;
use crate::domain::entity::flow_definition::FlowDefinition;
use crate::domain::entity::flow_instance::{FlowInstance, FlowInstanceStatus};
use crate::domain::repository::{EventRecordRepository, FlowDefinitionRepository, FlowInstanceRepository};
use super::config::Config;
use super::dlq_client::{DlqManagerClient, NoopDlqClient};
use crate::usecase;

pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-event-monitor-server".to_string(),
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
        "starting event-monitor server"
    );

    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-event-monitor-server",
    ));

    // Repositories: PostgreSQL or InMemory fallback
    let (event_repo, flow_def_repo, flow_inst_repo): (
        Arc<dyn EventRecordRepository>,
        Arc<dyn FlowDefinitionRepository>,
        Arc<dyn FlowInstanceRepository>,
    ) = if let Some(ref db_cfg) = cfg.database {
        info!(
            "connecting to PostgreSQL: {}:{}/{}",
            db_cfg.host, db_cfg.port, db_cfg.name
        );
        let pool = Arc::new(super::database::connect(db_cfg).await?);
        info!("PostgreSQL connection established");

        let event_repo: Arc<dyn EventRecordRepository> =
            Arc::new(EventRecordPostgresRepository::new(pool.clone()));
        let flow_def_repo: Arc<dyn FlowDefinitionRepository> =
            Arc::new(FlowDefinitionPostgresRepository::new(pool.clone()));
        let flow_inst_repo: Arc<dyn FlowInstanceRepository> =
            Arc::new(FlowInstancePostgresRepository::new(pool));

        (event_repo, flow_def_repo, flow_inst_repo)
    } else {
        info!("no database configured, using in-memory repositories");
        let event_repo: Arc<dyn EventRecordRepository> =
            Arc::new(InMemoryEventRecordRepository::new());
        let flow_def_repo: Arc<dyn FlowDefinitionRepository> =
            Arc::new(InMemoryFlowDefinitionRepository::new());
        let flow_inst_repo: Arc<dyn FlowInstanceRepository> =
            Arc::new(InMemoryFlowInstanceRepository::new());
        (event_repo, flow_def_repo, flow_inst_repo)
    };

    // KPI Cache
    let kpi_cache = Arc::new(super::cache::KpiCache::new(
        cfg.cache.kpi_max_entries,
        cfg.cache.kpi_ttl_seconds,
    ));

    // DLQ Manager client
    let dlq_client: Arc<dyn DlqManagerClient> = {
        info!("using no-op DLQ manager client");
        Arc::new(NoopDlqClient)
    };

    // Use cases
    let list_events_uc = Arc::new(usecase::ListEventsUseCase::new(event_repo.clone()));
    let trace_by_correlation_uc = Arc::new(usecase::TraceByCorrelationUseCase::new(
        event_repo.clone(),
        flow_def_repo.clone(),
        flow_inst_repo.clone(),
    ));
    let create_flow_uc = Arc::new(usecase::CreateFlowUseCase::new(flow_def_repo.clone()));
    let get_flow_uc = Arc::new(usecase::GetFlowUseCase::new(flow_def_repo.clone()));
    let update_flow_uc = Arc::new(usecase::UpdateFlowUseCase::new(flow_def_repo.clone()));
    let delete_flow_uc = Arc::new(usecase::DeleteFlowUseCase::new(flow_def_repo.clone()));
    let list_flows_uc = Arc::new(usecase::ListFlowsUseCase::new(flow_def_repo.clone()));
    let get_flow_instances_uc =
        Arc::new(usecase::GetFlowInstancesUseCase::new(flow_inst_repo.clone()));
    let get_flow_instance_uc =
        Arc::new(usecase::GetFlowInstanceUseCase::new(flow_inst_repo.clone()));
    let get_flow_kpi_uc = Arc::new(
        usecase::GetFlowKpiUseCase::new(
            flow_def_repo.clone(),
            flow_inst_repo.clone(),
        )
        .with_cache(kpi_cache.clone()),
    );
    let get_kpi_summary_uc = Arc::new(usecase::GetKpiSummaryUseCase::new(
        flow_def_repo.clone(),
        flow_inst_repo.clone(),
    ));
    let get_slo_status_uc = Arc::new(usecase::GetSloStatusUseCase::new(
        flow_def_repo.clone(),
        flow_inst_repo.clone(),
    ));
    let get_slo_burn_rate_uc = Arc::new(usecase::GetSloBurnRateUseCase::new(
        flow_def_repo.clone(),
        flow_inst_repo.clone(),
    ));
    let preview_replay_uc = Arc::new(usecase::PreviewReplayUseCase::new(
        event_repo.clone(),
        flow_def_repo.clone(),
        dlq_client.clone(),
    ));
    let execute_replay_uc = Arc::new(usecase::ExecuteReplayUseCase::new(dlq_client));

    let grpc_svc = Arc::new(EventMonitorGrpcService::new(
        list_events_uc.clone(),
        trace_by_correlation_uc.clone(),
        create_flow_uc.clone(),
        get_flow_uc.clone(),
        update_flow_uc.clone(),
        delete_flow_uc.clone(),
        list_flows_uc.clone(),
        get_flow_kpi_uc.clone(),
        get_kpi_summary_uc.clone(),
        get_slo_status_uc.clone(),
        get_slo_burn_rate_uc.clone(),
        preview_replay_uc.clone(),
        execute_replay_uc.clone(),
    ));

    // Token verifier
    let auth_state = k1s0_server_common::require_auth_state(
        "event-monitor-server",
        &cfg.app.environment,
        cfg.auth.as_ref().map(|auth_cfg| {
            info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for event-monitor-server");
            let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
                &auth_cfg.jwks_url,
                &auth_cfg.issuer,
                &auth_cfg.audience,
                std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
            ));
            crate::adapter::middleware::auth::EventMonitorAuthState {
                verifier: jwks_verifier,
            }
        }),
    )?;

    let mut state = crate::adapter::handler::AppState {
        list_events_uc,
        trace_by_correlation_uc,
        create_flow_uc,
        get_flow_uc,
        update_flow_uc,
        delete_flow_uc,
        list_flows_uc,
        get_flow_instances_uc,
        get_flow_instance_uc,
        get_flow_kpi_uc,
        get_kpi_summary_uc,
        get_slo_status_uc,
        get_slo_burn_rate_uc,
        preview_replay_uc,
        execute_replay_uc,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    let app =
        crate::adapter::handler::router(state).layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC server
    use crate::proto::k1s0::system::event_monitor::v1::event_monitor_service_server::EventMonitorServiceServer;

    let grpc_tonic = crate::adapter::grpc::EventMonitorServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(EventMonitorServiceServer::new(grpc_tonic))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app);

    // Kafka consumer (background task)
    let kafka_future = async {
        if let Some(ref kafka_cfg) = cfg.kafka {
            info!(
                brokers = %kafka_cfg.brokers.join(","),
                consumer_group = %kafka_cfg.consumer_group,
                "starting Kafka consumer"
            );
            match super::kafka_consumer::EventKafkaConsumer::new(
                kafka_cfg,
                event_repo,
                flow_def_repo,
                flow_inst_repo,
            ) {
                Ok(consumer) => {
                    if let Err(e) = consumer.run().await {
                        tracing::error!("Kafka consumer error: {}", e);
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to create Kafka consumer: {}", e);
                }
            }
        } else {
            info!("no Kafka configured, skipping consumer");
            // Keep the future alive
            std::future::pending::<()>().await;
        }
        Ok::<(), anyhow::Error>(())
    };

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
        result = kafka_future => {
            if let Err(e) = result {
                tracing::error!("Kafka consumer error: {}", e);
            }
        }
    }

    Ok(())
}

// --- InMemory Repositories ---

struct InMemoryEventRecordRepository {
    records: tokio::sync::RwLock<HashMap<Uuid, EventRecord>>,
}

impl InMemoryEventRecordRepository {
    fn new() -> Self {
        Self {
            records: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl EventRecordRepository for InMemoryEventRecordRepository {
    async fn create(&self, record: &EventRecord) -> anyhow::Result<()> {
        let mut records = self.records.write().await;
        records.insert(record.id, record.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<EventRecord>> {
        let records = self.records.read().await;
        Ok(records.get(id).cloned())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        domain: Option<String>,
        event_type: Option<String>,
        source: Option<String>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        status: Option<String>,
    ) -> anyhow::Result<(Vec<EventRecord>, u64)> {
        let records = self.records.read().await;
        let mut filtered: Vec<EventRecord> = records
            .values()
            .filter(|r| {
                if let Some(ref d) = domain {
                    if r.domain != *d {
                        return false;
                    }
                }
                if let Some(ref et) = event_type {
                    if r.event_type != *et {
                        return false;
                    }
                }
                if let Some(ref s) = source {
                    if r.source != *s {
                        return false;
                    }
                }
                if let Some(f) = from {
                    if r.timestamp < f {
                        return false;
                    }
                }
                if let Some(t) = to {
                    if r.timestamp > t {
                        return false;
                    }
                }
                if let Some(ref st) = status {
                    if r.status != *st {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        let total = filtered.len() as u64;
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let items: Vec<EventRecord> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((items, total))
    }

    async fn find_by_correlation_id(
        &self,
        correlation_id: String,
    ) -> anyhow::Result<Vec<EventRecord>> {
        let records = self.records.read().await;
        let mut events: Vec<EventRecord> = records
            .values()
            .filter(|r| r.correlation_id == correlation_id)
            .cloned()
            .collect();
        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(events)
    }
}

struct InMemoryFlowDefinitionRepository {
    flows: tokio::sync::RwLock<HashMap<Uuid, FlowDefinition>>,
}

impl InMemoryFlowDefinitionRepository {
    fn new() -> Self {
        Self {
            flows: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl FlowDefinitionRepository for InMemoryFlowDefinitionRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<FlowDefinition>> {
        let flows = self.flows.read().await;
        Ok(flows.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<FlowDefinition>> {
        let flows = self.flows.read().await;
        Ok(flows.values().filter(|f| f.enabled).cloned().collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        domain: Option<String>,
    ) -> anyhow::Result<(Vec<FlowDefinition>, u64)> {
        let flows = self.flows.read().await;
        let mut filtered: Vec<FlowDefinition> = flows
            .values()
            .filter(|f| {
                if let Some(ref d) = domain {
                    if f.domain != *d {
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
        let items: Vec<FlowDefinition> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((items, total))
    }

    async fn find_by_domain_and_event_type(
        &self,
        domain: String,
        _event_type: String,
    ) -> anyhow::Result<Vec<FlowDefinition>> {
        let flows = self.flows.read().await;
        Ok(flows
            .values()
            .filter(|f| f.domain == domain && f.enabled)
            .cloned()
            .collect())
    }

    async fn create(&self, flow: &FlowDefinition) -> anyhow::Result<()> {
        let mut flows = self.flows.write().await;
        flows.insert(flow.id, flow.clone());
        Ok(())
    }

    async fn update(&self, flow: &FlowDefinition) -> anyhow::Result<()> {
        let mut flows = self.flows.write().await;
        flows.insert(flow.id, flow.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut flows = self.flows.write().await;
        Ok(flows.remove(id).is_some())
    }

    async fn exists_by_name(&self, name: String) -> anyhow::Result<bool> {
        let flows = self.flows.read().await;
        Ok(flows.values().any(|f| f.name == name))
    }
}

struct InMemoryFlowInstanceRepository {
    instances: tokio::sync::RwLock<HashMap<Uuid, FlowInstance>>,
}

impl InMemoryFlowInstanceRepository {
    fn new() -> Self {
        Self {
            instances: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl FlowInstanceRepository for InMemoryFlowInstanceRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<FlowInstance>> {
        let instances = self.instances.read().await;
        Ok(instances.get(id).cloned())
    }

    async fn find_by_correlation_id(
        &self,
        correlation_id: String,
    ) -> anyhow::Result<Option<FlowInstance>> {
        let instances = self.instances.read().await;
        Ok(instances
            .values()
            .find(|i| i.correlation_id == correlation_id)
            .cloned())
    }

    async fn find_by_flow_id_paginated(
        &self,
        flow_id: &Uuid,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<FlowInstance>, u64)> {
        let instances = self.instances.read().await;
        let mut filtered: Vec<FlowInstance> = instances
            .values()
            .filter(|i| i.flow_id == *flow_id)
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        let total = filtered.len() as u64;
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let items: Vec<FlowInstance> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((items, total))
    }

    async fn find_in_progress(&self) -> anyhow::Result<Vec<FlowInstance>> {
        let instances = self.instances.read().await;
        Ok(instances
            .values()
            .filter(|i| i.status == FlowInstanceStatus::InProgress)
            .cloned()
            .collect())
    }

    async fn create(&self, instance: &FlowInstance) -> anyhow::Result<()> {
        let mut instances = self.instances.write().await;
        instances.insert(instance.id, instance.clone());
        Ok(())
    }

    async fn update(&self, instance: &FlowInstance) -> anyhow::Result<()> {
        let mut instances = self.instances.write().await;
        instances.insert(instance.id, instance.clone());
        Ok(())
    }
}
