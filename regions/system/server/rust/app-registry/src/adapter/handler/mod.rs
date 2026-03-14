pub mod app_handler;
pub mod download_handler;
pub mod version_handler;

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
    CreateAppUseCase, CreateVersionUseCase, DeleteAppUseCase, DeleteVersionUseCase,
    GenerateDownloadUrlUseCase, GetAppUseCase, GetDownloadStatsUseCase, GetLatestUseCase,
    ListAppsUseCase, ListVersionsUseCase, UpdateAppUseCase,
};

/// ValidateTokenUseCase はトークン検証のためのユースケース。
/// auth server の ValidateTokenUseCase と同等だが、app-registry 用に簡略化。
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
    pub list_apps_uc: Arc<ListAppsUseCase>,
    pub get_app_uc: Arc<GetAppUseCase>,
    pub create_app_uc: Arc<CreateAppUseCase>,
    pub update_app_uc: Arc<UpdateAppUseCase>,
    pub delete_app_uc: Arc<DeleteAppUseCase>,
    pub list_versions_uc: Arc<ListVersionsUseCase>,
    pub create_version_uc: Arc<CreateVersionUseCase>,
    pub delete_version_uc: Arc<DeleteVersionUseCase>,
    pub get_latest_uc: Arc<GetLatestUseCase>,
    pub get_download_stats_uc: Arc<GetDownloadStatsUseCase>,
    pub generate_download_url_uc: Arc<GenerateDownloadUrlUseCase>,
    pub validate_token_uc: Arc<ValidateTokenUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub db_pool: Option<sqlx::PgPool>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        app_handler::healthz,
        app_handler::readyz,
        app_handler::metrics,
        app_handler::list_apps,
        app_handler::get_app,
        app_handler::get_download_stats,
        app_handler::create_app,
        app_handler::update_app,
        app_handler::delete_app,
        version_handler::list_versions,
        version_handler::create_version,
        version_handler::delete_version,
        download_handler::get_latest,
        download_handler::download_version,
    ),
    components(schemas(
        crate::domain::entity::app::App,
        crate::domain::entity::version::AppVersion,
        crate::domain::entity::platform::Platform,
        crate::domain::entity::download_stat::DownloadStat,
        crate::usecase::generate_download_url::DownloadUrlResult,
        crate::usecase::get_download_stats::DownloadStatsSummary,
        app_handler::AppListResponse,
        version_handler::VersionListResponse,
        version_handler::CreateVersionResponse,
        version_handler::CreateVersionRequest,
        ErrorResponse,
        ErrorBody,
    )),
    security(("bearer_auth" = [])),
)]
struct ApiDoc;

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    // Read routes: require "apps" / "read" permission
    let app_read_routes = Router::new()
        .route("/api/v1/apps", get(app_handler::list_apps))
        .route("/api/v1/apps/:id", get(app_handler::get_app))
        .route(
            "/api/v1/apps/:id/versions",
            get(version_handler::list_versions),
        )
        .route("/api/v1/apps/:id/latest", get(download_handler::get_latest))
        .route(
            "/api/v1/apps/:id/versions/:version/download",
            get(download_handler::download_version),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_rbac_middleware("apps", "read"),
        ));

    let app_admin_routes = Router::new()
        .route("/api/v1/apps/:id/stats", get(app_handler::get_download_stats))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_rbac_middleware("apps", "admin"),
        ));

    // Write routes: require "apps" / "write" permission (publisher/admin)
    let app_write_routes = Router::new()
        .route("/api/v1/apps", post(app_handler::create_app))
        .route(
            "/api/v1/apps/:id",
            put(app_handler::update_app).delete(app_handler::delete_app),
        )
        .route(
            "/api/v1/apps/:id/versions",
            post(version_handler::create_version),
        )
        .route(
            "/api/v1/apps/:id/versions/:version",
            delete(version_handler::delete_version),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_rbac_middleware("apps", "write"),
        ));

    // Protected routes share auth_middleware for Bearer token validation
    let protected = Router::new()
        .merge(app_read_routes)
        .merge(app_admin_routes)
        .merge(app_write_routes)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Public endpoints (no auth required)
    let public = Router::new()
        .route("/healthz", get(app_handler::healthz))
        .route("/readyz", get(app_handler::readyz))
        .route("/metrics", get(app_handler::metrics));

    Router::new()
        .merge(protected)
        .merge(public)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
