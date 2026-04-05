use anyhow::Context;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;

use super::config::Config;
use super::kafka_producer::{
    KafkaSessionProducer, NoopSessionEventPublisher, SessionEventPublisher,
};
use crate::adapter::grpc::SessionGrpcService;
use crate::adapter::repository::session_metadata_postgres::{
    NoopSessionMetadataRepository, SessionMetadataPostgresRepository, SessionMetadataRepository,
};
use crate::adapter::repository::session_redis::RedisSessionRepository;
use crate::domain::entity::session::Session;
use crate::domain::repository::SessionRepository;
use crate::error::SessionError;

/// HIGH-002 対応: jti チェッカー構築ヘルパー。
/// Redis に接続済みの場合は jti 失効確認コールバックを返す。
/// コールバックは同期 Fn であるため、tokio の block_in_place を使って非同期処理をブロッキングで実行する。
/// auth ライブラリが同期コールバックを要求しているため、この変換が必要となる。
fn build_jti_checker(
    redis_repo: Arc<RedisSessionRepository>,
) -> k1s0_auth::JtiRevokedChecker {
    Arc::new(move |jti: &str| {
        let repo = Arc::clone(&redis_repo);
        let jti = jti.to_string();
        // tokio::runtime::Handle::current() で現在のランタイムを取得し、
        // block_on で非同期の is_jti_revoked を同期的に実行する。
        // block_in_place を使い tokio スレッドプールをブロックしないようにする。
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                repo.is_jti_revoked(&jti).await
            })
        })
    })
}

pub async fn run() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-session-server".to_string(),
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

    info!(port = cfg.server.port, "starting session server");

    // --- Session Repository: Redis or InMemory fallback ---
    // HIGH-002 対応: Redis 実装の場合は Arc<RedisSessionRepository> を保持し、
    // JWKS 検証器の jti チェッカーに渡せるようにする。
    let mut redis_repo_for_jti: Option<Arc<RedisSessionRepository>> = None;

    let repo: Arc<dyn SessionRepository> = if let Some(ref redis_cfg) = cfg.redis {
        info!(url = %redis_cfg.url, "connecting to Redis");
        let client = redis::Client::open(redis_cfg.url.as_str())
            .map_err(|e| anyhow::anyhow!("failed to create Redis client: {}", e))?;
        let conn = redis::aio::ConnectionManager::new(client)
            .await
            .map_err(|e| anyhow::anyhow!("failed to connect to Redis: {}", e))?;
        info!("Redis connection established");
        let redis_repo = Arc::new(RedisSessionRepository::new(conn));
        // jti チェッカー用に Arc<RedisSessionRepository> を保持する（Arc::clone で参照カウントのみ増加）
        redis_repo_for_jti = Some(Arc::clone(&redis_repo));
        redis_repo
    } else {
        // infra_guard: stable サービスでは Redis 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "session",
            k1s0_server_common::InfraKind::Redis,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("Redis not configured, using InMemory session repository (dev/test bypass)");
        Arc::new(InMemorySessionRepository::new())
    };

    // --- Session Metadata Repository: PostgreSQL or Noop fallback ---
    let metadata_repo: Arc<dyn SessionMetadataRepository> = if let Some(ref db_cfg) = cfg.database {
        info!("connecting to PostgreSQL for session metadata");
        let pool = super::database::create_pool(&db_cfg.url, db_cfg.max_connections).await?;
        info!("PostgreSQL connection pool established");
        Arc::new(SessionMetadataPostgresRepository::new(Arc::new(pool)))
    } else {
        info!("Database not configured, using Noop session metadata repository");
        Arc::new(NoopSessionMetadataRepository)
    };

    // --- Event Publisher: Kafka or Noop fallback ---
    let event_publisher: Arc<dyn SessionEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        info!(brokers = ?kafka_cfg.brokers, "connecting to Kafka");
        let producer = KafkaSessionProducer::new(kafka_cfg)?;
        info!("Kafka producer initialized");
        Arc::new(producer)
    } else {
        info!("Kafka not configured, using Noop event publisher");
        Arc::new(NoopSessionEventPublisher)
    };

    let create_uc = Arc::new(crate::usecase::CreateSessionUseCase::new(
        repo.clone(),
        metadata_repo.clone(),
        event_publisher.clone(),
        cfg.session.default_ttl_seconds,
        cfg.session.max_ttl_seconds,
    ));
    let get_uc = Arc::new(crate::usecase::GetSessionUseCase::new(repo.clone()));
    let refresh_uc = Arc::new(crate::usecase::RefreshSessionUseCase::new(
        repo.clone(),
        cfg.session.max_ttl_seconds,
    ));
    let revoke_uc = Arc::new(crate::usecase::RevokeSessionUseCase::new(
        repo.clone(),
        metadata_repo.clone(),
        event_publisher.clone(),
    ));
    let list_uc = Arc::new(crate::usecase::ListUserSessionsUseCase::new(repo.clone()));
    let revoke_all_uc = Arc::new(crate::usecase::RevokeAllSessionsUseCase::new(
        repo,
        metadata_repo.clone(),
    ));

    // --- Kafka consumer (optional, background task) ---
    if let Some(ref kafka_cfg) = cfg.kafka {
        match super::kafka_consumer::SessionKafkaConsumer::new(kafka_cfg, revoke_all_uc.clone()) {
            Ok(consumer) => {
                let consumer = consumer.with_metrics(Arc::new(
                    k1s0_telemetry::metrics::Metrics::new("k1s0-session-server"),
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

    let grpc_svc = Arc::new(SessionGrpcService::new(
        create_uc.clone(),
        get_uc.clone(),
        refresh_uc.clone(),
        revoke_uc.clone(),
        revoke_all_uc.clone(),
        list_uc.clone(),
        cfg.session.default_ttl_seconds,
    ));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-session-server"));

    // Log metadata and event publisher status
    info!(
        has_metadata_repo = cfg.database.is_some(),
        has_event_publisher = cfg.kafka.is_some(),
        "infrastructure components initialized"
    );

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "session-server",
        &cfg.app.environment,
        cfg.auth
            .as_ref()
            .map(|auth_cfg| -> anyhow::Result<_> {
                // nested 形式の AuthConfig から JWKS 検証器を初期化する
                let jwks = auth_cfg
                    .jwks
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("auth.jwks configuration is required"))?;
                info!(jwks_url = %jwks.url, "initializing JWKS verifier for session-server");
                let base_verifier = k1s0_auth::JwksVerifier::new(
                    &jwks.url,
                    &auth_cfg.jwt.issuer,
                    &auth_cfg.jwt.audience,
                    std::time::Duration::from_secs(jwks.cache_ttl_secs),
                )
                .context("JWKS 検証器の作成に失敗")?;

                // HIGH-002 対応: Redis が構成されている場合は jti チェッカーを有効化する。
                // ログアウト後のJWT再利用防止のために Redis の失効リストを参照する。
                // Redis が構成されていない場合（dev/test）は jti チェックを無効にする（後方互換）。
                // Phase 2: 他サービスで jti チェックを有効化する際は
                // session サービスの Redis lookup パターン（build_jti_checker）を参考にすること。
                let jwks_verifier = if let Some(ref redis_repo) = redis_repo_for_jti {
                    info!("jti ブラックリスト有効: Redis が構成されているため jti チェッカーを JWKS 検証器に設定します");
                    Arc::new(base_verifier.with_jti_checker(build_jti_checker(Arc::clone(redis_repo))))
                } else {
                    info!("jti ブラックリスト無効: Redis が構成されていないため jti チェックをスキップします（dev/test）");
                    Arc::new(base_verifier)
                };

                Ok(crate::adapter::middleware::auth::AuthState {
                    verifier: jwks_verifier,
                })
            })
            .transpose()?,
    )?;

    // gRPC 認証レイヤー（未認証アクセスを middleware レベルでブロック）
    let grpc_auth_layer = GrpcAuthLayer::new(auth_state.clone(), Tier::System, session_grpc_action);

    // Redis が構成されているかを health エンドポイントで判定するために記録
    let redis_configured = cfg.redis.is_some();

    let mut state = crate::adapter::handler::session_handler::AppState {
        create_uc,
        get_uc,
        refresh_uc,
        revoke_uc,
        list_uc,
        revoke_all_uc,
        metadata_repo: metadata_repo.clone(),
        event_publisher: event_publisher.clone(),
        metrics: metrics.clone(),
        auth_state: None,
        redis_configured,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    // Public routes that do not require auth.
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

    use crate::adapter::middleware::auth::auth_middleware;
    use crate::adapter::middleware::rbac::require_permission;

    let api_routes = if let Some(ref auth_st) = state.auth_state {
        let read_routes = axum::Router::new()
            .route(
                "/api/v1/users/{user_id}/sessions",
                axum::routing::get(crate::adapter::handler::session_handler::list_user_sessions),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "sessions", "read",
            )));

        let write_routes = axum::Router::new()
            .route(
                "/api/v1/users/{user_id}/sessions",
                axum::routing::delete(
                    crate::adapter::handler::session_handler::revoke_all_sessions,
                ),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "sessions", "write",
            )));

        let auth_only_routes = axum::Router::new()
            .route(
                "/api/v1/sessions",
                axum::routing::post(crate::adapter::handler::session_handler::create_session),
            )
            .route(
                "/api/v1/sessions/{session_id}",
                axum::routing::get(crate::adapter::handler::session_handler::get_session)
                    .delete(crate::adapter::handler::session_handler::revoke_session),
            )
            .route(
                "/api/v1/sessions/{session_id}/refresh",
                axum::routing::post(crate::adapter::handler::session_handler::refresh_session),
            );

        axum::Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .merge(auth_only_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_st.clone(),
                auth_middleware,
            ))
    } else {
        // Development mode routes without auth.
        axum::Router::new()
            .route(
                "/api/v1/sessions",
                axum::routing::post(crate::adapter::handler::session_handler::create_session),
            )
            .route(
                "/api/v1/sessions/{session_id}",
                axum::routing::get(crate::adapter::handler::session_handler::get_session)
                    .delete(crate::adapter::handler::session_handler::revoke_session),
            )
            .route(
                "/api/v1/sessions/{session_id}/refresh",
                axum::routing::post(crate::adapter::handler::session_handler::refresh_session),
            )
            .route(
                "/api/v1/users/{user_id}/sessions",
                axum::routing::get(crate::adapter::handler::session_handler::list_user_sessions)
                    .delete(crate::adapter::handler::session_handler::revoke_all_sessions),
            )
    };

    let app = public_routes
        .merge(api_routes)
        .with_state(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC service
    use crate::proto::k1s0::system::session::v1::session_service_server::SessionServiceServer;

    let session_tonic = crate::adapter::grpc::SessionServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    // gRPC Health Check Protocol サービスを登録する。
    // readyz エンドポイントや Kubernetes の livenessProbe/readinessProbe が
    // Bearer token なしでヘルスチェックできるようにするため。
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<SessionServiceServer<crate::adapter::grpc::SessionServiceTonic>>()
        .await;

    let grpc_metrics = metrics;
    // gRPC グレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .layer(grpc_auth_layer)
            .add_service(health_service)
            .add_service(SessionServiceServer::new(session_tonic))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    // REST グレースフルシャットダウン設定
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

/// gRPC メソッド名からアクション種別へのマッパー（RBAC 判定に使用）。
fn session_grpc_action(method: &str) -> &'static str {
    match method {
        "CreateSession" | "RefreshSession" | "RevokeSession" | "RevokeAllSessions" => "write",
        _ => "read",
    }
}

async fn metrics_handler(
    axum::extract::State(state): axum::extract::State<
        crate::adapter::handler::session_handler::AppState,
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

struct InMemorySessionRepository {
    sessions: tokio::sync::RwLock<HashMap<String, Session>>,
}

impl InMemorySessionRepository {
    fn new() -> Self {
        Self {
            sessions: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn save(&self, session: &Session) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(id).cloned())
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().find(|s| s.token == token).cloned())
    }

    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions
            .values()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn delete(&self, id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id);
        Ok(())
    }
}
