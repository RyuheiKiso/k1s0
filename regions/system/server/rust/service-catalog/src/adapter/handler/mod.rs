pub mod dependency;
pub mod doc;
pub mod health;
pub mod scorecard;
pub mod search;
pub mod service;
pub mod team;

use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::get;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub use k1s0_server_common::{ErrorBody, ErrorResponse};

use crate::adapter::middleware::auth::auth_middleware;
use crate::adapter::middleware::rbac::make_method_rbac_middleware;
use crate::usecase::{
    CreateTeamUseCase, DeleteServiceUseCase, DeleteTeamUseCase, GetScorecardUseCase,
    GetServiceUseCase, GetTeamUseCase, HealthStatusUseCase, ListServicesUseCase,
    ManageDependenciesUseCase, ManageDocsUseCase, RegisterServiceUseCase, SearchServicesUseCase,
    UpdateServiceUseCase, UpdateTeamUseCase,
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
    pub get_team_uc: Arc<GetTeamUseCase>,
    pub create_team_uc: Arc<CreateTeamUseCase>,
    pub update_team_uc: Arc<UpdateTeamUseCase>,
    pub delete_team_uc: Arc<DeleteTeamUseCase>,
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

/// REST API ルーターを構築する。
/// 同一パスの GET/PUT/DELETE を1つの .route() に統合し、
/// HTTP メソッドに基づいて read/write 権限を自動判定する。
pub fn router(state: AppState) -> Router {
    // サービスルート: GET→read, POST/PUT/DELETE→write をメソッドベースで判定
    let service_routes = Router::new()
        .route(
            "/api/v1/services",
            get(service::list_services).post(service::register_service),
        )
        .route(
            "/api/v1/services/:id",
            get(service::get_service)
                .put(service::update_service)
                .delete(service::delete_service),
        )
        .route(
            "/api/v1/services/:id/dependencies",
            get(dependency::list_dependencies).put(dependency::set_dependencies),
        )
        .route(
            "/api/v1/services/:id/health",
            get(health::get_health).post(health::report_health),
        )
        .route(
            "/api/v1/services/:id/docs",
            get(doc::list_docs).put(doc::set_docs),
        )
        .route(
            "/api/v1/services/:id/scorecard",
            get(scorecard::get_scorecard),
        )
        .route("/api/v1/services/search", get(search::search_services))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_method_rbac_middleware("services"),
        ));

    // チームルート: GET→read, POST/PUT/DELETE→write をメソッドベースで判定
    let team_routes = Router::new()
        .route(
            "/api/v1/teams",
            get(team::list_teams).post(team::create_team),
        )
        .route(
            "/api/v1/teams/:team_id",
            get(team::get_team)
                .put(team::update_team)
                .delete(team::delete_team),
        )
        .route(
            "/api/v1/teams/:team_id/services",
            get(team::get_team_services),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_method_rbac_middleware("teams"),
        ));

    // 認証済みルートは auth_middleware で Bearer トークン検証を共有
    let protected = Router::new()
        .merge(service_routes)
        .merge(team_routes)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // 公開エンドポイント（認証不要）
    let public = Router::new()
        .route("/healthz", get(service::healthz))
        .route("/readyz", get(service::readyz))
        .route("/metrics", get(service::metrics_endpoint));

    // with_state で Router<()> に変換後、SwaggerUI を merge する
    Router::new()
        .merge(protected)
        .merge(public)
        .with_state(state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
