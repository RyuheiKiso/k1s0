// Policy サーバーの handler テスト
// axum-test を使って REST API エンドポイントの動作を確認する
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use async_trait::async_trait;
use axum_test::TestServer;
use tokio::sync::RwLock;
use uuid::Uuid;

use k1s0_policy_server::adapter::handler::{router, AppState};
use k1s0_policy_server::domain::entity::policy::Policy;
use k1s0_policy_server::domain::entity::policy_bundle::PolicyBundle;
use k1s0_policy_server::domain::repository::{PolicyBundleRepository, PolicyRepository};
use k1s0_policy_server::usecase::{
    CreateBundleUseCase, CreatePolicyUseCase, DeletePolicyUseCase, EvaluatePolicyUseCase,
    GetBundleUseCase, GetPolicyUseCase, ListBundlesUseCase, ListPoliciesUseCase,
    UpdatePolicyUseCase,
};

// ---------------------------------------------------------------------------
// Stub: In-memory PolicyRepository
// ---------------------------------------------------------------------------

struct StubPolicyRepo {
    policies: RwLock<Vec<Policy>>,
}

impl StubPolicyRepo {
    fn new() -> Self {
        Self { policies: RwLock::new(Vec::new()) }
    }
}

#[async_trait]
impl PolicyRepository for StubPolicyRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Policy>> {
        let policies = self.policies.read().await;
        Ok(policies.iter().find(|p| p.id == *id).cloned())
    }
    async fn find_all(&self) -> anyhow::Result<Vec<Policy>> {
        Ok(self.policies.read().await.clone())
    }
    async fn find_all_paginated(
        &self,
        _page: u32,
        _page_size: u32,
        _bundle_id: Option<Uuid>,
        _enabled_only: bool,
    ) -> anyhow::Result<(Vec<Policy>, u64)> {
        let p = self.policies.read().await.clone();
        let n = p.len() as u64;
        Ok((p, n))
    }
    async fn create(&self, policy: &Policy) -> anyhow::Result<()> {
        self.policies.write().await.push(policy.clone());
        Ok(())
    }
    async fn update(&self, policy: &Policy) -> anyhow::Result<()> {
        let mut policies = self.policies.write().await;
        if let Some(e) = policies.iter_mut().find(|p| p.id == policy.id) {
            *e = policy.clone();
        }
        Ok(())
    }
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut policies = self.policies.write().await;
        let before = policies.len();
        policies.retain(|p| p.id != *id);
        Ok(policies.len() < before)
    }
    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        Ok(self.policies.read().await.iter().any(|p| p.name == name))
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory PolicyBundleRepository
// ---------------------------------------------------------------------------

struct StubBundleRepo {
    bundles: RwLock<Vec<PolicyBundle>>,
}

impl StubBundleRepo {
    fn new() -> Self {
        Self { bundles: RwLock::new(Vec::new()) }
    }
}

#[async_trait]
impl PolicyBundleRepository for StubBundleRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<PolicyBundle>> {
        Ok(self.bundles.read().await.iter().find(|b| b.id == *id).cloned())
    }
    async fn find_all(&self) -> anyhow::Result<Vec<PolicyBundle>> {
        Ok(self.bundles.read().await.clone())
    }
    async fn create(&self, bundle: &PolicyBundle) -> anyhow::Result<()> {
        self.bundles.write().await.push(bundle.clone());
        Ok(())
    }
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut bundles = self.bundles.write().await;
        let before = bundles.len();
        bundles.retain(|b| b.id != *id);
        Ok(bundles.len() < before)
    }
}

// ---------------------------------------------------------------------------
// Helper: AppState を構築する
// ---------------------------------------------------------------------------

fn build_state() -> AppState {
    let policy_repo = Arc::new(StubPolicyRepo::new());
    let bundle_repo = Arc::new(StubBundleRepo::new());
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("policy-test"));
    AppState {
        create_policy_uc: Arc::new(CreatePolicyUseCase::new(policy_repo.clone())),
        get_policy_uc: Arc::new(GetPolicyUseCase::new(policy_repo.clone())),
        list_policies_uc: Arc::new(ListPoliciesUseCase::new(policy_repo.clone())),
        update_policy_uc: Arc::new(UpdatePolicyUseCase::new(policy_repo.clone())),
        delete_policy_uc: Arc::new(DeletePolicyUseCase::new(policy_repo.clone())),
        evaluate_policy_uc: Arc::new(EvaluatePolicyUseCase::new(policy_repo.clone(), None)),
        create_bundle_uc: Arc::new(CreateBundleUseCase::new(bundle_repo.clone())),
        get_bundle_uc: Arc::new(GetBundleUseCase::new(bundle_repo.clone())),
        list_bundles_uc: Arc::new(ListBundlesUseCase::new(bundle_repo.clone())),
        metrics,
        auth_state: None,
        backend_kind: "in-memory".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// GET /healthz は 200 {"status":"ok"} を返す
#[tokio::test]
async fn healthz_returns_ok() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/healthz").await;
    resp.assert_status_ok();
    assert_eq!(resp.json::<serde_json::Value>()["status"], "ok");
}

/// GET /readyz は in-memory バックエンドで "degraded" を返す
#[tokio::test]
async fn readyz_in_memory_returns_degraded() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/readyz").await;
    resp.assert_status_ok();
    let body = resp.json::<serde_json::Value>();
    assert_eq!(body["status"], "degraded");
    assert_eq!(body["backend"], "in-memory");
}

/// GET /api/v1/policies は空リストを返す（認証なしモード）
#[tokio::test]
async fn list_policies_returns_empty() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/api/v1/policies").await;
    resp.assert_status_ok();
    let body = resp.json::<serde_json::Value>();
    assert_eq!(body["policies"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total_count"], 0);
}

/// GET /api/v1/policies/{id} は存在しない ID に 404 を返す
#[tokio::test]
async fn get_policy_not_found() {
    let server = TestServer::new(router(build_state())).unwrap();
    let id = Uuid::new_v4();
    let resp = server.get(&format!("/api/v1/policies/{}", id)).await;
    resp.assert_status_not_found();
}

/// POST /api/v1/policies で名前が空の場合は 400 を返す
#[tokio::test]
async fn create_policy_empty_name_returns_bad_request() {
    let server = TestServer::new(router(build_state())).unwrap();
    let body = serde_json::json!({
        "name": "   ",
        "description": "test",
        "rego_content": "package authz",
        "package_path": "k1s0.test"
    });
    let resp = server.post("/api/v1/policies").json(&body).await;
    resp.assert_status_bad_request();
}

/// POST /api/v1/policies で正常にポリシーを作成できる
#[tokio::test]
async fn create_policy_success() {
    let server = TestServer::new(router(build_state())).unwrap();
    let body = serde_json::json!({
        "name": "test-policy",
        "description": "Test policy",
        "rego_content": "package authz\ndefault allow = true",
        "package_path": "k1s0.test.authz"
    });
    let resp = server.post("/api/v1/policies").json(&body).await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let created = resp.json::<serde_json::Value>();
    assert_eq!(created["name"], "test-policy");
}

/// GET /api/v1/bundles は空リストを返す
#[tokio::test]
async fn list_bundles_returns_empty() {
    let server = TestServer::new(router(build_state())).unwrap();
    let resp = server.get("/api/v1/bundles").await;
    resp.assert_status_ok();
    let body = resp.json::<serde_json::Value>();
    assert!(body["bundles"].as_array().unwrap().is_empty());
}
