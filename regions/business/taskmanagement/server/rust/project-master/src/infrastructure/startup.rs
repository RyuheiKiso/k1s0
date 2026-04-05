// サーバー起動処理。
// DB 接続・マイグレーション・認証初期化・ユースケース・ルーター構築・サーバー起動を行う。
// BSL-MED-002 監査対応: RequestBodyLimitLayer を使用して最大リクエストボディサイズを 2MB に制限する。
// BSL-MED-003 監査対応: テレメトリ設定を設定ファイルから読み込むよう変更する。
use std::sync::Arc;
use tracing::info;
use anyhow::Context;
use tower_http::limit::RequestBodyLimitLayer;

use crate::adapter::handler::{AppState, router};
use crate::adapter::middleware::auth::AuthState;
use crate::infrastructure::config::Config;
// CRIT-004 監査対応: connection_url() が Secret<String> を返すため expose_secret() で取り出す
use secrecy::ExposeSecret;
use crate::usecase::{
    event_publisher::NoopProjectMasterEventPublisher,
    get_status_definition_versions::GetStatusDefinitionVersionsUseCase,
    manage_project_types::ManageProjectTypesUseCase,
    manage_status_definitions::ManageStatusDefinitionsUseCase,
    manage_tenant_extensions::ManageTenantExtensionsUseCase,
};
use k1s0_server_common::require_auth_state;

pub async fn run() -> anyhow::Result<()> {
    // 設定読み込み
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config/default.yaml".to_string());
    let cfg = Config::load(&config_path)?;
    info!(service = "project-master", "starting server");

    // BSL-MED-003 監査対応: テレメトリ設定をハードコードではなく設定ファイルから読み込む
    // version は env!("CARGO_PKG_VERSION") を使用して Cargo.toml の値と同期させる（LOW-3 監査対応）
    // trace_endpoint は cfg.observability.trace.enabled が true の場合のみ設定する
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: cfg.app.name.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        tier: "business".to_string(),
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
        .map_err(|e| anyhow::anyhow!("テレメトリ初期化に失敗: {}", e))?;

    // DB 接続（CRIT-004 監査対応: connection_url() は Secret<String> を返すため expose_secret() で取り出す）
    let db_pool = if let Some(ref db_cfg) = cfg.database {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| db_cfg.connection_url().expose_secret().clone());
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_cfg.max_connections)
            .connect(&url)
            .await?;
        // マイグレーション実行
        crate::MIGRATOR.run(&pool).await?;
        info!("database migrations completed");
        Some(pool)
    } else {
        None
    };

    // メトリクス
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("project-master"));

    // リポジトリ
    // H-02 監査対応: 複数リポジトリを一括で初期化するため複雑な型タプルになる
    #[allow(clippy::type_complexity)]
    let (project_type_repo, status_def_repo, tenant_ext_repo, version_repo): (
        Arc<dyn crate::domain::repository::project_type_repository::ProjectTypeRepository>,
        Arc<dyn crate::domain::repository::status_definition_repository::StatusDefinitionRepository>,
        Arc<dyn crate::domain::repository::tenant_extension_repository::TenantExtensionRepository>,
        Arc<dyn crate::domain::repository::version_repository::VersionRepository>,
    ) = if let Some(ref pool) = db_pool {
        (
            Arc::new(crate::infrastructure::persistence::project_type_postgres_repository::ProjectTypePostgresRepository::new(pool.clone())),
            Arc::new(crate::infrastructure::persistence::status_definition_postgres_repository::StatusDefinitionPostgresRepository::new(pool.clone())),
            Arc::new(crate::infrastructure::persistence::tenant_extension_postgres_repository::TenantExtensionPostgresRepository::new(pool.clone())),
            Arc::new(crate::infrastructure::persistence::version_postgres_repository::VersionPostgresRepository::new(pool.clone())),
        )
    } else {
        anyhow::bail!("database configuration is required");
    };

    // Kafka プロデューサー（任意）
    let event_publisher: Arc<dyn crate::usecase::event_publisher::ProjectMasterEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            match crate::infrastructure::messaging::kafka_producer::ProjectMasterKafkaProducer::new(kafka_cfg) {
                Ok(p) => {
                    info!("kafka producer initialized");
                    Arc::new(p)
                }
                Err(e) => {
                    tracing::warn!("failed to initialize kafka producer: {}", e);
                    Arc::new(NoopProjectMasterEventPublisher)
                }
            }
        } else {
            Arc::new(NoopProjectMasterEventPublisher)
        };

    // ユースケース
    let manage_project_types_uc = Arc::new(ManageProjectTypesUseCase::new(
        project_type_repo,
        event_publisher.clone(),
    ));
    let manage_status_definitions_uc = Arc::new(ManageStatusDefinitionsUseCase::new(
        status_def_repo,
        event_publisher.clone(),
    ));
    let get_versions_uc = Arc::new(GetStatusDefinitionVersionsUseCase::new(version_repo));
    let manage_tenant_extensions_uc = Arc::new(ManageTenantExtensionsUseCase::new(
        tenant_ext_repo,
        event_publisher,
    ));

    // 認証状態の初期化（auth 設定がある場合は JWKS 検証器を生成する）
    let auth_state_opt = cfg.auth
        .as_ref()
        .map(|auth_cfg| -> anyhow::Result<AuthState> {
            // JWKS URL を取得（nested 形式: auth.jwks.url）
            let jwks_url = auth_cfg.jwks.as_ref().map(|j| j.url.as_str()).unwrap_or_default();
            let cache_ttl = auth_cfg.jwks.as_ref().map(|j| j.cache_ttl_secs).unwrap_or(300);
            info!(jwks_url = %jwks_url, "initializing JWKS verifier for project-master");
            let verifier = Arc::new(
                k1s0_auth::JwksVerifier::new(
                    jwks_url,
                    &auth_cfg.jwt.issuer,
                    &auth_cfg.jwt.audience,
                    std::time::Duration::from_secs(cache_ttl),
                )
                .context("JWKS 検証器の作成に失敗")?,
            );
            Ok(AuthState { verifier })
        })
        .transpose()?;

    // 認証設定が未指定の場合は dev/test 環境かつ ALLOW_INSECURE_NO_AUTH=true のみ許可する
    let auth_state = require_auth_state(
        "project-master",
        &cfg.app.environment,
        auth_state_opt,
    )?;

    // AppState + ルーター
    let state = AppState {
        manage_project_types_uc,
        manage_status_definitions_uc,
        get_versions_uc,
        manage_tenant_extensions_uc,
        metrics: metrics.clone(),
        auth_state,
    };
    let app = router(state);
    // BSL-MED-002 監査対応: 大きなリクエストボディによるメモリ枯渇攻撃を防ぐため最大2MBに制限する
    let app = app.layer(RequestBodyLimitLayer::new(2 * 1024 * 1024));

    // サーバー起動
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    info!(addr = %addr, "REST server listening");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    k1s0_telemetry::shutdown();
    Ok(())
}
