use std::net::SocketAddr;
use std::sync::Arc;

use tower::limit::ConcurrencyLimitLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::info;

use super::auth::JwksVerifier;
use super::config::Config;
use super::grpc::{
    AuthGrpcClient, ConfigGrpcClient, FeatureFlagGrpcClient, NavigationGrpcClient,
    NotificationGrpcClient, SchedulerGrpcClient, ServiceCatalogGrpcClient, SessionGrpcClient,
    TenantGrpcClient, VaultGrpcClient, WorkflowGrpcClient,
};
use crate::adapter::graphql_handler::{self, GatewayClients, GatewayResolvers};
use crate::adapter::middleware::ratelimit_middleware::RateLimitLayer;
use crate::usecase::{
    AuthMutationResolver, AuthQueryResolver, ConfigQueryResolver, FeatureFlagQueryResolver,
    NavigationQueryResolver, NotificationMutationResolver, NotificationQueryResolver,
    SchedulerMutationResolver, SchedulerQueryResolver, ServiceCatalogMutationResolver,
    ServiceCatalogQueryResolver, SessionMutationResolver, SessionQueryResolver,
    SubscriptionResolver, TenantMutationResolver, TenantQueryResolver, VaultMutationResolver,
    VaultQueryResolver, WorkflowMutationResolver, WorkflowQueryResolver,
};

pub async fn run() -> anyhow::Result<()> {
    // --- Telemetry ---
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-graphql-gateway-server".to_string(),
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

    // --- Config ---
    cfg.validate()?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting graphql-gateway server"
    );

    // --- gRPC クライアント ---
    // connect_lazy() により起動時のバックエンド接続依存を排除する。
    // 実際の接続は最初のRPCリクエスト時に確立されるため、バックエンドサービスの起動順序に依存しない。
    let tenant_client = Arc::new(TenantGrpcClient::new(&cfg.backends.tenant)?);
    let feature_flag_client =
        Arc::new(FeatureFlagGrpcClient::new(&cfg.backends.featureflag)?);
    let config_client = Arc::new(ConfigGrpcClient::new(&cfg.backends.config)?);
    let navigation_client =
        Arc::new(NavigationGrpcClient::new(&cfg.backends.navigation)?);
    let service_catalog_client =
        Arc::new(ServiceCatalogGrpcClient::new(&cfg.backends.service_catalog)?);
    let auth_client = Arc::new(AuthGrpcClient::new(&cfg.backends.auth)?);
    let session_client = Arc::new(SessionGrpcClient::new(&cfg.backends.session)?);
    let vault_client = Arc::new(VaultGrpcClient::new(&cfg.backends.vault)?);
    let scheduler_client = Arc::new(SchedulerGrpcClient::new(&cfg.backends.scheduler)?);
    let notification_client =
        Arc::new(NotificationGrpcClient::new(&cfg.backends.notification)?);
    let workflow_client = Arc::new(WorkflowGrpcClient::new(&cfg.backends.workflow)?);

    // --- JWT 検証 ---
    // new() が Result を返すようになったため ? で伝播する
    let jwks_verifier = Arc::new(JwksVerifier::new(cfg.auth.jwks_url.clone())?);

    // --- Metrics ---
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-graphql-gateway-server",
    ));

    // --- Resolver DI ---
    let resolvers = GatewayResolvers {
        tenant_query: Arc::new(TenantQueryResolver::new(tenant_client.clone())),
        feature_flag_query: Arc::new(FeatureFlagQueryResolver::new(feature_flag_client.clone())),
        config_query: Arc::new(ConfigQueryResolver::new(config_client.clone())),
        navigation_query: Arc::new(NavigationQueryResolver::new(navigation_client.clone())),
        service_catalog_query: Arc::new(ServiceCatalogQueryResolver::new(
            service_catalog_client.clone(),
        )),
        tenant_mutation: Arc::new(TenantMutationResolver::new(tenant_client.clone())),
        service_catalog_mutation: Arc::new(ServiceCatalogMutationResolver::new(
            service_catalog_client.clone(),
        )),
        subscription: Arc::new(SubscriptionResolver::new(
            config_client.clone(),
            tenant_client.clone(),
            feature_flag_client.clone(),
        )),
        auth_query: Arc::new(AuthQueryResolver::new(auth_client.clone())),
        auth_mutation: Arc::new(AuthMutationResolver::new(auth_client.clone())),
        session_query: Arc::new(SessionQueryResolver::new(session_client.clone())),
        session_mutation: Arc::new(SessionMutationResolver::new(session_client.clone())),
        vault_query: Arc::new(VaultQueryResolver::new(vault_client.clone())),
        vault_mutation: Arc::new(VaultMutationResolver::new(vault_client.clone())),
        scheduler_query: Arc::new(SchedulerQueryResolver::new(scheduler_client.clone())),
        scheduler_mutation: Arc::new(SchedulerMutationResolver::new(scheduler_client.clone())),
        notification_query: Arc::new(NotificationQueryResolver::new(notification_client.clone())),
        notification_mutation: Arc::new(NotificationMutationResolver::new(
            notification_client.clone(),
        )),
        workflow_query: Arc::new(WorkflowQueryResolver::new(workflow_client.clone())),
        workflow_mutation: Arc::new(WorkflowMutationResolver::new(workflow_client.clone())),
    };

    let clients = GatewayClients {
        tenant: tenant_client,
        feature_flag: feature_flag_client,
        config: config_client,
        navigation: navigation_client,
        service_catalog: service_catalog_client,
        auth: auth_client,
        session: session_client,
        vault: vault_client,
        scheduler: scheduler_client,
        notification: notification_client,
        workflow: workflow_client,
    };

    // --- レート制限クライアント ---
    // 設定で有効化されている場合、ratelimit gRPC サービスに接続するクライアントを生成する。
    // 無効の場合はレート制限レイヤーを追加しない。
    let ratelimit_layer = if cfg.ratelimit.enabled {
        info!(
            server_url = %cfg.ratelimit.server_url,
            scope = %cfg.ratelimit.scope,
            requests_per_window = cfg.ratelimit.requests_per_window,
            window_secs = cfg.ratelimit.window_secs,
            "レート制限を有効化"
        );
        let ratelimit_client =
            k1s0_ratelimit_client::GrpcRateLimitClient::new(cfg.ratelimit.server_url.clone())
                .await
                .map_err(|e| anyhow::anyhow!("ratelimit クライアントの作成に失敗: {}", e))?;
        Some(RateLimitLayer::new(
            Arc::new(ratelimit_client),
            cfg.ratelimit.scope.clone(),
        ))
    } else {
        info!("レート制限は無効化されています");
        None
    };

    // --- Router ---
    // CorrelationLayerを追加してリクエスト間の相関IDを伝播する
    // ConcurrencyLimitLayerで同時リクエスト数を制限する（最大100並列）
    // RequestBodyLimitLayerでリクエストボディサイズを2MBに制限する
    // RateLimitLayerでリクエストレートを制限する（有効時のみ）
    let router = graphql_handler::router(
        jwks_verifier,
        clients,
        resolvers,
        cfg.graphql.clone(),
        metrics,
    );

    // レート制限が有効な場合のみレイヤーを追加する
    let app = if let Some(rl_layer) = ratelimit_layer {
        router
            .layer(rl_layer)
            .layer(k1s0_correlation::layer::CorrelationLayer::new())
            .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
            .layer(ConcurrencyLimitLayer::new(100))
    } else {
        router
            .layer(k1s0_correlation::layer::CorrelationLayer::new())
            .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
            .layer(ConcurrencyLimitLayer::new(100))
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("graphql-gateway starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = k1s0_server_common::shutdown::shutdown_signal().await;
        })
        .await?;

    info!("graphql-gateway exited");

    // テレメトリのエクスポーターをフラッシュしてシャットダウンする
    k1s0_telemetry::shutdown();

    Ok(())
}
