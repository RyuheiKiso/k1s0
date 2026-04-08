use anyhow::Context;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use super::config::{parse_pool_duration, Config};
use super::file_storage::FileStorage;
use crate::adapter::handler::{self, AppState, ValidateTokenUseCase};
use crate::infrastructure::signature_verifier::{CosignVerifier, StubCosignVerifier, SubprocessCosignVerifier};
use crate::adapter::repository::app_postgres::AppPostgresRepository;
use crate::adapter::repository::download_stats_postgres::DownloadStatsPostgresRepository;
use crate::adapter::repository::version_postgres::VersionPostgresRepository;
use crate::usecase;

// HIGH-001 監査対応: 起動処理は構造上行数が多くなるため許容する
#[allow(clippy::too_many_lines, clippy::items_after_statements)]
pub async fn run() -> anyhow::Result<()> {
    // Load config
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-app-registry".to_string(),
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
        "starting app-registry server"
    );

    // Token verifier (JWKS verifier if configured, stub otherwise)
    let token_verifier: Arc<dyn super::TokenVerifier> = if let Some(jwks_config) = &cfg.auth.jwks {
        let jwks_verifier = Arc::new(
            k1s0_auth::JwksVerifier::new(
                &jwks_config.url,
                &cfg.auth.jwt.issuer,
                &cfg.auth.jwt.audience,
                std::time::Duration::from_secs(jwks_config.cache_ttl_secs),
            )
            .context("JWKS 検証器の作成に失敗")?,
        );
        Arc::new(super::JwksVerifierAdapter::new(jwks_verifier))
    } else {
        Arc::new(StubTokenVerifier)
    };

    // Database pool (optional)
    let db_pool = if let Some(ref db_config) = cfg.database {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_config.connection_url());
        info!("connecting to database");
        let lifetime = parse_pool_duration(&db_config.conn_max_lifetime)
            .unwrap_or_else(|| std::time::Duration::from_secs(300));
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_config.max_open_conns)
            .min_connections(db_config.max_idle_conns.min(db_config.max_open_conns))
            .idle_timeout(Some(lifetime))
            .max_lifetime(Some(lifetime))
            .connect(&url)
            .await?;
        info!("database connection pool established");
        Some(pool)
    } else if let Ok(url) = std::env::var("DATABASE_URL") {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(25)
            .min_connections(5)
            .idle_timeout(Some(std::time::Duration::from_secs(300)))
            .max_lifetime(Some(std::time::Duration::from_secs(300)))
            .connect(&url)
            .await?;
        info!("database connection pool established from DATABASE_URL");
        Some(pool)
    } else {
        // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "app-registry",
            k1s0_server_common::InfraKind::Database,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no database configured, using in-memory/stub repositories (dev/test bypass)");
        None
    };

    // ファイルストレージ（ローカルFS）の初期化
    let file_storage = Arc::new(FileStorage::new(&cfg.storage.path));
    info!(storage_path = %cfg.storage.path, "initializing local file storage");

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-app-registry"));

    // Repositories
    let app_repo: Arc<dyn crate::domain::repository::AppRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(AppPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(StubAppRepository)
        };

    let version_repo: Arc<dyn crate::domain::repository::VersionRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(VersionPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(StubVersionRepository)
        };

    let download_stats_repo: Arc<dyn crate::domain::repository::DownloadStatsRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(DownloadStatsPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(StubDownloadStatsRepository)
        };

    // Use cases
    let list_apps_uc = Arc::new(usecase::ListAppsUseCase::new(app_repo.clone()));
    let get_app_uc = Arc::new(usecase::GetAppUseCase::new(app_repo.clone()));
    let create_app_uc = Arc::new(usecase::CreateAppUseCase::new(app_repo.clone()));
    let update_app_uc = Arc::new(usecase::UpdateAppUseCase::new(app_repo.clone()));
    let delete_app_uc = Arc::new(usecase::DeleteAppUseCase::new(app_repo.clone()));
    let list_versions_uc = Arc::new(usecase::ListVersionsUseCase::new(
        app_repo.clone(),
        version_repo.clone(),
    ));
    let create_version_uc = Arc::new(usecase::CreateVersionUseCase::new(version_repo.clone()));
    let delete_version_uc = Arc::new(usecase::DeleteVersionUseCase::new(
        app_repo.clone(),
        version_repo.clone(),
    ));
    let get_latest_uc = Arc::new(usecase::GetLatestUseCase::new(
        app_repo.clone(),
        version_repo.clone(),
    ));
    let get_download_stats_uc = Arc::new(usecase::GetDownloadStatsUseCase::new(
        app_repo.clone(),
        version_repo.clone(),
        download_stats_repo.clone(),
    ));
    let generate_download_url_uc = Arc::new(usecase::GenerateDownloadUrlUseCase::new(
        app_repo.clone(),
        version_repo.clone(),
        download_stats_repo.clone(),
        file_storage.clone(),
    ));

    let validate_token_uc = Arc::new(ValidateTokenUseCase::new(
        token_verifier,
        cfg.auth.jwt.issuer,
        cfg.auth.jwt.audience,
    ));

    // STATIC-CRITICAL-002: Cosign 署名検証器の初期化
    // verify_enabled が true の場合は cosign CLI を使用、false の場合はスタブを使用する
    let cosign_verifier: Arc<dyn CosignVerifier> = if cfg.cosign.verify_enabled {
        info!(
            public_key_path = %cfg.cosign.public_key_path,
            "Cosign 署名検証を有効化します（SubprocessCosignVerifier）"
        );
        Arc::new(SubprocessCosignVerifier::new(
            cfg.cosign.public_key_path.clone(),
        ))
    } else {
        info!("Cosign 署名検証は無効です（StubCosignVerifier）。本番環境では cosign.verify_enabled: true を設定してください");
        Arc::new(StubCosignVerifier)
    };

    // AppState
    let state = AppState {
        list_apps_uc,
        get_app_uc,
        create_app_uc,
        update_app_uc,
        delete_app_uc,
        list_versions_uc,
        create_version_uc,
        delete_version_uc,
        get_latest_uc,
        get_download_stats_uc,
        generate_download_url_uc,
        validate_token_uc,
        cosign_verifier,
        metrics: metrics.clone(),
        db_pool,
    };

    // Router
    let app = handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    // REST グレースフルシャットダウン設定
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = k1s0_server_common::shutdown::shutdown_signal().await;
        })
        .await?;
    // テレメトリのシャットダウン処理
    k1s0_telemetry::shutdown();

    Ok(())
}

// --- Stub implementations for dev mode ---

struct StubTokenVerifier;

#[async_trait::async_trait]
impl super::TokenVerifier for StubTokenVerifier {
    async fn verify_token(
        &self,
        _token: &str,
    ) -> anyhow::Result<crate::domain::entity::claims::Claims> {
        anyhow::bail!("stub token verifier: not implemented")
    }
}

struct StubAppRepository;

// CRIT-004 監査対応: tenant_id パラメータを追加（スタブ実装のため無視）
#[async_trait::async_trait]
impl crate::domain::repository::AppRepository for StubAppRepository {
    async fn list(
        &self,
        _tenant_id: &str,
        _category: Option<String>,
        _search: Option<String>,
    ) -> anyhow::Result<Vec<crate::domain::entity::app::App>> {
        Ok(vec![])
    }

    async fn find_by_id(
        &self,
        _tenant_id: &str,
        _id: &str,
    ) -> anyhow::Result<Option<crate::domain::entity::app::App>> {
        Ok(None)
    }

    async fn create(
        &self,
        _tenant_id: &str,
        app: &crate::domain::entity::app::App,
    ) -> anyhow::Result<crate::domain::entity::app::App> {
        Ok(app.clone())
    }

    async fn update(
        &self,
        _tenant_id: &str,
        app: &crate::domain::entity::app::App,
    ) -> anyhow::Result<crate::domain::entity::app::App> {
        Ok(app.clone())
    }

    async fn delete(&self, _tenant_id: &str, _id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

struct StubVersionRepository;

#[async_trait::async_trait]
impl crate::domain::repository::VersionRepository for StubVersionRepository {
    async fn list_by_app(
        &self,
        _app_id: &str,
    ) -> anyhow::Result<Vec<crate::domain::entity::version::AppVersion>> {
        Ok(vec![])
    }

    async fn create(
        &self,
        version: &crate::domain::entity::version::AppVersion,
    ) -> anyhow::Result<crate::domain::entity::version::AppVersion> {
        Ok(version.clone())
    }

    async fn delete(
        &self,
        _app_id: &str,
        _version: &str,
        _platform: &crate::domain::entity::platform::Platform,
        _arch: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

struct StubDownloadStatsRepository;

#[async_trait::async_trait]
impl crate::domain::repository::DownloadStatsRepository for StubDownloadStatsRepository {
    async fn record(
        &self,
        _stat: &crate::domain::entity::download_stat::DownloadStat,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn count_by_app(&self, _app_id: &str) -> anyhow::Result<i64> {
        Ok(0)
    }

    async fn count_by_version(&self, _app_id: &str, _version: &str) -> anyhow::Result<i64> {
        Ok(0)
    }
}
