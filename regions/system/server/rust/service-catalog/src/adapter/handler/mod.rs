pub mod dependency;
pub mod doc;
pub mod health;
pub mod scorecard;
pub mod search;
pub mod service;
pub mod team;

use std::sync::Arc;

use axum::middleware;
use axum::routing::{delete, get, post, put};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub use k1s0_server_common::{ErrorBody, ErrorResponse};

use crate::adapter::middleware::auth::auth_middleware;
use crate::adapter::middleware::rbac::make_rbac_middleware;
use crate::usecase::{
    DeleteServiceUseCase, GetScorecardUseCase, GetServiceUseCase, HealthStatusUseCase,
    ListServicesUseCase, ManageDependenciesUseCase, ManageDocsUseCase, RegisterServiceUseCase,
    SearchServicesUseCase, UpdateServiceUseCase,
};

/// ValidateTokenUseCase はトークン検証のためのユースケース。
pub struct ValidateTokenUseCase {
    verifier: Arc<dyn crate::infrastructure::TokenVerifier>,
    expected_issuer: String,
    expected_audience: String,
}

impl ValidateTokenUseCase {
    pub fn new(
        verifier: Arc<dyn crate::infrastructure::TokenVerifier>,
        expected_issuer: String,
        expected_audience: String,
    ) -> Self {
        Self {
            verifier,
            expected_issuer,
            expected_audience,
        }
    }

    pub async fn execute(
        &self,
        token: &str,
    ) -> anyhow::Result<crate::domain::entity::claims::Claims> {
        let claims = self.verifier.verify_token(token).await?;

        if claims.iss != self.expected_issuer {
            anyhow::bail!("invalid issuer");
        }
        if claims.aud != self.expected_audience {
            anyhow::bail!("invalid audience");
        }
        let now = chrono::Utc::now().timestamp();
        if claims.exp < now {
            anyhow::bail!("token expired");
        }

        Ok(claims)
    }
}

/// AppState はアプリケーション全体の共有状態を表す。
#[derive(Clone)]
pub struct AppState {
    pub list_services_uc: Arc<ListServicesUseCase>,
    pub get_service_uc: Arc<GetServiceUseCase>,
    pub register_service_uc: Arc<RegisterServiceUseCase>,
    pub update_service_uc: Arc<UpdateServiceUseCase>,
    pub delete_service_uc: Arc<DeleteServiceUseCase>,
    pub manage_deps_uc: Arc<ManageDependenciesUseCase>,
    pub health_status_uc: Arc<HealthStatusUseCase>,
    pub manage_docs_uc: Arc<ManageDocsUseCase>,
    pub get_scorecard_uc: Arc<GetScorecardUseCase>,
    pub search_services_uc: Arc<SearchServicesUseCase>,
    pub list_teams_uc: Arc<crate::usecase::list_teams::ListTeamsUseCase>,
    pub validate_token_uc: Arc<ValidateTokenUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub db_pool: Option<sqlx::PgPool>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        service::healthz,
        service::readyz,
        service::metrics_endpoint,
        service::list_services,
        service::get_service,
        service::register_service,
        service::update_service,
        service::delete_service,
        team::list_teams,
        team::get_team_services,
        dependency::list_dependencies,
        dependency::set_dependencies,
        health::get_health,
        health::report_health,
        doc::list_docs,
        doc::set_docs,
        scorecard::get_scorecard,
        search::search_services,
    ),
    components(schemas(
        crate::domain::entity::service::Service,
        crate::domain::entity::service::ServiceTier,
        crate::domain::entity::service::ServiceLifecycle,
        crate::domain::entity::team::Team,
        crate::domain::entity::dependency::Dependency,
        crate::domain::entity::dependency::DependencyType,
        crate::domain::entity::health::HealthStatus,
        crate::domain::entity::health::HealthState,
        crate::domain::entity::scorecard::Scorecard,
        crate::domain::entity::service_doc::ServiceDoc,
        crate::domain::entity::service_doc::DocType,
        crate::usecase::register_service::RegisterServiceInput,
        crate::usecase::update_service::UpdateServiceInput,
        ErrorResponse,
        ErrorBody,
    )),
    security(("bearer_auth" = [])),
)]
struct ApiDoc;

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    // Read routes: require "services" / "read" permission
    let service_read_routes = Router::new()
        .route("/api/v1/services", get(service::list_services))
        .route("/api/v1/services/{id}", get(service::get_service))
        .route(
            "/api/v1/services/{id}/dependencies",
            get(dependency::list_dependencies),
        )
        .route("/api/v1/services/{id}/health", get(health::get_health))
        .route("/api/v1/services/{id}/docs", get(doc::list_docs))
        .route(
            "/api/v1/services/{id}/scorecard",
            get(scorecard::get_scorecard),
        )
        .route("/api/v1/services/search", get(search::search_services))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_rbac_middleware("services", "read"),
        ));

    // Team read routes
    let team_read_routes = Router::new()
        .route("/api/v1/teams", get(team::list_teams))
        .route(
            "/api/v1/teams/{team_id}/services",
            get(team::get_team_services),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_rbac_middleware("teams", "read"),
        ));

    // Write routes: require "services" / "write" permission
    let service_write_routes = Router::new()
        .route("/api/v1/services", post(service::register_service))
        .route("/api/v1/services/{id}", put(service::update_service))
        .route("/api/v1/services/{id}", delete(service::delete_service))
        .route(
            "/api/v1/services/{id}/dependencies",
            put(dependency::set_dependencies),
        )
        .route(
            "/api/v1/services/{id}/health",
            post(health::report_health),
        )
        .route("/api/v1/services/{id}/docs", put(doc::set_docs))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_rbac_middleware("services", "write"),
        ));

    // Protected routes share auth_middleware for Bearer token validation
    let protected = Router::new()
        .merge(service_read_routes)
        .merge(team_read_routes)
        .merge(service_write_routes)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Public endpoints (no auth required)
    let public = Router::new()
        .route("/healthz", get(service::healthz))
        .route("/readyz", get(service::readyz))
        .route("/metrics", get(service::metrics_endpoint));

    Router::new()
        .merge(protected)
        .merge(public)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
