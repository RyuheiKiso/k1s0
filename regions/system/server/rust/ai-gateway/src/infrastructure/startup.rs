use anyhow::Context;
// AI Gatewayサーバーの起動処理。
// 設定読み込み、テレメトリ初期化、依存関係の組み立て、REST/gRPCサーバーの起動を行う。

use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use super::config::Config;
use crate::domain::repository::ModelRepository;
use crate::domain::repository::RoutingRuleRepository;
use crate::domain::repository::UsageRepository;

/// バインドアドレスを解決する。
async fn resolve_bind_addr(host: &str, port: u16) -> anyhow::Result<SocketAddr> {
    tokio::net::lookup_host((host, port))
        .await?
        .next()
        .ok_or_else(|| anyhow::anyhow!("failed to resolve bind address {host}:{port}"))
}

/// AI Gatewayサーバーのメインエントリポイント。
/// 設定ファイルを読み込み、依存関係を構築し、REST/gRPCサーバーを起動する。
pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    // テレメトリ初期化
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-ai-gateway-server".to_string(),
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
        "starting AI Gateway server"
    );

    // --- リポジトリの構築 ---
    let (model_repo, usage_repo, routing_rule_repo): (
        Arc<dyn ModelRepository>,
        Arc<dyn UsageRepository>,
        Arc<dyn RoutingRuleRepository>,
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
            Arc::new(crate::adapter::repository::ModelPostgresRepository::new(
                pool.clone(),
            )),
            Arc::new(crate::adapter::repository::UsagePostgresRepository::new(
                pool.clone(),
            )),
            Arc::new(crate::adapter::repository::RoutingRulePostgresRepository::new(pool)),
        )
    } else {
        info!("no database config found, using in-memory repositories");
        (
            Arc::new(crate::adapter::repository::ModelPostgresRepository::in_memory()),
            Arc::new(crate::adapter::repository::UsagePostgresRepository::in_memory()),
            Arc::new(crate::adapter::repository::RoutingRulePostgresRepository::in_memory()),
        )
    };

    // --- LLMクライアントの構築 ---
    let llm_client = Arc::new(if let Some(ref llm_cfg) = cfg.llm {
        info!(base_url = %llm_cfg.base_url, "LLMクライアント初期化");
        super::llm_client::LlmClient::new(llm_cfg.base_url.clone(), llm_cfg.api_key.clone())
    } else {
        info!("LLM設定なし、デフォルトURL使用");
        super::llm_client::LlmClient::new("https://api.openai.com/v1".to_string(), String::new())
    });

    // --- ドメインサービスの構築 ---
    let guardrail_svc = Arc::new(crate::domain::service::GuardrailService::new());
    let routing_svc = Arc::new(crate::domain::service::RoutingService::new(
        model_repo.clone(),
        routing_rule_repo,
    ));

    // --- ユースケースの構築 ---
    let complete_uc = Arc::new(crate::usecase::CompleteUseCase::new(
        guardrail_svc,
        routing_svc,
        llm_client.clone(),
        usage_repo.clone(),
    ));
    let embed_uc = Arc::new(crate::usecase::EmbedUseCase::new(llm_client));
    let list_models_uc = Arc::new(crate::usecase::ListModelsUseCase::new(model_repo));
    let get_usage_uc = Arc::new(crate::usecase::GetUsageUseCase::new(usage_repo));

    // --- gRPCサービスの構築 ---
    let grpc_svc = Arc::new(crate::adapter::grpc::AiGatewayGrpcService::new(
        complete_uc.clone(),
        embed_uc.clone(),
        list_models_uc.clone(),
        get_usage_uc.clone(),
    ));

    let _grpc_tonic = crate::adapter::grpc::AiGatewayServiceTonic::new(grpc_svc);

    // --- メトリクス ---
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-ai-gateway-server",
    ));

    // --- 認証 ---
    let auth_state = k1s0_server_common::require_auth_state(
        "ai-gateway-server",
        &cfg.app.environment,
        cfg.auth.as_ref().map(|auth_cfg| -> anyhow::Result<_> {
            info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for ai-gateway-server");
            let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
                &auth_cfg.jwks_url,
                &auth_cfg.issuer,
                &auth_cfg.audience,
                std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
            ).context("JWKS 検証器の作成に失敗")?);
            Ok(crate::adapter::middleware::auth::AuthState {
                verifier: jwks_verifier,
            })
        }).transpose()?,
    )?;

    let mut handler_state = crate::adapter::handler::AppState {
        complete_uc,
        embed_uc,
        list_models_uc,
        get_usage_uc,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        handler_state = handler_state.with_auth(auth_st);
    }
    let grpc_auth_state = handler_state.auth_state.clone();

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

    // --- gRPCサーバー ---
    // proto生成コードが利用可能になったらサービス登録を有効化する
    // 現時点ではRESTサーバーのみ起動する
    let grpc_addr = resolve_bind_addr(&cfg.server.host, cfg.server.grpc_port).await?;
    info!("gRPC server (placeholder) addr: {}", grpc_addr);
    let _grpc_auth_layer =
        crate::adapter::middleware::grpc_auth::GrpcAuthLayer::new(grpc_auth_state);
    let _grpc_metrics = metrics;

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = k1s0_server_common::shutdown::shutdown_signal().await;
        })
        .await?;

    // テレメトリのエクスポーターをフラッシュしてシャットダウンする
    k1s0_telemetry::shutdown();

    Ok(())
}
