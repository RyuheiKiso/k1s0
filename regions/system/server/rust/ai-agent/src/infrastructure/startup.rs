// サーバー起動処理
// REST/gRPCサーバーの初期化と起動を管理する

use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use crate::adapter::grpc::AiAgentGrpcService;
use crate::domain::repository::{AgentRepository, ExecutionRepository};
use crate::domain::service::{ReActEngine, ToolRegistry};
use super::config::Config;
use super::in_memory::{InMemoryAgentRepository, InMemoryExecutionRepository};
use k1s0_bb_ai_client::traits::AiClient;

/// ソケットアドレスを解決する
async fn resolve_bind_addr(host: &str, port: u16) -> anyhow::Result<SocketAddr> {
    tokio::net::lookup_host((host, port))
        .await?
        .next()
        .ok_or_else(|| anyhow::anyhow!("failed to resolve bind address {host}:{port}"))
}

/// シャットダウンシグナルを待機する
async fn shutdown_signal() -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut terminate = signal(SignalKind::terminate())?;
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
    }

    Ok(())
}

/// サーバーを起動する
pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    // テレメトリを初期化する
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-ai-agent-server".to_string(),
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
        "starting ai-agent server"
    );

    // --- Repository ---
    let (agent_repo, execution_repo): (
        Arc<dyn AgentRepository>,
        Arc<dyn ExecutionRepository>,
    ) = if let Some(ref db_cfg) = cfg.database {
        let pool = Arc::new(
            super::database::create_pool(
                &db_cfg.connection_url(),
                db_cfg.max_open_conns,
                db_cfg.max_idle_conns,
                &db_cfg.conn_max_lifetime,
            )
            .await?,
        );
        info!("connected to PostgreSQL database");

        (
            Arc::new(crate::adapter::repository::AgentPostgresRepository::new(
                pool.clone(),
            )),
            Arc::new(crate::adapter::repository::ExecutionPostgresRepository::new(
                pool,
            )),
        )
    } else {
        info!("no database config found, using in-memory repositories");
        (
            Arc::new(InMemoryAgentRepository::new()),
            Arc::new(InMemoryExecutionRepository::new()),
        )
    };

    // --- AI Client ---
    let ai_client: Arc<dyn AiClient> = if let Some(ref gw_cfg) = cfg.ai_gateway {
        info!(endpoint = %gw_cfg.internal_endpoint, "using AI Gateway client");
        Arc::new(super::ai_gateway_client::AiGatewayClient::new(
            &gw_cfg.internal_endpoint,
        ))
    } else {
        info!("no AI Gateway config found, using in-memory AI client");
        Arc::new(k1s0_bb_ai_client::InMemoryAiClient::new(
            vec![k1s0_bb_ai_client::CompleteResponse {
                id: "default".to_string(),
                model: "default".to_string(),
                content: r#"{"action": "final_answer", "output": "No AI Gateway configured"}"#.to_string(),
                prompt_tokens: 0,
                completion_tokens: 0,
            }],
            vec![],
        ))
    };

    // --- ReAct Engine ---
    let tool_registry = ToolRegistry::new();
    let react_engine = Arc::new(ReActEngine::new(tool_registry));

    // --- Use Cases ---
    let create_agent_uc = Arc::new(crate::usecase::CreateAgentUseCase::new(
        agent_repo.clone(),
    ));
    let execute_agent_uc = Arc::new(crate::usecase::ExecuteAgentUseCase::new(
        agent_repo.clone(),
        execution_repo.clone(),
        react_engine,
        ai_client,
    ));
    let list_executions_uc = Arc::new(crate::usecase::ListExecutionsUseCase::new(
        execution_repo.clone(),
    ));
    let review_step_uc = Arc::new(crate::usecase::ReviewStepUseCase::new(
        execution_repo,
    ));

    // --- gRPC Service ---
    let grpc_svc = Arc::new(AiAgentGrpcService::new(
        execute_agent_uc.clone(),
        review_step_uc.clone(),
    ));

    // --- Metrics ---
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-ai-agent-server",
    ));

    // --- Auth ---
    let auth_state = k1s0_server_common::require_auth_state(
        "ai-agent-server",
        &cfg.app.environment,
        cfg.auth.as_ref().map(|auth_cfg| {
            info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for ai-agent-server");
            let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
                &auth_cfg.jwks_url,
                &auth_cfg.issuer,
                &auth_cfg.audience,
                std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
            ));
            crate::adapter::middleware::auth::AgentAuthState {
                verifier: jwks_verifier,
            }
        }),
    )?;

    let mut handler_state = crate::adapter::handler::AppState {
        create_agent_uc,
        execute_agent_uc,
        list_executions_uc,
        review_step_uc,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        handler_state = handler_state.with_auth(auth_st);
    }
    let grpc_auth_state = handler_state.auth_state.clone();

    // --- REST Router ---
    // CorrelationLayerを追加してリクエスト間の相関IDを伝播する
    let app = crate::adapter::handler::router(
        handler_state,
        cfg.observability.metrics.enabled,
        &cfg.observability.metrics.path,
    )
    .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
    .layer(k1s0_correlation::layer::CorrelationLayer::new());

    let rest_addr = resolve_bind_addr(&cfg.server.host, cfg.server.port).await?;
    info!("REST server starting on {}", rest_addr);

    // --- gRPC Server ---
    use crate::proto::k1s0::system::ai_agent::v1::ai_agent_service_server::AiAgentServiceServer;

    let agent_tonic = crate::adapter::grpc::AiAgentServiceTonic::new(grpc_svc);

    let grpc_addr = resolve_bind_addr(&cfg.server.host, cfg.server.grpc_port).await?;
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_auth_layer =
        crate::adapter::middleware::grpc_auth::GrpcAuthLayer::new(grpc_auth_state);
    let grpc_shutdown = shutdown_signal();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(AiAgentServiceServer::new(agent_tonic))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app).with_graceful_shutdown(async {
        let _ = shutdown_signal().await;
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

    // テレメトリのエクスポーターをフラッシュしてシャットダウンする
    k1s0_telemetry::shutdown();

    Ok(())
}
