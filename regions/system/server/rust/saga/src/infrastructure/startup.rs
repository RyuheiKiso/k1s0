use anyhow::Context;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

// gRPC 認証レイヤー
use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;

use crate::adapter;
use crate::domain;
use crate::infrastructure;
use crate::proto;
use crate::usecase;

use super::config::Config;
use super::grpc_caller::{ServiceRegistry, TonicGrpcCaller};
use crate::adapter::grpc::{SagaGrpcService, SagaServiceTonic};
use crate::adapter::handler::{self, AppState};
use crate::adapter::repository::saga_postgres::SagaPostgresRepository;
use crate::adapter::repository::workflow_in_memory::InMemoryWorkflowRepository;
use crate::adapter::repository::workflow_postgres::WorkflowPostgresRepository;
use crate::domain::repository::WorkflowRepository;

// HIGH-001 監査対応: 起動処理は構造上行数が多くなるため許容する
#[allow(clippy::too_many_lines, clippy::items_after_statements)]
pub async fn run() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-saga-server".to_string(),
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
        .map_err(|e| anyhow::anyhow!("テレメトリの初期化に失敗: {e}"))?;

    // Config

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting saga server"
    );

    // Database pool (optional)
    let db_pool = if let Some(ref db_config) = cfg.database {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_config.connection_url());
        info!("connecting to database");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_config.max_open_conns)
            .connect(&url)
            .await?;
        info!("database connection pool established");
        // 起動時に saga-db マイグレーションを適用する（C-01 対応）
        crate::MIGRATOR
            .run(&pool)
            .await
            .map_err(|e| anyhow::anyhow!("saga-db migration failed: {e}"))?;
        tracing::info!("saga-db migrations applied successfully");
        Some(pool)
    } else if let Ok(url) = std::env::var("DATABASE_URL") {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(25)
            .connect(&url)
            .await?;
        info!("database connection pool established from DATABASE_URL");
        // 起動時に saga-db マイグレーションを適用する（C-01 対応）
        crate::MIGRATOR
            .run(&pool)
            .await
            .map_err(|e| anyhow::anyhow!("saga-db migration failed: {e}"))?;
        tracing::info!("saga-db migrations applied successfully");
        Some(pool)
    } else {
        // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "saga",
            k1s0_server_common::InfraKind::Database,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no database configured, using in-memory repositories (dev/test bypass)");
        None
    };

    // Saga repository
    let saga_repo: Arc<dyn domain::repository::SagaRepository> = if let Some(ref pool) = db_pool {
        Arc::new(SagaPostgresRepository::new(pool.clone()))
    } else {
        Arc::new(InMemorySagaRepository::new())
    };

    // Workflow repository
    let workflow_repo: Arc<dyn WorkflowRepository> = if let Some(ref pool) = db_pool {
        Arc::new(WorkflowPostgresRepository::new(pool.clone()))
    } else {
        Arc::new(InMemoryWorkflowRepository::new())
    };

    let workflow_loader =
        infrastructure::workflow_loader::WorkflowLoader::new(&cfg.saga.workflow_dir);
    let loaded_definitions = workflow_loader.load_all().await?;
    for workflow in &loaded_definitions {
        workflow_repo.register(workflow.clone()).await?;
    }
    info!(
        count = loaded_definitions.len(),
        "workflow definitions loaded from directory via WorkflowLoader"
    );

    // Service registry + gRPC caller
    let registry = Arc::new(ServiceRegistry::new(cfg.services.clone()));
    let grpc_caller: Arc<dyn infrastructure::grpc_caller::GrpcStepCaller> =
        Arc::new(TonicGrpcCaller::new(registry));

    // Kafka publisher (optional)
    let publisher: Option<Arc<dyn infrastructure::kafka_producer::SagaEventPublisher>> =
        if let Some(ref kafka_config) = cfg.kafka {
            match infrastructure::kafka_producer::KafkaProducer::new(kafka_config) {
                Ok(producer) => {
                    info!("kafka producer initialized");
                    Some(Arc::new(producer))
                }
                Err(e) => {
                    tracing::warn!(error = %e, "failed to create kafka producer, events will not be published");
                    None
                }
            }
        } else {
            info!("no kafka configured, saga events will not be published");
            None
        };

    // Use cases
    // Kafka シャットダウン時の close() 呼び出し用にクローンを保持する（publisher は直後に消費される）
    let publisher_for_shutdown = publisher.clone();
    let execute_saga_uc = Arc::new(
        usecase::ExecuteSagaUseCase::new(saga_repo.clone(), grpc_caller.clone(), publisher)
            .with_workflow_repo(
                workflow_repo.clone() as Arc<dyn domain::repository::WorkflowRepository>
            ),
    );

    let start_saga_uc = Arc::new(usecase::StartSagaUseCase::new(
        saga_repo.clone(),
        workflow_repo.clone() as Arc<dyn domain::repository::WorkflowRepository>,
        execute_saga_uc.clone(),
    ));
    // グレースフルシャットダウン用に task_tracker を AppState へのムーブ前に取得する
    let task_tracker = start_saga_uc.task_tracker().clone();

    let get_saga_uc = Arc::new(usecase::GetSagaUseCase::new(saga_repo.clone()));
    let list_sagas_uc = Arc::new(usecase::ListSagasUseCase::new(saga_repo.clone()));
    let cancel_saga_uc = Arc::new(usecase::CancelSagaUseCase::new(saga_repo.clone()));
    let register_workflow_uc = Arc::new(usecase::RegisterWorkflowUseCase::new(
        workflow_repo.clone() as Arc<dyn domain::repository::WorkflowRepository>,
    ));
    let list_workflows_uc = Arc::new(usecase::ListWorkflowsUseCase::new(
        workflow_repo.clone() as Arc<dyn domain::repository::WorkflowRepository>
    ));

    // Startup recovery
    let recover_uc = usecase::RecoverSagasUseCase::new(
        saga_repo.clone(),
        workflow_repo.clone() as Arc<dyn domain::repository::WorkflowRepository>,
        execute_saga_uc.clone(),
    );
    let recovered = recover_uc.execute().await?;
    if recovered > 0 {
        info!(count = recovered, "sagas recovered at startup");
    }

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "saga-server",
        &cfg.app.environment,
        cfg.auth
            .as_ref()
            .map(|auth_cfg| -> anyhow::Result<_> {
                // nested 形式の AuthConfig から JWKS URL を取得する
                let jwks_url = auth_cfg
                    .jwks
                    .as_ref()
                    .map(|j| j.url.as_str())
                    .unwrap_or_default();
                let cache_ttl = auth_cfg
                    .jwks
                    .as_ref()
                    .map_or(300, |j| j.cache_ttl_secs);
                info!(jwks_url = %jwks_url, "initializing JWKS verifier for saga-server");
                let jwks_verifier = Arc::new(
                    k1s0_auth::JwksVerifier::new(
                        jwks_url,
                        &auth_cfg.jwt.issuer,
                        &auth_cfg.jwt.audience,
                        std::time::Duration::from_secs(cache_ttl),
                    )
                    .context("JWKS 検証器の作成に失敗")?,
                );
                Ok(adapter::middleware::auth::AuthState {
                    verifier: jwks_verifier,
                })
            })
            .transpose()?,
    )?;

    // gRPC 認証レイヤー用に auth_state を REST への移動前にクローンしておく。
    let grpc_auth_layer = GrpcAuthLayer::new(auth_state.clone(), Tier::System, saga_grpc_action);

    // AppState (REST handler用)
    // db_pool は /healthz エンドポイントで DB 接続確認に使用する（C-02 対応）
    let mut state = AppState {
        start_saga_uc,
        get_saga_uc,
        list_sagas_uc,
        cancel_saga_uc,
        execute_saga_uc: execute_saga_uc.clone(),
        register_workflow_uc,
        list_workflows_uc,
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-saga-server")),
        auth_state: None,
        db_pool: db_pool.clone(),
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    // gRPC service
    let saga_grpc_svc = Arc::new(SagaGrpcService::new(
        state.start_saga_uc.clone(),
        state.get_saga_uc.clone(),
        state.list_sagas_uc.clone(),
        state.cancel_saga_uc.clone(),
        state.execute_saga_uc.clone(),
        state.register_workflow_uc.clone(),
        state.list_workflows_uc.clone(),
    ));
    use proto::k1s0::system::saga::v1::saga_service_server::SagaServiceServer;

    let saga_tonic = SagaServiceTonic::new(saga_grpc_svc);

    // Metrics for layers
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-saga-server"));

    // Router
    let app = handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC server（server.host は設定ファイルで制御可能。本番は 0.0.0.0、テスト環境は 127.0.0.1 を使用する）
    let grpc_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.grpc_port)
        .parse()
        .context("gRPC バインドアドレスのパースに失敗")?;
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    // gRPCサーバーのグレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(SagaServiceServer::new(saga_tonic))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {e}"))
    };

    // REST server（server.host は設定ファイルで制御可能）
    let rest_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port)
        .parse()
        .context("REST バインドアドレスのパースに失敗")?;
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    // RESTサーバーのグレースフルシャットダウン設定
    let rest_future = axum::serve(listener, app).with_graceful_shutdown(async {
        let _ = k1s0_server_common::shutdown::shutdown_signal().await;
    });

    // task_tracker は start_saga_uc 作成直後（AppState へのムーブ前）に取得済み

    // Run REST and gRPC concurrently.
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

    // サーバーが終了した後、実行中の Saga タスクが完了するまで最大 30 秒待機する。
    // 超過した場合でもタスクは Tokio ランタイムのシャットダウンで強制終了される。
    let active = task_tracker.active_count();
    if active > 0 {
        info!(
            active_tasks = active,
            "waiting for in-progress saga tasks to complete (max 30s)"
        );
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            task_tracker.wait_for_completion(),
        )
        .await;
        info!("saga task drain complete");
    }

    // Kafka プロデューサーをフラッシュしてシャットダウンする（未送信メッセージを確実に送出する）
    if let Some(ref pub_handle) = publisher_for_shutdown {
        if let Err(e) = pub_handle.close().await {
            tracing::warn!("kafka producer close error during shutdown: {}", e);
        }
    }

    // テレメトリのシャットダウン処理
    k1s0_telemetry::shutdown();

    Ok(())
}

/// gRPC メソッド名から必要な RBAC アクション文字列を返す。
/// `StartSaga` / `CancelSaga` / `CompensateSaga` / `RegisterWorkflow` は write、それ以外は read。
fn saga_grpc_action(method: &str) -> &'static str {
    match method {
        "StartSaga" | "CancelSaga" | "CompensateSaga" | "RegisterWorkflow" => "write",
        _ => "read",
    }
}

// --- In-memory Saga Repository for dev mode ---

struct InMemorySagaRepository {
    states: tokio::sync::RwLock<Vec<domain::entity::saga_state::SagaState>>,
    step_logs: tokio::sync::RwLock<Vec<domain::entity::saga_step_log::SagaStepLog>>,
}

impl InMemorySagaRepository {
    fn new() -> Self {
        Self {
            states: tokio::sync::RwLock::new(Vec::new()),
            step_logs: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl domain::repository::SagaRepository for InMemorySagaRepository {
    async fn create(&self, state: &domain::entity::saga_state::SagaState) -> anyhow::Result<()> {
        self.states.write().await.push(state.clone());
        Ok(())
    }

    async fn update_with_step_log(
        &self,
        state: &domain::entity::saga_state::SagaState,
        log: &domain::entity::saga_step_log::SagaStepLog,
    ) -> anyhow::Result<()> {
        let mut states = self.states.write().await;
        if let Some(s) = states.iter_mut().find(|s| s.saga_id == state.saga_id) {
            *s = state.clone();
        }
        self.step_logs.write().await.push(log.clone());
        Ok(())
    }

    /// `InMemory実装`: `tenant_id` はテナントフィルタに使用しないが、トレイト定義に合わせて引数を受け取る。
    async fn update_status(
        &self,
        saga_id: uuid::Uuid,
        status: &domain::entity::saga_state::SagaStatus,
        error_message: Option<String>,
        _tenant_id: &str,
    ) -> anyhow::Result<()> {
        let mut states = self.states.write().await;
        if let Some(s) = states.iter_mut().find(|s| s.saga_id == saga_id) {
            s.status = status.clone();
            s.error_message = error_message;
            s.updated_at = chrono::Utc::now();
        }
        Ok(())
    }

    /// `InMemory実装`: `tenant_id` はテナントフィルタに使用しないが、トレイト定義に合わせて引数を受け取る。
    async fn find_by_id(
        &self,
        saga_id: uuid::Uuid,
        _tenant_id: &str,
    ) -> anyhow::Result<Option<domain::entity::saga_state::SagaState>> {
        let states = self.states.read().await;
        Ok(states.iter().find(|s| s.saga_id == saga_id).cloned())
    }

    /// `InMemory実装`: `tenant_id` はテナントフィルタに使用しないが、トレイト定義に合わせて引数を受け取る。
    async fn find_step_logs(
        &self,
        saga_id: uuid::Uuid,
        _tenant_id: &str,
    ) -> anyhow::Result<Vec<domain::entity::saga_step_log::SagaStepLog>> {
        let logs = self.step_logs.read().await;
        Ok(logs
            .iter()
            .filter(|l| l.saga_id == saga_id)
            .cloned()
            .collect())
    }

    async fn list(
        &self,
        params: &domain::repository::saga_repository::SagaListParams,
    ) -> anyhow::Result<(Vec<domain::entity::saga_state::SagaState>, i32)> {
        let states = self.states.read().await;
        let filtered: Vec<_> = states
            .iter()
            .filter(|s| {
                if let Some(ref wn) = params.workflow_name {
                    if s.workflow_name != *wn {
                        return false;
                    }
                }
                if let Some(ref st) = params.status {
                    if s.status != *st {
                        return false;
                    }
                }
                if let Some(ref ci) = params.correlation_id {
                    if s.correlation_id.as_deref() != Some(ci.as_str()) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let total = filtered.len() as i32;
        let page = params.page.max(1);
        let page_size = params.page_size.max(1);
        let offset = ((page - 1) * page_size) as usize;
        let limit = page_size as usize;
        let paged: Vec<_> = filtered.into_iter().skip(offset).take(limit).collect();

        Ok((paged, total))
    }

    async fn find_incomplete(&self) -> anyhow::Result<Vec<domain::entity::saga_state::SagaState>> {
        let states = self.states.read().await;
        Ok(states
            .iter()
            .filter(|s| {
                matches!(
                    s.status,
                    domain::entity::saga_state::SagaStatus::Started
                        | domain::entity::saga_state::SagaStatus::Running
                        | domain::entity::saga_state::SagaStatus::Compensating
                )
            })
            .cloned()
            .collect())
    }
}
