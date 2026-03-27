// handler テスト: axum ルーターの HTTP エンドポイントを oneshot で検証する
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tower::ServiceExt;
use uuid::Uuid;

use k1s0_rule_engine_server::adapter::handler::{router, AppState};
use k1s0_rule_engine_server::domain::entity::rule::{EvaluationLog, Rule, RuleSet, RuleSetVersion};
use k1s0_rule_engine_server::domain::repository::{
    EvaluationLogRepository, RuleRepository, RuleSetRepository, RuleSetVersionRepository,
};
use k1s0_rule_engine_server::usecase::{
    CreateRuleSetUseCase, CreateRuleUseCase, DeleteRuleSetUseCase, DeleteRuleUseCase,
    EvaluateUseCase, GetRuleSetUseCase, GetRuleUseCase, ListEvaluationLogsUseCase,
    ListRuleSetsUseCase, ListRulesUseCase, PublishRuleSetUseCase, RollbackRuleSetUseCase,
    UpdateRuleSetUseCase, UpdateRuleUseCase,
};

// ---------------------------------------------------------------------------
// スタブ: RuleRepository
// ---------------------------------------------------------------------------

struct StubRuleRepo {
    rules: RwLock<Vec<Rule>>,
}

impl StubRuleRepo {
    fn new() -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
        }
    }

    fn with_rules(rules: Vec<Rule>) -> Self {
        Self {
            rules: RwLock::new(rules),
        }
    }
}

#[async_trait]
impl RuleRepository for StubRuleRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Rule>> {
        Ok(self
            .rules
            .read()
            .await
            .iter()
            .find(|r| &r.id == id)
            .cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Rule>> {
        Ok(self.rules.read().await.clone())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        _rule_set_id: Option<Uuid>,
        _domain: Option<String>,
    ) -> anyhow::Result<(Vec<Rule>, u64)> {
        let rules = self.rules.read().await;
        let total = rules.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(rules.len());
        let items = if start < rules.len() {
            rules[start..end].to_vec()
        } else {
            Vec::new()
        };
        Ok((items, total))
    }

    async fn create(&self, rule: &Rule) -> anyhow::Result<()> {
        self.rules.write().await.push(rule.clone());
        Ok(())
    }

    async fn update(&self, rule: &Rule) -> anyhow::Result<()> {
        let mut rules = self.rules.write().await;
        if let Some(existing) = rules.iter_mut().find(|r| r.id == rule.id) {
            *existing = rule.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut rules = self.rules.write().await;
        let before = rules.len();
        rules.retain(|r| &r.id != id);
        Ok(rules.len() < before)
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        Ok(self.rules.read().await.iter().any(|r| r.name == name))
    }

    async fn find_by_ids(&self, ids: &[Uuid]) -> anyhow::Result<Vec<Rule>> {
        let rules = self.rules.read().await;
        Ok(rules
            .iter()
            .filter(|r| ids.contains(&r.id))
            .cloned()
            .collect())
    }
}

// ---------------------------------------------------------------------------
// スタブ: RuleSetRepository
// ---------------------------------------------------------------------------

struct StubRuleSetRepo {
    rule_sets: RwLock<Vec<RuleSet>>,
}

impl StubRuleSetRepo {
    fn new() -> Self {
        Self {
            rule_sets: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl RuleSetRepository for StubRuleSetRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<RuleSet>> {
        Ok(self
            .rule_sets
            .read()
            .await
            .iter()
            .find(|rs| &rs.id == id)
            .cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<RuleSet>> {
        Ok(self.rule_sets.read().await.clone())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        _domain: Option<String>,
    ) -> anyhow::Result<(Vec<RuleSet>, u64)> {
        let sets = self.rule_sets.read().await;
        let total = sets.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(sets.len());
        let items = if start < sets.len() {
            sets[start..end].to_vec()
        } else {
            Vec::new()
        };
        Ok((items, total))
    }

    async fn find_by_domain_and_name(
        &self,
        domain: &str,
        name: &str,
    ) -> anyhow::Result<Option<RuleSet>> {
        Ok(self
            .rule_sets
            .read()
            .await
            .iter()
            .find(|rs| rs.domain == domain && rs.name == name)
            .cloned())
    }

    async fn create(&self, rule_set: &RuleSet) -> anyhow::Result<()> {
        self.rule_sets.write().await.push(rule_set.clone());
        Ok(())
    }

    async fn update(&self, rule_set: &RuleSet) -> anyhow::Result<()> {
        let mut sets = self.rule_sets.write().await;
        if let Some(existing) = sets.iter_mut().find(|rs| rs.id == rule_set.id) {
            *existing = rule_set.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut sets = self.rule_sets.write().await;
        let before = sets.len();
        sets.retain(|rs| &rs.id != id);
        Ok(sets.len() < before)
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        Ok(self.rule_sets.read().await.iter().any(|rs| rs.name == name))
    }
}

// ---------------------------------------------------------------------------
// スタブ: RuleSetVersionRepository
// ---------------------------------------------------------------------------

struct StubVersionRepo;

#[async_trait]
impl RuleSetVersionRepository for StubVersionRepo {
    async fn find_by_rule_set_id_and_version(
        &self,
        _rule_set_id: &Uuid,
        _version: u32,
    ) -> anyhow::Result<Option<RuleSetVersion>> {
        Ok(None)
    }

    async fn find_latest_by_rule_set_id(
        &self,
        _rule_set_id: &Uuid,
    ) -> anyhow::Result<Option<RuleSetVersion>> {
        Ok(None)
    }

    async fn create(&self, _version: &RuleSetVersion) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// スタブ: EvaluationLogRepository
// ---------------------------------------------------------------------------

struct StubEvalLogRepo;

#[async_trait]
impl EvaluationLogRepository for StubEvalLogRepo {
    async fn create(&self, _log: &EvaluationLog) -> anyhow::Result<()> {
        Ok(())
    }

    async fn find_all_paginated(
        &self,
        _page: u32,
        _page_size: u32,
        _rule_set_name: Option<String>,
        _domain: Option<String>,
        _from: Option<DateTime<Utc>>,
        _to: Option<DateTime<Utc>>,
    ) -> anyhow::Result<(Vec<EvaluationLog>, u64)> {
        Ok((Vec::new(), 0))
    }
}

// ---------------------------------------------------------------------------
// AppState ビルダーヘルパー
// ---------------------------------------------------------------------------

fn build_state(rule_repo: Arc<StubRuleRepo>, rule_set_repo: Arc<StubRuleSetRepo>) -> AppState {
    let version_repo = Arc::new(StubVersionRepo);
    let eval_log_repo: Arc<dyn EvaluationLogRepository> = Arc::new(StubEvalLogRepo);
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("rule-engine-test"));

    AppState {
        create_rule_uc: Arc::new(CreateRuleUseCase::new(rule_repo.clone())),
        get_rule_uc: Arc::new(GetRuleUseCase::new(rule_repo.clone())),
        list_rules_uc: Arc::new(ListRulesUseCase::new(rule_repo.clone())),
        update_rule_uc: Arc::new(UpdateRuleUseCase::new(rule_repo.clone())),
        delete_rule_uc: Arc::new(DeleteRuleUseCase::new(rule_repo.clone())),
        create_rule_set_uc: Arc::new(CreateRuleSetUseCase::new(rule_set_repo.clone())),
        get_rule_set_uc: Arc::new(GetRuleSetUseCase::new(rule_set_repo.clone())),
        list_rule_sets_uc: Arc::new(ListRuleSetsUseCase::new(rule_set_repo.clone())),
        update_rule_set_uc: Arc::new(UpdateRuleSetUseCase::new(rule_set_repo.clone())),
        delete_rule_set_uc: Arc::new(DeleteRuleSetUseCase::new(rule_set_repo.clone())),
        publish_rule_set_uc: Arc::new(PublishRuleSetUseCase::new(
            rule_set_repo.clone(),
            version_repo.clone(),
        )),
        rollback_rule_set_uc: Arc::new(RollbackRuleSetUseCase::new(
            rule_set_repo.clone(),
            version_repo,
        )),
        evaluate_uc: Arc::new(EvaluateUseCase::new(
            rule_set_repo,
            rule_repo,
            eval_log_repo,
        )),
        list_evaluation_logs_uc: Arc::new(ListEvaluationLogsUseCase::new(Arc::new(
            StubEvalLogRepo,
        ))),
        metrics,
        auth_state: None,
        backend_kind: "in-memory".to_string(),
    }
}

/// テスト用ルール生成ヘルパー
fn make_rule(name: &str) -> Rule {
    Rule::new(
        name.to_string(),
        "test rule".to_string(),
        10,
        serde_json::json!({"field": "status", "operator": "eq", "value": "active"}),
        serde_json::json!({"approved": true}),
    )
}

// ---------------------------------------------------------------------------
// health エンドポイントテスト
// ---------------------------------------------------------------------------

/// GET /healthz は {"status": "ok"} を返す
#[tokio::test]
async fn healthz_returns_ok() {
    let state = build_state(
        Arc::new(StubRuleRepo::new()),
        Arc::new(StubRuleSetRepo::new()),
    );
    let app = router(state);

    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

/// GET /readyz はin-memoryバックエンドのとき degraded を返す
#[tokio::test]
async fn readyz_in_memory_returns_degraded() {
    let state = build_state(
        Arc::new(StubRuleRepo::new()),
        Arc::new(StubRuleSetRepo::new()),
    );
    let app = router(state);

    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "degraded");
    assert_eq!(json["backend"], "in-memory");
}

// ---------------------------------------------------------------------------
// rules エンドポイントテスト
// ---------------------------------------------------------------------------

/// GET /api/v1/rules はルールが0件の場合に空リストを返す
#[tokio::test]
async fn list_rules_empty() {
    let state = build_state(
        Arc::new(StubRuleRepo::new()),
        Arc::new(StubRuleSetRepo::new()),
    );
    let app = router(state);

    let req = Request::builder()
        .uri("/api/v1/rules")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["rules"], serde_json::json!([]));
    assert_eq!(json["pagination"]["total_count"], 0);
}

/// GET /api/v1/rules/{id} は存在しないIDに対して404を返す
#[tokio::test]
async fn get_rule_not_found() {
    let state = build_state(
        Arc::new(StubRuleRepo::new()),
        Arc::new(StubRuleSetRepo::new()),
    );
    let app = router(state);

    let req = Request::builder()
        .uri(&format!("/api/v1/rules/{}", Uuid::new_v4()))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["code"], "SYS_RULE_NOT_FOUND");
}

/// GET /api/v1/rules/{id} は存在するルールを200で返す
#[tokio::test]
async fn get_rule_found() {
    let rule = make_rule("approval-rule");
    let rule_id = rule.id;
    let rule_repo = Arc::new(StubRuleRepo::with_rules(vec![rule]));
    let state = build_state(rule_repo, Arc::new(StubRuleSetRepo::new()));
    let app = router(state);

    let req = Request::builder()
        .uri(&format!("/api/v1/rules/{}", rule_id))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "approval-rule");
}

/// POST /api/v1/rules は正常なボディで201を返す
#[tokio::test]
async fn create_rule_success() {
    let state = build_state(
        Arc::new(StubRuleRepo::new()),
        Arc::new(StubRuleSetRepo::new()),
    );
    let app = router(state);

    let body = serde_json::json!({
        "name": "new-approval-rule",
        "description": "テスト用ルール",
        "priority": 10,
        "when": {"field": "amount", "operator": "gt", "value": 1000},
        "then": {"action": "require_approval"}
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/rules")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp_body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&resp_body).unwrap();
    assert_eq!(json["name"], "new-approval-rule");
}

/// POST /api/v1/rules は重複ルール名で409を返す
#[tokio::test]
async fn create_rule_duplicate_returns_conflict() {
    let existing = make_rule("duplicate-rule");
    let rule_repo = Arc::new(StubRuleRepo::with_rules(vec![existing]));
    let state = build_state(rule_repo, Arc::new(StubRuleSetRepo::new()));
    let app = router(state);

    let body = serde_json::json!({
        "name": "duplicate-rule",
        "description": "重複テスト",
        "priority": 5,
        "when": {"field": "status", "operator": "eq", "value": "active"},
        "then": {"result": "ok"}
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/rules")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

/// DELETE /api/v1/rules/{id} は存在するルールを削除して200を返す
#[tokio::test]
async fn delete_rule_success() {
    let rule = make_rule("rule-to-delete");
    let rule_id = rule.id;
    let rule_repo = Arc::new(StubRuleRepo::with_rules(vec![rule]));
    let state = build_state(rule_repo, Arc::new(StubRuleSetRepo::new()));
    let app = router(state);

    let req = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/rules/{}", rule_id))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

/// DELETE /api/v1/rules/{id} は存在しないルールに対して404を返す
#[tokio::test]
async fn delete_rule_not_found() {
    let state = build_state(
        Arc::new(StubRuleRepo::new()),
        Arc::new(StubRuleSetRepo::new()),
    );
    let app = router(state);

    let req = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/rules/{}", Uuid::new_v4()))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// rule-sets エンドポイントテスト
// ---------------------------------------------------------------------------

/// GET /api/v1/rule-sets はルールセットが0件の場合に空リストを返す
#[tokio::test]
async fn list_rule_sets_empty() {
    let state = build_state(
        Arc::new(StubRuleRepo::new()),
        Arc::new(StubRuleSetRepo::new()),
    );
    let app = router(state);

    let req = Request::builder()
        .uri("/api/v1/rule-sets")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["rule_sets"], serde_json::json!([]));
}

/// GET /api/v1/rule-sets/{id} は存在しないIDに対して404を返す
#[tokio::test]
async fn get_rule_set_not_found() {
    let state = build_state(
        Arc::new(StubRuleRepo::new()),
        Arc::new(StubRuleSetRepo::new()),
    );
    let app = router(state);

    let req = Request::builder()
        .uri(&format!("/api/v1/rule-sets/{}", Uuid::new_v4()))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// evaluate エンドポイントテスト
// ---------------------------------------------------------------------------

/// POST /api/v1/evaluate はルールセットが存在しない場合に404を返す
#[tokio::test]
async fn evaluate_rule_set_not_found() {
    let state = build_state(
        Arc::new(StubRuleRepo::new()),
        Arc::new(StubRuleSetRepo::new()),
    );
    let app = router(state);

    let body = serde_json::json!({
        "rule_set": "nonexistent.rule_set",
        "input": {"status": "active"}
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/evaluate")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let resp_body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&resp_body).unwrap();
    assert_eq!(json["error"]["code"], "SYS_RULE_SET_NOT_FOUND");
}

// ---------------------------------------------------------------------------
// evaluation-logs エンドポイントテスト
// ---------------------------------------------------------------------------

/// GET /api/v1/evaluation-logs はログが0件の場合に空リストを返す
#[tokio::test]
async fn list_evaluation_logs_empty() {
    let state = build_state(
        Arc::new(StubRuleRepo::new()),
        Arc::new(StubRuleSetRepo::new()),
    );
    let app = router(state);

    let req = Request::builder()
        .uri("/api/v1/evaluation-logs")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["logs"], serde_json::json!([]));
}

/// GET /api/v1/rules?rule_set_id=invalid は400を返す
#[tokio::test]
async fn list_rules_invalid_rule_set_id_returns_bad_request() {
    let state = build_state(
        Arc::new(StubRuleRepo::new()),
        Arc::new(StubRuleSetRepo::new()),
    );
    let app = router(state);

    let req = Request::builder()
        .uri("/api/v1/rules?rule_set_id=not-a-uuid")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
