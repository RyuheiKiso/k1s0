use anyhow::Context;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::info;

// gRPC 認証レイヤー
use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;

use crate::adapter;
use crate::infrastructure;
use crate::proto;
use crate::usecase;

use super::config::Config;
use crate::adapter::grpc::QuotaGrpcService;
use crate::domain::entity::quota::QuotaPolicy;
use crate::domain::repository::{
    CheckAndIncrementResult, QuotaPolicyRepository, QuotaUsageRepository,
};

// HIGH-001 監査対応: 起動処理は構造上行数が多くなるため許容する
#[allow(clippy::too_many_lines, clippy::items_after_statements)]
pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-quota-server".to_string(),
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

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting quota server"
    );

    // --- Repository initialization: Redis → PostgreSQL → InMemory fallback ---

    // Redis ConnectionManager (usage_repo 用、policy_repo は常にDB/InMemory)
    let redis_conn: Option<redis::aio::ConnectionManager> = if let Some(ref redis_cfg) = cfg.redis {
        info!(url = %redis_cfg.url, "connecting to Redis for usage counters");
        match redis::Client::open(redis_cfg.url.as_str()) {
            Ok(client) => match redis::aio::ConnectionManager::new(client).await {
                Ok(cm) => {
                    info!("Redis connection established for usage counters");
                    Some(cm)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "failed to connect to Redis, will fall back");
                    None
                }
            },
            Err(e) => {
                tracing::warn!(error = %e, "invalid Redis URL, will fall back");
                None
            }
        }
    } else {
        None
    };

    // CRITICAL-003 対応: readyz ハンドラに渡す db_pool を事前確保する
    let mut db_pool_for_readyz: Option<sqlx::PgPool> = None;

    let (policy_repo, usage_repo): (
        Arc<dyn QuotaPolicyRepository>,
        Arc<dyn QuotaUsageRepository>,
    ) = if let Some(ref db_cfg) = cfg.database {
        info!(url = %db_cfg.url, "connecting to PostgreSQL");
        match infrastructure::database::create_pool(&db_cfg.url, db_cfg.max_connections).await {
            Ok(raw_pool) => {
                // readyz で SELECT 1 チェックに使用するため clone を保持する（PgPool はArc-backed で軽量）
                db_pool_for_readyz = Some(raw_pool.clone());
                let pool = Arc::new(raw_pool);
                info!("PostgreSQL connection pool created successfully");
                let policy_repo = Arc::new(
                    adapter::repository::QuotaPolicyPostgresRepository::new(pool.clone()),
                );
                // usage_repo: Redis が使えればRedis、なければPostgreSQL
                let usage_repo: Arc<dyn QuotaUsageRepository> = if let Some(ref cm) = redis_conn {
                    let prefix = cfg
                        .redis
                        .as_ref().map_or_else(|| "quota:".to_string(), |r| r.key_prefix.clone());
                    info!("using Redis for usage counters (prefix={})", prefix);
                    Arc::new(infrastructure::redis_store::RedisQuotaUsageRepository::new(
                        cm.clone(),
                        prefix,
                    ))
                } else {
                    Arc::new(adapter::repository::QuotaUsagePostgresRepository::new(pool))
                };
                (policy_repo, usage_repo)
            }
            Err(e) => {
                // 環境に応じてフォールバックの許否を判断する。
                // dev/test 以外ではインフラ接続失敗時に即座にサーバー起動を中断する。
                if !k1s0_server_common::allow_in_memory_infra(&cfg.app.environment) {
                    return Err(anyhow::anyhow!(
                        "PostgreSQL 接続に失敗しました。本番環境ではフォールバックは許可されていません: {e}"
                    ));
                }
                tracing::warn!(
                    error = %e,
                    "dev/test 環境: PostgreSQL 接続失敗のため InMemory フォールバックで起動します"
                );
                // usage_repo: Redis が使えればRedis、なければInMemory
                let usage_repo: Arc<dyn QuotaUsageRepository> = if let Some(ref cm) = redis_conn {
                    let prefix = cfg
                        .redis
                        .as_ref().map_or_else(|| "quota:".to_string(), |r| r.key_prefix.clone());
                    Arc::new(infrastructure::redis_store::RedisQuotaUsageRepository::new(
                        cm.clone(),
                        prefix,
                    ))
                } else {
                    Arc::new(InMemoryQuotaUsageRepository::new())
                };
                (
                    Arc::new(InMemoryQuotaPolicyRepository::new())
                        as Arc<dyn QuotaPolicyRepository>,
                    usage_repo,
                )
            }
        }
    } else {
        // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "quota",
            k1s0_server_common::InfraKind::Database,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no database config found, using InMemory repositories (dev/test bypass)");
        // usage_repo: Redis が使えればRedis、なければInMemory
        let usage_repo: Arc<dyn QuotaUsageRepository> = if let Some(ref cm) = redis_conn {
            let prefix = cfg
                .redis
                .as_ref().map_or_else(|| "quota:".to_string(), |r| r.key_prefix.clone());
            Arc::new(infrastructure::redis_store::RedisQuotaUsageRepository::new(
                cm.clone(),
                prefix,
            ))
        } else {
            Arc::new(InMemoryQuotaUsageRepository::new())
        };
        (
            Arc::new(InMemoryQuotaPolicyRepository::new()) as Arc<dyn QuotaPolicyRepository>,
            usage_repo,
        )
    };

    // --- Kafka event publisher initialization ---
    let event_publisher: Arc<dyn infrastructure::kafka_producer::QuotaEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            match infrastructure::kafka_producer::KafkaQuotaProducer::new(
                &kafka_cfg.brokers.join(","),
                &kafka_cfg.security_protocol,
                &kafka_cfg.topic_exceeded,
                &kafka_cfg.topic_threshold,
            ) {
                Ok(producer) => {
                    info!("Kafka producer initialized for quota exceeded events");
                    Arc::new(producer)
                }
                Err(e) => {
                    // 環境に応じてフォールバックの許否を判断する。
                    // dev/test 以外では Kafka 初期化失敗時に即座にサーバー起動を中断する。
                    if !k1s0_server_common::allow_in_memory_infra(&cfg.app.environment) {
                        return Err(anyhow::anyhow!(
                            "Kafka プロデューサーの初期化に失敗しました。本番環境ではフォールバックは許可されていません: {e}"
                        ));
                    }
                    tracing::warn!(
                        error = %e,
                        "dev/test 環境: Kafka 初期化失敗のため NoopQuotaEventPublisher で起動します"
                    );
                    Arc::new(infrastructure::kafka_producer::NoopQuotaEventPublisher)
                }
            }
        } else {
            // Kafka 設定が未指定の場合も infra_guard で dev/test 環境のみ許可する。
            k1s0_server_common::require_infra(
                "quota",
                k1s0_server_common::InfraKind::Kafka,
                &cfg.app.environment,
                None::<String>,
            )?;
            info!("no Kafka config found, using NoopQuotaEventPublisher (dev/test bypass)");
            Arc::new(infrastructure::kafka_producer::NoopQuotaEventPublisher)
        };

    let create_policy_uc = Arc::new(usecase::CreateQuotaPolicyUseCase::new(policy_repo.clone()));
    let get_policy_uc = Arc::new(usecase::GetQuotaPolicyUseCase::new(policy_repo.clone()));
    let list_policies_uc = Arc::new(usecase::ListQuotaPoliciesUseCase::new(policy_repo.clone()));
    let update_policy_uc = Arc::new(usecase::UpdateQuotaPolicyUseCase::new(policy_repo.clone()));
    let delete_policy_uc = Arc::new(usecase::DeleteQuotaPolicyUseCase::new(policy_repo.clone()));
    let get_usage_uc = Arc::new(usecase::GetQuotaUsageUseCase::new(
        policy_repo.clone(),
        usage_repo.clone(),
    ));
    let increment_usage_uc = Arc::new(usecase::IncrementQuotaUsageUseCase::new(
        policy_repo.clone(),
        usage_repo.clone(),
        event_publisher,
    ));
    let reset_usage_uc = Arc::new(usecase::ResetQuotaUsageUseCase::new(
        policy_repo,
        usage_repo,
    ));

    let grpc_svc = Arc::new(QuotaGrpcService::new(
        create_policy_uc.clone(),
        get_policy_uc.clone(),
        list_policies_uc.clone(),
        update_policy_uc.clone(),
        delete_policy_uc.clone(),
        get_usage_uc.clone(),
        increment_usage_uc.clone(),
        reset_usage_uc.clone(),
    ));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-quota-server"));

    // H-006 監査対応: cron タスクの健全性フラグを初期化する（true = 正常稼働中）
    // readyz ハンドラがこのフラグを参照し、false になったら 503 を返してプロセス再起動を促す
    let cron_healthy = Arc::new(std::sync::atomic::AtomicBool::new(true));

    // Quota usage auto-reset cron
    {
        let daily_cron = cfg.quota.reset_schedule.daily.clone();
        let monthly_cron = cfg.quota.reset_schedule.monthly.clone();
        let cron_reset_uc = reset_usage_uc.clone();
        let cron_list_uc = list_policies_uc.clone();
        // H-006 監査対応: cron タスクに健全性フラグの参照を渡し、
        // 最大リトライ到達時に false にセットさせることで readyz に連携する
        let cron_healthy_flag = Arc::clone(&cron_healthy);
        tokio::spawn(async move {
            run_reset_cron(daily_cron, monthly_cron, cron_reset_uc, cron_list_uc, cron_healthy_flag).await;
        });
    }

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "quota-server",
        &cfg.app.environment,
        cfg.auth
            .as_ref()
            .map(|auth_cfg| -> anyhow::Result<_> {
                // nested 形式の AuthConfig から JWKS 検証器を初期化する
                let jwks = auth_cfg
                    .jwks
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("auth.jwks configuration is required"))?;
                info!(jwks_url = %jwks.url, "initializing JWKS verifier for quota-server");
                let jwks_verifier = Arc::new(
                    k1s0_auth::JwksVerifier::new(
                        &jwks.url,
                        &auth_cfg.jwt.issuer,
                        &auth_cfg.jwt.audience,
                        std::time::Duration::from_secs(jwks.cache_ttl_secs),
                    )
                    .context("JWKS 検証器の作成に失敗")?,
                );
                Ok(adapter::middleware::auth::AuthState {
                    verifier: jwks_verifier,
                })
            })
            .transpose()?,
    )?;

    let mut state = adapter::handler::AppState {
        create_policy_uc,
        get_policy_uc,
        list_policies_uc,
        update_policy_uc,
        delete_policy_uc,
        get_usage_uc,
        increment_usage_uc,
        reset_usage_uc,
        metrics: metrics.clone(),
        auth_state: None,
        // CRITICAL-003 対応: /readyz で DB 疎通確認に使用する
        db_pool: db_pool_for_readyz,
        // H-006 監査対応: cron リセットタスクの健全性フラグを AppState に渡す
        cron_healthy: Arc::clone(&cron_healthy),
    };
    // gRPC 認証レイヤー用に auth_state を REST への移動前にクローンしておく。
    let grpc_auth_layer = GrpcAuthLayer::new(auth_state.clone(), Tier::System, quota_grpc_action);
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    let app = adapter::handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC server
    use proto::k1s0::system::quota::v1::quota_service_server::QuotaServiceServer;

    let quota_tonic = adapter::grpc::QuotaServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    // gRPC グレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(QuotaServiceServer::new(quota_tonic))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {e}"))
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
/// `CreateQuotaPolicy` / `UpdateQuotaPolicy` / `DeleteQuotaPolicy` / `IncrementQuotaUsage` / `ResetQuotaUsage` は write、
/// それ以外は read。
fn quota_grpc_action(method: &str) -> &'static str {
    match method {
        "CreateQuotaPolicy"
        | "UpdateQuotaPolicy"
        | "DeleteQuotaPolicy"
        | "IncrementQuotaUsage"
        | "ResetQuotaUsage" => "write",
        _ => "read",
    }
}

// --- Cron-based quota usage reset ---

// H-006/M-005 監査対応: cron計算失敗時の最大リトライ回数
// この回数を超えた場合はエラーログを出力してタスクを終了する
const CRON_MAX_RETRY: u32 = 10;

// H-006/M-005 監査対応: cron計算失敗時のリトライ待機時間（秒）
// 一時的な障害（時刻同期ずれ等）を考慮して60秒後に再試行する
const CRON_RETRY_WAIT_SECS: u64 = 60;

/// H-006 監査対応: quota リセット cron タスク。
/// `cron_healthy` フラグを受け取り、最大リトライ到達時に false にセットして
/// /readyz が 503 を返すようにし、プロセス再起動を促す。
async fn run_reset_cron(
    daily_expr: String,
    monthly_expr: String,
    reset_uc: Arc<usecase::ResetQuotaUsageUseCase>,
    list_uc: Arc<usecase::ListQuotaPoliciesUseCase>,
    cron_healthy: Arc<std::sync::atomic::AtomicBool>,
) {
    use std::sync::atomic::Ordering;

    // HIGH-004 監査対応: croner v2 は with_seconds_required() を明示しないと 5 フィールドモードで動作する。
    // Cron::from_str() や "...".parse() はデフォルト 5 フィールドのため、
    // 6 フィールド式（"0 0 0 * * *"）を渡すと find_next_occurrence が常に Err を返す。
    // Cron::new(expr).with_seconds_required().parse() に変更して 6 フィールドモードを明示する。
    let schedules: Vec<(&str, croner::Cron)> = [
        ("daily", daily_expr.as_str()),
        ("monthly", monthly_expr.as_str()),
    ]
    .into_iter()
    .filter_map(|(label, expr)| match croner::Cron::new(expr).with_seconds_required().parse() {
        Ok(cron) => {
            info!(
                schedule = label,
                expression = expr,
                "cron schedule registered"
            );
            Some((label, cron))
        }
        Err(e) => {
            tracing::error!(
                schedule = label,
                expression = expr,
                error = %e,
                "failed to parse cron expression, skipping"
            );
            None
        }
    })
    .collect();

    // 有効なスケジュールが1件もない場合はタスクを終了する
    // H-006 監査対応: スケジュールが空の場合も cron_healthy を false にして readyz に連携する
    if schedules.is_empty() {
        tracing::warn!("no valid cron schedules, reset cron task exiting");
        cron_healthy.store(false, Ordering::Relaxed);
        return;
    }

    // H-006 監査対応: cron計算失敗の連続カウンタ
    // find_next_occurrence が連続してNoneを返した場合にリトライ上限でタスクを終了する
    let mut consecutive_failures: u32 = 0;

    loop {
        let now = chrono::Utc::now();
        let mut next_fire: Option<(chrono::DateTime<chrono::Utc>, &str)> = None;

        for (label, cron) in &schedules {
            if let Ok(next) = cron.find_next_occurrence(&now, false) {
                // 次の発火時刻が未設定、または現在の候補より早い場合に更新する
                let should_update = match next_fire {
                    None => true,
                    Some((current_next, _)) => next < current_next,
                };
                if should_update {
                    next_fire = Some((next, label));
                }
            }
        }

        let (fire_at, label) = if let Some(v) = next_fire {
            // cron計算に成功したので連続失敗カウンタをリセットする
            consecutive_failures = 0;
            v
        } else {
            // H-006 監査対応: cron計算失敗時に即座に終了するのではなく、
            // 60秒待機後にリトライする。最大リトライ回数到達後にフラグを false にして終了する。
            consecutive_failures += 1;
            tracing::error!(
                retry = consecutive_failures,
                max_retry = CRON_MAX_RETRY,
                wait_secs = CRON_RETRY_WAIT_SECS,
                "cron occurrence の計算に失敗しました。リトライ待機後に再試行します",
            );
            if consecutive_failures >= CRON_MAX_RETRY {
                // H-006 監査対応: 最大リトライ到達時に cron_healthy を false にセットして
                // /readyz が 503 を返すようにし、Kubernetes の readinessProbe によるプロセス再起動を促す
                cron_healthy.store(false, Ordering::Relaxed);
                tracing::error!(
                    max_retry = CRON_MAX_RETRY,
                    "cron occurrence の計算が連続して失敗しました。\
                    reset cron タスクを終了します（readyz に unhealthy を通知済み）。\
                    システム管理者は cron 式の設定を確認してください。",
                );
                return;
            }
            tokio::time::sleep(std::time::Duration::from_secs(CRON_RETRY_WAIT_SECS)).await;
            continue;
        };

        let wait = (fire_at - chrono::Utc::now())
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(1));

        info!(
            schedule = label,
            next_run = %fire_at,
            wait_secs = wait.as_secs(),
            "sleeping until next reset"
        );

        tokio::time::sleep(wait).await;

        info!(schedule = label, "running quota usage reset");
        reset_all_policies(&reset_uc, &list_uc, label).await;
    }
}

async fn reset_all_policies(
    reset_uc: &usecase::ResetQuotaUsageUseCase,
    list_uc: &usecase::ListQuotaPoliciesUseCase,
    schedule_label: &str,
) {
    let mut page = 1u32;
    let page_size = 100u32;
    let mut total_reset = 0u64;

    loop {
        // CRITICAL-RUST-001 監査対応: スケジューラは全テナントを対象とするため
        // system テナントとして実行する（RLS bypass 権限を持つロールが望ましい）。
        let input = usecase::list_quota_policies::ListQuotaPoliciesInput {
            page,
            page_size,
            subject_type: None,
            subject_id: None,
            enabled_only: None,
            tenant_id: "system".to_string(),
        };
        match list_uc.execute(&input).await {
            Ok(output) => {
                for policy in &output.quotas {
                    let reset_input = usecase::reset_quota_usage::ResetQuotaUsageInput {
                        quota_id: policy.id.clone(),
                        reason: format!("scheduled {schedule_label} reset"),
                        reset_by: "system-cron".to_string(),
                        tenant_id: policy.tenant_id.clone(),
                    };
                    if let Err(e) = reset_uc.execute(&reset_input).await {
                        tracing::warn!(
                            quota_id = %policy.id,
                            error = %e,
                            "failed to reset quota usage"
                        );
                    } else {
                        total_reset += 1;
                    }
                }
                if !output.has_next {
                    break;
                }
                page += 1;
            }
            Err(e) => {
                tracing::error!(
                    page = page,
                    error = %e,
                    "failed to list policies for reset, aborting this cycle"
                );
                break;
            }
        }
    }

    info!(
        schedule = schedule_label,
        total_reset = total_reset,
        "quota usage reset cycle completed"
    );
}

// --- InMemory Repositories ---

struct InMemoryQuotaPolicyRepository {
    policies: RwLock<HashMap<String, QuotaPolicy>>,
}

impl InMemoryQuotaPolicyRepository {
    fn new() -> Self {
        Self {
            policies: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl QuotaPolicyRepository for InMemoryQuotaPolicyRepository {
    /// `InMemory` 実装では `tenant_id` によるフィルタリングは行わない（テスト・開発用）。
    async fn find_by_id(&self, id: &str, _tenant_id: &str) -> anyhow::Result<Option<QuotaPolicy>> {
        let policies = self.policies.read().await;
        Ok(policies.get(id).cloned())
    }

    async fn find_all(
        &self,
        page: u32,
        page_size: u32,
        _tenant_id: &str,
    ) -> anyhow::Result<(Vec<QuotaPolicy>, u64)> {
        let policies = self.policies.read().await;
        let all: Vec<QuotaPolicy> = policies.values().cloned().collect();
        let total = all.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let items: Vec<QuotaPolicy> = all
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((items, total))
    }

    async fn create(&self, policy: &QuotaPolicy) -> anyhow::Result<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy.clone());
        Ok(())
    }

    async fn update(&self, policy: &QuotaPolicy) -> anyhow::Result<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy.clone());
        Ok(())
    }

    async fn delete(&self, id: &str, _tenant_id: &str) -> anyhow::Result<bool> {
        let mut policies = self.policies.write().await;
        Ok(policies.remove(id).is_some())
    }
}

struct InMemoryQuotaUsageRepository {
    counters: RwLock<HashMap<String, u64>>,
}

impl InMemoryQuotaUsageRepository {
    fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl QuotaUsageRepository for InMemoryQuotaUsageRepository {
    /// `InMemory` 実装では `tenant_id` によるフィルタリングは行わない（テスト・開発用）。
    async fn get_usage(&self, quota_id: &str, _tenant_id: &str) -> anyhow::Result<Option<u64>> {
        let counters = self.counters.read().await;
        Ok(counters.get(quota_id).copied())
    }

    async fn increment(&self, quota_id: &str, amount: u64, _tenant_id: &str) -> anyhow::Result<u64> {
        let mut counters = self.counters.write().await;
        let counter = counters.entry(quota_id.to_string()).or_insert(0);
        *counter += amount;
        Ok(*counter)
    }

    async fn reset(&self, quota_id: &str, _tenant_id: &str) -> anyhow::Result<()> {
        let mut counters = self.counters.write().await;
        counters.insert(quota_id.to_string(), 0);
        Ok(())
    }

    async fn check_and_increment(
        &self,
        quota_id: &str,
        amount: u64,
        limit: u64,
        _tenant_id: &str,
    ) -> anyhow::Result<CheckAndIncrementResult> {
        let mut counters = self.counters.write().await;
        let current = counters.entry(quota_id.to_string()).or_insert(0);
        if *current + amount > limit {
            Ok(CheckAndIncrementResult {
                used: *current,
                allowed: false,
            })
        } else {
            *current += amount;
            Ok(CheckAndIncrementResult {
                used: *current,
                allowed: true,
            })
        }
    }
}
