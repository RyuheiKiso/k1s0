#![allow(clippy::unwrap_used)]
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use tower::ServiceExt;
use uuid::Uuid;

use k1s0_service_catalog::adapter::handler::{router, AppState, ValidateTokenUseCase};
use k1s0_service_catalog::domain::entity::claims::{Claims, RealmAccess};
use k1s0_service_catalog::domain::entity::dependency::Dependency;
use k1s0_service_catalog::domain::entity::health::HealthStatus;
use k1s0_service_catalog::domain::entity::scorecard::Scorecard;
use k1s0_service_catalog::domain::entity::service::{Service, ServiceLifecycle, ServiceTier};
use k1s0_service_catalog::domain::entity::service_doc::ServiceDoc;
use k1s0_service_catalog::domain::entity::team::Team;
use k1s0_service_catalog::domain::repository::service_repository::ServiceListFilters;
use k1s0_service_catalog::domain::repository::{
    DependencyRepository, DocRepository, HealthRepository, ScorecardRepository, ServiceRepository,
    TeamRepository,
};
use k1s0_service_catalog::infrastructure::TokenVerifier;

// --- Test doubles ---

struct TestTokenVerifier {
    should_succeed: bool,
}

#[async_trait::async_trait]
impl TokenVerifier for TestTokenVerifier {
    async fn verify_token(&self, _token: &str) -> anyhow::Result<Claims> {
        if self.should_succeed {
            let now = Utc::now().timestamp();
            Ok(Claims {
                sub: "test-user-1".to_string(),
                iss: "test-issuer".to_string(),
                aud: "test-audience".to_string(),
                exp: now + 3600,
                iat: now,
                preferred_username: "test.user".to_string(),
                email: "test@example.com".to_string(),
                realm_access: RealmAccess {
                    roles: vec!["user".to_string(), "sys_operator".to_string()],
                },
                tier_access: vec!["system".to_string()],
                ..Default::default()
            })
        } else {
            anyhow::bail!("token verification failed")
        }
    }
}

struct TestServiceRepository {
    services: tokio::sync::RwLock<Vec<Service>>,
}

impl TestServiceRepository {
    fn new() -> Self {
        Self {
            services: tokio::sync::RwLock::new(Vec::new()),
        }
    }

    fn with_services(services: Vec<Service>) -> Self {
        Self {
            services: tokio::sync::RwLock::new(services),
        }
    }
}

#[async_trait::async_trait]
impl ServiceRepository for TestServiceRepository {
    async fn list(&self, _filters: ServiceListFilters) -> anyhow::Result<Vec<Service>> {
        Ok(self.services.read().await.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Service>> {
        Ok(self
            .services
            .read()
            .await
            .iter()
            .find(|s| s.id == id)
            .cloned())
    }

    async fn create(&self, service: &Service) -> anyhow::Result<Service> {
        self.services.write().await.push(service.clone());
        Ok(service.clone())
    }

    async fn update(&self, service: &Service) -> anyhow::Result<Service> {
        let mut services = self.services.write().await;
        if let Some(existing) = services.iter_mut().find(|s| s.id == service.id) {
            *existing = service.clone();
            Ok(service.clone())
        } else {
            anyhow::bail!("service not found")
        }
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        let mut services = self.services.write().await;
        services.retain(|s| s.id != id);
        Ok(())
    }

    async fn search(
        &self,
        query: Option<String>,
        _tags: Option<Vec<String>>,
        _tier: Option<ServiceTier>,
    ) -> anyhow::Result<Vec<Service>> {
        let services = self.services.read().await;
        if let Some(q) = query {
            Ok(services
                .iter()
                .filter(|s| s.name.contains(&q))
                .cloned()
                .collect())
        } else {
            Ok(services.clone())
        }
    }
}

struct TestTeamRepository {
    teams: tokio::sync::RwLock<Vec<Team>>,
}

impl TestTeamRepository {
    fn new() -> Self {
        Self {
            teams: tokio::sync::RwLock::new(Vec::new()),
        }
    }

    fn with_teams(teams: Vec<Team>) -> Self {
        Self {
            teams: tokio::sync::RwLock::new(teams),
        }
    }
}

#[async_trait::async_trait]
impl TeamRepository for TestTeamRepository {
    async fn list(&self) -> anyhow::Result<Vec<Team>> {
        Ok(self.teams.read().await.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Team>> {
        Ok(self.teams.read().await.iter().find(|t| t.id == id).cloned())
    }

    async fn create(&self, team: &Team) -> anyhow::Result<Team> {
        self.teams.write().await.push(team.clone());
        Ok(team.clone())
    }

    async fn update(&self, team: &Team) -> anyhow::Result<Team> {
        let mut teams = self.teams.write().await;
        if let Some(existing) = teams.iter_mut().find(|t| t.id == team.id) {
            *existing = team.clone();
            Ok(team.clone())
        } else {
            anyhow::bail!("team not found")
        }
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<bool> {
        let mut teams = self.teams.write().await;
        let len_before = teams.len();
        teams.retain(|t| t.id != id);
        Ok(teams.len() < len_before)
    }
}

struct TestDependencyRepository;

#[async_trait::async_trait]
impl DependencyRepository for TestDependencyRepository {
    async fn list_by_service(&self, _service_id: Uuid) -> anyhow::Result<Vec<Dependency>> {
        Ok(vec![])
    }

    async fn set_dependencies(
        &self,
        _service_id: Uuid,
        _deps: Vec<Dependency>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_all_dependencies(&self) -> anyhow::Result<Vec<Dependency>> {
        Ok(vec![])
    }
}

struct TestHealthRepository;

#[async_trait::async_trait]
impl HealthRepository for TestHealthRepository {
    async fn get_latest(&self, _service_id: Uuid) -> anyhow::Result<Option<HealthStatus>> {
        Ok(None)
    }

    async fn upsert(&self, _health: &HealthStatus) -> anyhow::Result<()> {
        Ok(())
    }

    async fn list_all_latest(&self) -> anyhow::Result<Vec<HealthStatus>> {
        Ok(vec![])
    }
}

struct TestDocRepository;

#[async_trait::async_trait]
impl DocRepository for TestDocRepository {
    async fn list_by_service(&self, _service_id: Uuid) -> anyhow::Result<Vec<ServiceDoc>> {
        Ok(vec![])
    }

    async fn set_docs(&self, _service_id: Uuid, _docs: Vec<ServiceDoc>) -> anyhow::Result<()> {
        Ok(())
    }
}

struct TestScorecardRepository;

#[async_trait::async_trait]
impl ScorecardRepository for TestScorecardRepository {
    async fn get(&self, _service_id: Uuid) -> anyhow::Result<Option<Scorecard>> {
        Ok(None)
    }

    async fn upsert(&self, _scorecard: &Scorecard) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- Factory functions ---

fn make_test_app_with_repos(
    token_success: bool,
    service_repo: Arc<dyn ServiceRepository>,
    team_repo: Arc<dyn TeamRepository>,
) -> axum::Router {
    let dep_repo: Arc<dyn DependencyRepository> = Arc::new(TestDependencyRepository);
    let health_repo: Arc<dyn HealthRepository> = Arc::new(TestHealthRepository);
    let doc_repo: Arc<dyn DocRepository> = Arc::new(TestDocRepository);
    let scorecard_repo: Arc<dyn ScorecardRepository> = Arc::new(TestScorecardRepository);

    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("test"));

    let state = AppState {
        list_services_uc: Arc::new(k1s0_service_catalog::usecase::ListServicesUseCase::new(
            service_repo.clone(),
        )),
        get_service_uc: Arc::new(k1s0_service_catalog::usecase::GetServiceUseCase::new(
            service_repo.clone(),
        )),
        register_service_uc: Arc::new(k1s0_service_catalog::usecase::RegisterServiceUseCase::new(
            service_repo.clone(),
            team_repo.clone(),
        )),
        update_service_uc: Arc::new(k1s0_service_catalog::usecase::UpdateServiceUseCase::new(
            service_repo.clone(),
        )),
        delete_service_uc: Arc::new(k1s0_service_catalog::usecase::DeleteServiceUseCase::new(
            service_repo.clone(),
        )),
        manage_deps_uc: Arc::new(
            k1s0_service_catalog::usecase::ManageDependenciesUseCase::new(dep_repo),
        ),
        health_status_uc: Arc::new(k1s0_service_catalog::usecase::HealthStatusUseCase::new(
            health_repo,
        )),
        manage_docs_uc: Arc::new(k1s0_service_catalog::usecase::ManageDocsUseCase::new(
            doc_repo,
        )),
        get_scorecard_uc: Arc::new(k1s0_service_catalog::usecase::GetScorecardUseCase::new(
            scorecard_repo,
        )),
        search_services_uc: Arc::new(k1s0_service_catalog::usecase::SearchServicesUseCase::new(
            service_repo.clone(),
        )),
        list_teams_uc: Arc::new(k1s0_service_catalog::usecase::ListTeamsUseCase::new(
            team_repo.clone(),
        )),
        get_team_uc: Arc::new(k1s0_service_catalog::usecase::GetTeamUseCase::new(
            team_repo.clone(),
        )),
        create_team_uc: Arc::new(k1s0_service_catalog::usecase::CreateTeamUseCase::new(
            team_repo.clone(),
        )),
        update_team_uc: Arc::new(k1s0_service_catalog::usecase::UpdateTeamUseCase::new(
            team_repo.clone(),
        )),
        delete_team_uc: Arc::new(k1s0_service_catalog::usecase::DeleteTeamUseCase::new(
            team_repo.clone(),
        )),
        validate_token_uc: Arc::new(ValidateTokenUseCase::new(
            Arc::new(TestTokenVerifier {
                should_succeed: token_success,
            }),
            "test-issuer".to_string(),
            "test-audience".to_string(),
        )),
        metrics: metrics.clone(),
        db_pool: None,
    };
    router(state)
}

fn make_test_app(token_success: bool) -> axum::Router {
    make_test_app_with_repos(
        token_success,
        Arc::new(TestServiceRepository::new()),
        Arc::new(TestTeamRepository::new()),
    )
}

fn make_test_service(name: &str, team_id: Uuid) -> Service {
    let now = Utc::now();
    Service {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: Some(format!("Description for {}", name)),
        team_id,
        tier: ServiceTier::Standard,
        lifecycle: ServiceLifecycle::Production,
        repository_url: None,
        api_endpoint: None,
        healthcheck_url: None,
        tags: vec!["test".to_string()],
        metadata: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    }
}

fn make_test_team(name: &str) -> Team {
    let now = Utc::now();
    Team {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: Some(format!("Team {}", name)),
        contact_email: Some(format!("{}@example.com", name)),
        slack_channel: Some(format!("#{}", name)),
        created_at: now,
        updated_at: now,
    }
}

// --- Integration Tests ---

#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app(true);

    // healthz
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // readyz
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_services() {
    let team_id = Uuid::new_v4();
    let svc = make_test_service("test-service-alpha", team_id);
    let svc_name = svc.name.clone();

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::with_services(vec![svc])),
        Arc::new(TestTeamRepository::new()),
    );

    let req = Request::builder()
        .uri("/api/v1/services")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let services = json.as_array().unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0]["name"], svc_name);
}

#[tokio::test]
async fn test_get_service_found() {
    let team_id = Uuid::new_v4();
    let svc = make_test_service("my-found-service", team_id);
    let svc_id = svc.id;
    let svc_name = svc.name.clone();

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::with_services(vec![svc])),
        Arc::new(TestTeamRepository::new()),
    );

    // GET /api/v1/services/:id で直接サービスを取得する
    let req = Request::builder()
        .uri(format!("/api/v1/services/{}", svc_id))
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], svc_name);
}

#[tokio::test]
async fn test_get_service_not_found() {
    let app = make_test_app(true);
    let random_id = Uuid::new_v4();

    let req = Request::builder()
        .uri(format!("/api/v1/services/{}", random_id))
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_register_service() {
    let team = make_test_team("platform-team");
    let team_id = team.id;

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::new()),
        Arc::new(TestTeamRepository::with_teams(vec![team])),
    );

    let input = serde_json::json!({
        "name": "new-service",
        "description": "A brand new service",
        "team_id": team_id,
        "tier": "standard",
        "lifecycle": "development",
        "tags": ["new"]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/services")
        .header("content-type", "application/json")
        .header("Authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&input).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "new-service");
}

#[tokio::test]
async fn test_list_teams() {
    let team_a = make_test_team("team-alpha");
    let team_b = make_test_team("team-beta");

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::new()),
        Arc::new(TestTeamRepository::with_teams(vec![team_a, team_b])),
    );

    let req = Request::builder()
        .uri("/api/v1/teams")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let teams = json.as_array().unwrap();
    assert_eq!(teams.len(), 2);
}

#[tokio::test]
async fn test_search_services() {
    let team_id = Uuid::new_v4();
    let svc1 = make_test_service("test-alpha", team_id);
    let svc2 = make_test_service("other-beta", team_id);

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::with_services(vec![svc1, svc2])),
        Arc::new(TestTeamRepository::new()),
    );

    let req = Request::builder()
        .uri("/api/v1/services/search?q=test")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let results = json.as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "test-alpha");
}

#[tokio::test]
async fn test_unauthorized_without_token() {
    let app = make_test_app(true);

    let req = Request::builder()
        .uri("/api/v1/services")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unauthorized_with_invalid_token() {
    let app = make_test_app(false);

    let req = Request::builder()
        .uri("/api/v1/services")
        .header("Authorization", "Bearer invalid-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// サービス登録の詳細テスト
// ---------------------------------------------------------------------------

/// 全フィールドを指定してサービスを登録する。
#[tokio::test]
async fn test_register_service_with_all_fields() {
    let team = make_test_team("full-team");
    let team_id = team.id;

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::new()),
        Arc::new(TestTeamRepository::with_teams(vec![team])),
    );

    let input = serde_json::json!({
        "name": "full-service",
        "description": "A fully specified service",
        "team_id": team_id,
        "tier": "critical",
        "lifecycle": "production",
        "repository_url": "https://github.com/example/repo",
        "api_endpoint": "https://api.example.com",
        "healthcheck_url": "https://api.example.com/healthz",
        "tags": ["critical", "payments"]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/services")
        .header("content-type", "application/json")
        .header("Authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&input).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "full-service");
    assert_eq!(json["tier"], "critical");
    assert_eq!(json["lifecycle"], "production");
}

/// 存在しないチームIDでサービス登録を試みるとエラーになる。
#[tokio::test]
async fn test_register_service_invalid_team_id() {
    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::new()),
        Arc::new(TestTeamRepository::new()),
    );

    let fake_team_id = Uuid::new_v4();
    let input = serde_json::json!({
        "name": "orphan-service",
        "team_id": fake_team_id,
        "tier": "standard",
        "lifecycle": "development"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/services")
        .header("content-type", "application/json")
        .header("Authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&input).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    // チームが存在しないためエラーになる
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

// ---------------------------------------------------------------------------
// サービス削除テスト
// ---------------------------------------------------------------------------

/// サービスを DELETE で削除できることを検証する。
#[tokio::test]
async fn test_delete_service() {
    let team_id = Uuid::new_v4();
    let svc = make_test_service("delete-me-service", team_id);
    let svc_id = svc.id;

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::with_services(vec![svc])),
        Arc::new(TestTeamRepository::new()),
    );

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/services/{}", svc_id))
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    // delete_service ハンドラーは NO_CONTENT (204) を返す
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ---------------------------------------------------------------------------
// チーム CRUD テスト
// ---------------------------------------------------------------------------

/// チームを POST で作成する。
#[tokio::test]
async fn test_create_team() {
    let app = make_test_app(true);

    let input = serde_json::json!({
        "name": "new-platform-team",
        "description": "Platform engineering team",
        "contact_email": "platform@example.com",
        "slack_channel": "#platform"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/teams")
        .header("content-type", "application/json")
        .header("Authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&input).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "new-platform-team");
}

/// 存在しないチームの取得は 404 になる。
#[tokio::test]
async fn test_get_team_not_found() {
    let app = make_test_app(true);
    let random_id = Uuid::new_v4();

    let req = Request::builder()
        .uri(format!("/api/v1/teams/{}", random_id))
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// チームを DELETE で削除する。
#[tokio::test]
async fn test_delete_team() {
    let team = make_test_team("delete-me-team");
    let team_id = team.id;

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::new()),
        Arc::new(TestTeamRepository::with_teams(vec![team])),
    );

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/teams/{}", team_id))
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    // delete_team ハンドラーは NO_CONTENT (204) を返す
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

/// 空のチームリストは空配列を返す。
#[tokio::test]
async fn test_list_teams_empty() {
    let app = make_test_app(true);

    let req = Request::builder()
        .uri("/api/v1/teams")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let teams = json.as_array().unwrap();
    assert!(teams.is_empty());
}

// ---------------------------------------------------------------------------
// 検索クエリテスト
// ---------------------------------------------------------------------------

/// 検索クエリに一致しない場合は空の結果を返す。
#[tokio::test]
async fn test_search_services_no_match() {
    let team_id = Uuid::new_v4();
    let svc = make_test_service("alpha-service", team_id);

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::with_services(vec![svc])),
        Arc::new(TestTeamRepository::new()),
    );

    let req = Request::builder()
        .uri("/api/v1/services/search?q=zzz-nonexistent")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let results = json.as_array().unwrap();
    assert!(results.is_empty());
}

/// 検索クエリなしで全サービスが返される。
#[tokio::test]
async fn test_search_services_no_query_returns_all() {
    let team_id = Uuid::new_v4();
    let svc1 = make_test_service("svc-one", team_id);
    let svc2 = make_test_service("svc-two", team_id);

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::with_services(vec![svc1, svc2])),
        Arc::new(TestTeamRepository::new()),
    );

    let req = Request::builder()
        .uri("/api/v1/services/search")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let results = json.as_array().unwrap();
    assert_eq!(results.len(), 2);
}

// ---------------------------------------------------------------------------
// サービスリスト（複数）テスト
// ---------------------------------------------------------------------------

/// 複数サービスが正しくリストされる。
#[tokio::test]
async fn test_list_multiple_services() {
    let team_id = Uuid::new_v4();
    let services: Vec<Service> = (0..5)
        .map(|i| make_test_service(&format!("svc-{}", i), team_id))
        .collect();

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::with_services(services)),
        Arc::new(TestTeamRepository::new()),
    );

    let req = Request::builder()
        .uri("/api/v1/services")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let svc_list = json.as_array().unwrap();
    assert_eq!(svc_list.len(), 5);
}

/// 空のサービスリストは空配列を返す。
#[tokio::test]
async fn test_list_services_empty() {
    let app = make_test_app(true);

    let req = Request::builder()
        .uri("/api/v1/services")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let svc_list = json.as_array().unwrap();
    assert!(svc_list.is_empty());
}

// ---------------------------------------------------------------------------
// ヘルスチェック詳細テスト
// ---------------------------------------------------------------------------

/// healthz と readyz が認証なしで使える。
#[tokio::test]
async fn test_health_endpoints_no_auth() {
    let app = make_test_app(false);

    // healthz は認証不要
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

/// 不正な Authorization 形式は 401 になる。
#[tokio::test]
async fn test_malformed_auth_header() {
    let app = make_test_app(true);

    let req = Request::builder()
        .uri("/api/v1/services")
        .header("Authorization", "Basic dXNlcjpwYXNz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// チーム更新テスト
// ---------------------------------------------------------------------------

/// チームを PUT で更新する。
#[tokio::test]
async fn test_update_team() {
    let team = make_test_team("old-team-name");
    let team_id = team.id;

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::new()),
        Arc::new(TestTeamRepository::with_teams(vec![team])),
    );

    let input = serde_json::json!({
        "name": "updated-team-name",
        "description": "Updated description",
        "contact_email": "updated@example.com"
    });

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/teams/{}", team_id))
        .header("content-type", "application/json")
        .header("Authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&input).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert!(resp.status().is_success());
}

/// 存在しないチームの更新は 404 になる。
#[tokio::test]
async fn test_update_team_not_found() {
    let app = make_test_app(true);

    let random_id = Uuid::new_v4();
    let input = serde_json::json!({
        "name": "phantom-team",
        "description": "Does not exist"
    });

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/teams/{}", random_id))
        .header("content-type", "application/json")
        .header("Authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&input).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// 特定のチームを ID で取得する。
#[tokio::test]
async fn test_get_team_found() {
    let team = make_test_team("findable-team");
    let team_id = team.id;

    let app = make_test_app_with_repos(
        true,
        Arc::new(TestServiceRepository::new()),
        Arc::new(TestTeamRepository::with_teams(vec![team])),
    );

    let req = Request::builder()
        .uri(format!("/api/v1/teams/{}", team_id))
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "findable-team");
}
