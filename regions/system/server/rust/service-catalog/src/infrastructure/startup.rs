use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use crate::adapter::handler::{self, AppState, ValidateTokenUseCase};
use crate::adapter::repository::dependency_postgres::DependencyPostgresRepository;
use crate::adapter::repository::doc_postgres::DocPostgresRepository;
use crate::adapter::repository::health_postgres::HealthPostgresRepository;
use crate::adapter::repository::scorecard_postgres::ScorecardPostgresRepository;
use crate::adapter::repository::service_postgres::ServicePostgresRepository;
use crate::adapter::repository::team_postgres::TeamPostgresRepository;
use super::config::{Config, parse_pool_duration};
use super::health_collector::HealthCollector;

pub async fn run() -> anyhow::Result<()> {
    // Load config
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-service-catalog".to_string(),
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
        "starting service-catalog server"
    );

    // Token verifier (JWKS verifier if configured, stub otherwise)
    let token_verifier: Arc<dyn super::TokenVerifier> =
        if let Some(jwks_config) = &cfg.auth.jwks {
            let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
                &jwks_config.url,
                &cfg.auth.jwt.issuer,
                &cfg.auth.jwt.audience,
                std::time::Duration::from_secs(jwks_config.cache_ttl_secs),
            ));
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
        info!("no database configured, using in-memory/stub repositories");
        None
    };

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-service-catalog"));

    // Repositories
    let service_repo: Arc<dyn crate::domain::repository::ServiceRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(ServicePostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(StubServiceRepository)
        };

    let team_repo: Arc<dyn crate::domain::repository::TeamRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(TeamPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(StubTeamRepository)
        };

    let dep_repo: Arc<dyn crate::domain::repository::DependencyRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(DependencyPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(StubDependencyRepository)
        };

    let health_repo: Arc<dyn crate::domain::repository::HealthRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(HealthPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(StubHealthRepository)
        };

    let doc_repo: Arc<dyn crate::domain::repository::DocRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(DocPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(StubDocRepository)
        };

    let scorecard_repo: Arc<dyn crate::domain::repository::ScorecardRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(ScorecardPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(StubScorecardRepository)
        };

    // Use cases
    let list_services_uc = Arc::new(crate::usecase::ListServicesUseCase::new(service_repo.clone()));
    let get_service_uc = Arc::new(crate::usecase::GetServiceUseCase::new(service_repo.clone()));
    let register_service_uc = Arc::new(crate::usecase::RegisterServiceUseCase::new(
        service_repo.clone(),
        team_repo.clone(),
    ));
    let update_service_uc = Arc::new(crate::usecase::UpdateServiceUseCase::new(service_repo.clone()));
    let delete_service_uc = Arc::new(crate::usecase::DeleteServiceUseCase::new(service_repo.clone()));
    let manage_deps_uc = Arc::new(crate::usecase::ManageDependenciesUseCase::new(dep_repo.clone()));
    let health_status_uc = Arc::new(crate::usecase::HealthStatusUseCase::new(health_repo.clone()));
    let manage_docs_uc = Arc::new(crate::usecase::ManageDocsUseCase::new(doc_repo.clone()));
    let get_scorecard_uc = Arc::new(crate::usecase::GetScorecardUseCase::new(scorecard_repo.clone()));
    let search_services_uc = Arc::new(crate::usecase::SearchServicesUseCase::new(service_repo.clone()));
    let list_teams_uc = Arc::new(crate::usecase::ListTeamsUseCase::new(team_repo.clone()));
    let get_team_uc = Arc::new(crate::usecase::GetTeamUseCase::new(team_repo.clone()));
    let create_team_uc = Arc::new(crate::usecase::CreateTeamUseCase::new(team_repo.clone()));
    let update_team_uc = Arc::new(crate::usecase::UpdateTeamUseCase::new(team_repo.clone()));
    let delete_team_uc = Arc::new(crate::usecase::DeleteTeamUseCase::new(team_repo.clone()));

    let validate_token_uc = Arc::new(ValidateTokenUseCase::new(
        token_verifier,
        cfg.auth.jwt.issuer,
        cfg.auth.jwt.audience,
    ));

    // Health collector background task
    if db_pool.is_some() {
        let collector = Arc::new(HealthCollector::new(
            service_repo.clone(),
            health_repo.clone(),
            cfg.health_collector.clone(),
        ));
        tokio::spawn(async move {
            collector.run().await;
        });
    }

    // AppState
    let state = AppState {
        list_services_uc,
        get_service_uc,
        register_service_uc,
        update_service_uc,
        delete_service_uc,
        manage_deps_uc,
        health_status_uc,
        manage_docs_uc,
        get_scorecard_uc,
        search_services_uc,
        list_teams_uc,
        get_team_uc,
        create_team_uc,
        update_team_uc,
        delete_team_uc,
        validate_token_uc,
        metrics: metrics.clone(),
        db_pool,
    };

    // Router
    let app = handler::router(state).layer(k1s0_telemetry::MetricsLayer::new(metrics));

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    axum::serve(listener, app).await?;

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

struct StubServiceRepository;

#[async_trait::async_trait]
impl crate::domain::repository::ServiceRepository for StubServiceRepository {
    async fn list(
        &self,
        _filters: crate::domain::repository::service_repository::ServiceListFilters,
    ) -> anyhow::Result<Vec<crate::domain::entity::service::Service>> {
        Ok(vec![])
    }

    async fn find_by_id(
        &self,
        _id: uuid::Uuid,
    ) -> anyhow::Result<Option<crate::domain::entity::service::Service>> {
        Ok(None)
    }

    async fn create(
        &self,
        service: &crate::domain::entity::service::Service,
    ) -> anyhow::Result<crate::domain::entity::service::Service> {
        Ok(service.clone())
    }

    async fn update(
        &self,
        service: &crate::domain::entity::service::Service,
    ) -> anyhow::Result<crate::domain::entity::service::Service> {
        Ok(service.clone())
    }

    async fn delete(&self, _id: uuid::Uuid) -> anyhow::Result<()> {
        Ok(())
    }

    async fn search(
        &self,
        _query: Option<String>,
        _tags: Option<Vec<String>>,
        _tier: Option<crate::domain::entity::service::ServiceTier>,
    ) -> anyhow::Result<Vec<crate::domain::entity::service::Service>> {
        Ok(vec![])
    }
}

struct StubTeamRepository;

#[async_trait::async_trait]
impl crate::domain::repository::TeamRepository for StubTeamRepository {
    async fn list(&self) -> anyhow::Result<Vec<crate::domain::entity::team::Team>> {
        Ok(vec![])
    }

    async fn find_by_id(
        &self,
        _id: uuid::Uuid,
    ) -> anyhow::Result<Option<crate::domain::entity::team::Team>> {
        Ok(None)
    }

    async fn create(
        &self,
        team: &crate::domain::entity::team::Team,
    ) -> anyhow::Result<crate::domain::entity::team::Team> {
        Ok(team.clone())
    }

    async fn update(
        &self,
        team: &crate::domain::entity::team::Team,
    ) -> anyhow::Result<crate::domain::entity::team::Team> {
        Ok(team.clone())
    }

    async fn delete(&self, _id: uuid::Uuid) -> anyhow::Result<bool> {
        Ok(false)
    }
}

struct StubDependencyRepository;

#[async_trait::async_trait]
impl crate::domain::repository::DependencyRepository for StubDependencyRepository {
    async fn list_by_service(
        &self,
        _service_id: uuid::Uuid,
    ) -> anyhow::Result<Vec<crate::domain::entity::dependency::Dependency>> {
        Ok(vec![])
    }

    async fn set_dependencies(
        &self,
        _service_id: uuid::Uuid,
        _deps: Vec<crate::domain::entity::dependency::Dependency>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_all_dependencies(
        &self,
    ) -> anyhow::Result<Vec<crate::domain::entity::dependency::Dependency>> {
        Ok(vec![])
    }
}

struct StubHealthRepository;

#[async_trait::async_trait]
impl crate::domain::repository::HealthRepository for StubHealthRepository {
    async fn get_latest(
        &self,
        _service_id: uuid::Uuid,
    ) -> anyhow::Result<Option<crate::domain::entity::health::HealthStatus>> {
        Ok(None)
    }

    async fn upsert(
        &self,
        _health: &crate::domain::entity::health::HealthStatus,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn list_all_latest(
        &self,
    ) -> anyhow::Result<Vec<crate::domain::entity::health::HealthStatus>> {
        Ok(vec![])
    }
}

struct StubDocRepository;

#[async_trait::async_trait]
impl crate::domain::repository::DocRepository for StubDocRepository {
    async fn list_by_service(
        &self,
        _service_id: uuid::Uuid,
    ) -> anyhow::Result<Vec<crate::domain::entity::service_doc::ServiceDoc>> {
        Ok(vec![])
    }

    async fn set_docs(
        &self,
        _service_id: uuid::Uuid,
        _docs: Vec<crate::domain::entity::service_doc::ServiceDoc>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

struct StubScorecardRepository;

#[async_trait::async_trait]
impl crate::domain::repository::ScorecardRepository for StubScorecardRepository {
    async fn get(
        &self,
        _service_id: uuid::Uuid,
    ) -> anyhow::Result<Option<crate::domain::entity::scorecard::Scorecard>> {
        Ok(None)
    }

    async fn upsert(
        &self,
        _scorecard: &crate::domain::entity::scorecard::Scorecard,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
