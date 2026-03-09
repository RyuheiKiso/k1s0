use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

use k1s0_rule_engine_server::domain::entity::rule::{
    EvaluationLog, EvaluationMode, Rule, RuleSet, RuleSetVersion,
};
use k1s0_rule_engine_server::domain::repository::{
    EvaluationLogRepository, RuleRepository, RuleSetRepository, RuleSetVersionRepository,
};
use k1s0_rule_engine_server::infrastructure::kafka_producer::RuleEventPublisher;

use k1s0_rule_engine_server::usecase::create_rule::{CreateRuleInput, CreateRuleUseCase};
use k1s0_rule_engine_server::usecase::create_rule_set::{CreateRuleSetInput, CreateRuleSetUseCase};
use k1s0_rule_engine_server::usecase::delete_rule::DeleteRuleUseCase;
use k1s0_rule_engine_server::usecase::delete_rule_set::DeleteRuleSetUseCase;
use k1s0_rule_engine_server::usecase::evaluate::{EvaluateInput, EvaluateUseCase};
use k1s0_rule_engine_server::usecase::get_rule::GetRuleUseCase;
use k1s0_rule_engine_server::usecase::get_rule_set::GetRuleSetUseCase;
use k1s0_rule_engine_server::usecase::list_evaluation_logs::{
    ListEvaluationLogsInput, ListEvaluationLogsUseCase,
};
use k1s0_rule_engine_server::usecase::list_rule_sets::{ListRuleSetsInput, ListRuleSetsUseCase};
use k1s0_rule_engine_server::usecase::list_rules::{ListRulesInput, ListRulesUseCase};
use k1s0_rule_engine_server::usecase::publish_rule_set::PublishRuleSetUseCase;
use k1s0_rule_engine_server::usecase::rollback_rule_set::RollbackRuleSetUseCase;
use k1s0_rule_engine_server::usecase::update_rule::{UpdateRuleInput, UpdateRuleUseCase};
use k1s0_rule_engine_server::usecase::update_rule_set::{
    UpdateRuleSetInput, UpdateRuleSetUseCase,
};

// ============================================================
// Stub implementations
// ============================================================

struct StubRuleRepository {
    rules: RwLock<Vec<Rule>>,
    should_error: bool,
}

impl StubRuleRepository {
    fn new() -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
            should_error: false,
        }
    }

    fn with_rules(rules: Vec<Rule>) -> Self {
        Self {
            rules: RwLock::new(rules),
            should_error: false,
        }
    }

    fn with_error() -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
            should_error: true,
        }
    }
}

#[async_trait]
impl RuleRepository for StubRuleRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Rule>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let rules = self.rules.read().await;
        Ok(rules.iter().find(|r| r.id == *id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Rule>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        Ok(self.rules.read().await.clone())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        _rule_set_id: Option<Uuid>,
        _domain: Option<String>,
    ) -> anyhow::Result<(Vec<Rule>, u64)> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let rules = self.rules.read().await;
        let total = rules.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(rules.len());
        let page_rules = if start < rules.len() {
            rules[start..end].to_vec()
        } else {
            vec![]
        };
        Ok((page_rules, total))
    }

    async fn create(&self, rule: &Rule) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        self.rules.write().await.push(rule.clone());
        Ok(())
    }

    async fn update(&self, rule: &Rule) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let mut rules = self.rules.write().await;
        if let Some(existing) = rules.iter_mut().find(|r| r.id == rule.id) {
            *existing = rule.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let mut rules = self.rules.write().await;
        let len_before = rules.len();
        rules.retain(|r| r.id != *id);
        Ok(rules.len() < len_before)
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let rules = self.rules.read().await;
        Ok(rules.iter().any(|r| r.name == name))
    }

    async fn find_by_ids(&self, ids: &[Uuid]) -> anyhow::Result<Vec<Rule>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let rules = self.rules.read().await;
        Ok(rules.iter().filter(|r| ids.contains(&r.id)).cloned().collect())
    }
}

struct StubRuleSetRepository {
    rule_sets: RwLock<Vec<RuleSet>>,
    should_error: bool,
}

impl StubRuleSetRepository {
    fn new() -> Self {
        Self {
            rule_sets: RwLock::new(Vec::new()),
            should_error: false,
        }
    }

    fn with_rule_sets(rule_sets: Vec<RuleSet>) -> Self {
        Self {
            rule_sets: RwLock::new(rule_sets),
            should_error: false,
        }
    }

    fn with_error() -> Self {
        Self {
            rule_sets: RwLock::new(Vec::new()),
            should_error: true,
        }
    }
}

#[async_trait]
impl RuleSetRepository for StubRuleSetRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<RuleSet>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let sets = self.rule_sets.read().await;
        Ok(sets.iter().find(|rs| rs.id == *id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<RuleSet>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        Ok(self.rule_sets.read().await.clone())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        domain: Option<String>,
    ) -> anyhow::Result<(Vec<RuleSet>, u64)> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let sets = self.rule_sets.read().await;
        let filtered: Vec<_> = if let Some(ref d) = domain {
            sets.iter().filter(|rs| rs.domain == *d).cloned().collect()
        } else {
            sets.clone()
        };
        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(filtered.len());
        let page_sets = if start < filtered.len() {
            filtered[start..end].to_vec()
        } else {
            vec![]
        };
        Ok((page_sets, total))
    }

    async fn find_by_domain_and_name(
        &self,
        domain: &str,
        name: &str,
    ) -> anyhow::Result<Option<RuleSet>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let sets = self.rule_sets.read().await;
        Ok(sets
            .iter()
            .find(|rs| rs.domain == domain && rs.name == name)
            .cloned())
    }

    async fn create(&self, rule_set: &RuleSet) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        self.rule_sets.write().await.push(rule_set.clone());
        Ok(())
    }

    async fn update(&self, rule_set: &RuleSet) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let mut sets = self.rule_sets.write().await;
        if let Some(existing) = sets.iter_mut().find(|rs| rs.id == rule_set.id) {
            *existing = rule_set.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let mut sets = self.rule_sets.write().await;
        let len_before = sets.len();
        sets.retain(|rs| rs.id != *id);
        Ok(sets.len() < len_before)
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let sets = self.rule_sets.read().await;
        Ok(sets.iter().any(|rs| rs.name == name))
    }
}

struct StubRuleSetVersionRepository {
    versions: RwLock<Vec<RuleSetVersion>>,
    should_error: bool,
}

impl StubRuleSetVersionRepository {
    fn new() -> Self {
        Self {
            versions: RwLock::new(Vec::new()),
            should_error: false,
        }
    }

    fn with_versions(versions: Vec<RuleSetVersion>) -> Self {
        Self {
            versions: RwLock::new(versions),
            should_error: false,
        }
    }
}

#[async_trait]
impl RuleSetVersionRepository for StubRuleSetVersionRepository {
    async fn find_by_rule_set_id_and_version(
        &self,
        rule_set_id: &Uuid,
        version: u32,
    ) -> anyhow::Result<Option<RuleSetVersion>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let versions = self.versions.read().await;
        Ok(versions
            .iter()
            .find(|v| v.rule_set_id == *rule_set_id && v.version == version)
            .cloned())
    }

    async fn find_latest_by_rule_set_id(
        &self,
        rule_set_id: &Uuid,
    ) -> anyhow::Result<Option<RuleSetVersion>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let versions = self.versions.read().await;
        Ok(versions
            .iter()
            .filter(|v| v.rule_set_id == *rule_set_id)
            .max_by_key(|v| v.version)
            .cloned())
    }

    async fn create(&self, version: &RuleSetVersion) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        self.versions.write().await.push(version.clone());
        Ok(())
    }
}

struct StubEvaluationLogRepository {
    logs: RwLock<Vec<EvaluationLog>>,
    should_error: bool,
}

impl StubEvaluationLogRepository {
    fn new() -> Self {
        Self {
            logs: RwLock::new(Vec::new()),
            should_error: false,
        }
    }
}

#[async_trait]
impl EvaluationLogRepository for StubEvaluationLogRepository {
    async fn create(&self, log: &EvaluationLog) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        self.logs.write().await.push(log.clone());
        Ok(())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        rule_set_name: Option<String>,
        _domain: Option<String>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> anyhow::Result<(Vec<EvaluationLog>, u64)> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let logs = self.logs.read().await;
        let filtered: Vec<_> = logs
            .iter()
            .filter(|l| {
                if let Some(ref name) = rule_set_name {
                    if l.rule_set_name != *name {
                        return false;
                    }
                }
                if let Some(ref f) = from {
                    if l.evaluated_at < *f {
                        return false;
                    }
                }
                if let Some(ref t) = to {
                    if l.evaluated_at > *t {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(filtered.len());
        let page_logs = if start < filtered.len() {
            filtered[start..end].to_vec()
        } else {
            vec![]
        };
        Ok((page_logs, total))
    }
}

struct StubEventPublisher;

#[async_trait]
impl RuleEventPublisher for StubEventPublisher {
    async fn publish_rule_changed(
        &self,
        _event: &k1s0_rule_engine_server::infrastructure::kafka_producer::RuleChangedEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ============================================================
// Helpers
// ============================================================

fn make_rule(name: &str, priority: i32, condition: serde_json::Value, result: serde_json::Value) -> Rule {
    let now = Utc::now();
    Rule {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: format!("{} description", name),
        priority,
        when_condition: condition,
        then_result: result,
        enabled: true,
        version: 1,
        created_at: now,
        updated_at: now,
    }
}

fn make_rule_set(
    name: &str,
    domain: &str,
    mode: EvaluationMode,
    rule_ids: Vec<Uuid>,
) -> RuleSet {
    let now = Utc::now();
    RuleSet {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: format!("{} description", name),
        domain: domain.to_string(),
        evaluation_mode: mode,
        default_result: serde_json::json!({"action": "default"}),
        rule_ids,
        current_version: 0,
        enabled: true,
        created_at: now,
        updated_at: now,
    }
}

fn make_publisher() -> Arc<dyn RuleEventPublisher> {
    Arc::new(StubEventPublisher)
}

// ============================================================
// CreateRule tests
// ============================================================

#[tokio::test]
async fn create_rule_success() {
    let repo = Arc::new(StubRuleRepository::new());
    let uc = CreateRuleUseCase::with_publisher(repo.clone(), make_publisher());

    let input = CreateRuleInput {
        name: "tax-rule".to_string(),
        description: "Tax calculation rule".to_string(),
        priority: 10,
        when_condition: serde_json::json!({"field": "category", "operator": "eq", "value": "food"}),
        then_result: serde_json::json!({"tax_rate": 0.08}),
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.name, "tax-rule");
    assert_eq!(result.priority, 10);
    assert!(result.enabled);
    assert_eq!(result.version, 1);

    // Verify stored
    let rules = repo.rules.read().await;
    assert_eq!(rules.len(), 1);
}

#[tokio::test]
async fn create_rule_empty_name_validation_error() {
    let repo = Arc::new(StubRuleRepository::new());
    let uc = CreateRuleUseCase::with_publisher(repo, make_publisher());

    let input = CreateRuleInput {
        name: "".to_string(),
        description: "".to_string(),
        priority: 10,
        when_condition: serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        then_result: serde_json::json!({}),
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("name is required"));
}

#[tokio::test]
async fn create_rule_invalid_priority_too_low() {
    let repo = Arc::new(StubRuleRepository::new());
    let uc = CreateRuleUseCase::with_publisher(repo, make_publisher());

    let input = CreateRuleInput {
        name: "test".to_string(),
        description: "".to_string(),
        priority: 0,
        when_condition: serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        then_result: serde_json::json!({}),
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("priority must be between 1 and 1000"));
}

#[tokio::test]
async fn create_rule_invalid_priority_too_high() {
    let repo = Arc::new(StubRuleRepository::new());
    let uc = CreateRuleUseCase::with_publisher(repo, make_publisher());

    let input = CreateRuleInput {
        name: "test".to_string(),
        description: "".to_string(),
        priority: 1001,
        when_condition: serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        then_result: serde_json::json!({}),
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("priority"));
}

#[tokio::test]
async fn create_rule_invalid_condition() {
    let repo = Arc::new(StubRuleRepository::new());
    let uc = CreateRuleUseCase::with_publisher(repo, make_publisher());

    let input = CreateRuleInput {
        name: "test".to_string(),
        description: "".to_string(),
        priority: 10,
        when_condition: serde_json::json!({"field": "x", "operator": "between", "value": 1}),
        then_result: serde_json::json!({}),
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("unknown operator"));
}

#[tokio::test]
async fn create_rule_already_exists() {
    let existing = make_rule(
        "existing-rule",
        10,
        serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        serde_json::json!({}),
    );
    let repo = Arc::new(StubRuleRepository::with_rules(vec![existing]));
    let uc = CreateRuleUseCase::with_publisher(repo, make_publisher());

    let input = CreateRuleInput {
        name: "existing-rule".to_string(),
        description: "".to_string(),
        priority: 10,
        when_condition: serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        then_result: serde_json::json!({}),
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

// ============================================================
// GetRule tests
// ============================================================

#[tokio::test]
async fn get_rule_success() {
    let rule = make_rule(
        "my-rule",
        5,
        serde_json::json!({"field": "status", "operator": "eq", "value": "active"}),
        serde_json::json!({"action": "approve"}),
    );
    let rule_id = rule.id;
    let repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let uc = GetRuleUseCase::new(repo);

    let result = uc.execute(&rule_id).await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "my-rule");
}

#[tokio::test]
async fn get_rule_not_found() {
    let repo = Arc::new(StubRuleRepository::new());
    let uc = GetRuleUseCase::new(repo);

    let result = uc.execute(&Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn get_rule_repo_error() {
    let repo = Arc::new(StubRuleRepository::with_error());
    let uc = GetRuleUseCase::new(repo);

    let result = uc.execute(&Uuid::new_v4()).await;
    assert!(result.is_err());
}

// ============================================================
// UpdateRule tests
// ============================================================

#[tokio::test]
async fn update_rule_success() {
    let rule = make_rule(
        "my-rule",
        5,
        serde_json::json!({"field": "status", "operator": "eq", "value": "active"}),
        serde_json::json!({"action": "approve"}),
    );
    let rule_id = rule.id;
    let repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let uc = UpdateRuleUseCase::with_publisher(repo.clone(), make_publisher());

    let input = UpdateRuleInput {
        id: rule_id,
        description: Some("updated description".to_string()),
        priority: Some(20),
        when_condition: None,
        then_result: Some(serde_json::json!({"action": "reject"})),
        enabled: Some(false),
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.description, "updated description");
    assert_eq!(result.priority, 20);
    assert_eq!(result.then_result, serde_json::json!({"action": "reject"}));
    assert!(!result.enabled);
    assert_eq!(result.version, 2, "version should be incremented");
}

#[tokio::test]
async fn update_rule_not_found() {
    let repo = Arc::new(StubRuleRepository::new());
    let uc = UpdateRuleUseCase::with_publisher(repo, make_publisher());

    let input = UpdateRuleInput {
        id: Uuid::new_v4(),
        description: Some("x".to_string()),
        priority: None,
        when_condition: None,
        then_result: None,
        enabled: None,
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn update_rule_invalid_priority() {
    let rule = make_rule(
        "my-rule",
        5,
        serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        serde_json::json!({}),
    );
    let rule_id = rule.id;
    let repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let uc = UpdateRuleUseCase::with_publisher(repo, make_publisher());

    let input = UpdateRuleInput {
        id: rule_id,
        description: None,
        priority: Some(0),
        when_condition: None,
        then_result: None,
        enabled: None,
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("priority"));
}

#[tokio::test]
async fn update_rule_invalid_condition() {
    let rule = make_rule(
        "my-rule",
        5,
        serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        serde_json::json!({}),
    );
    let rule_id = rule.id;
    let repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let uc = UpdateRuleUseCase::with_publisher(repo, make_publisher());

    let input = UpdateRuleInput {
        id: rule_id,
        description: None,
        priority: None,
        when_condition: Some(serde_json::json!({"field": "x", "operator": "between", "value": 1})),
        then_result: None,
        enabled: None,
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("unknown operator"));
}

// ============================================================
// DeleteRule tests
// ============================================================

#[tokio::test]
async fn delete_rule_success() {
    let rule = make_rule(
        "my-rule",
        5,
        serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        serde_json::json!({}),
    );
    let rule_id = rule.id;
    let repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let uc = DeleteRuleUseCase::with_publisher(repo.clone(), make_publisher());

    uc.execute(&rule_id).await.unwrap();
    let rules = repo.rules.read().await;
    assert!(rules.is_empty());
}

#[tokio::test]
async fn delete_rule_not_found() {
    let repo = Arc::new(StubRuleRepository::new());
    let uc = DeleteRuleUseCase::with_publisher(repo, make_publisher());

    let err = uc.execute(&Uuid::new_v4()).await.unwrap_err();
    assert!(err.to_string().contains("not found"));
}

// ============================================================
// ListRules tests
// ============================================================

#[tokio::test]
async fn list_rules_empty() {
    let repo = Arc::new(StubRuleRepository::new());
    let uc = ListRulesUseCase::new(repo);

    let input = ListRulesInput {
        page: 1,
        page_size: 10,
        rule_set_id: None,
        domain: None,
    };
    let result = uc.execute(&input).await.unwrap();
    assert!(result.rules.is_empty());
    assert_eq!(result.total_count, 0);
    assert!(!result.has_next);
}

#[tokio::test]
async fn list_rules_with_pagination() {
    let rules: Vec<Rule> = (0..5)
        .map(|i| {
            make_rule(
                &format!("rule-{}", i),
                i + 1,
                serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
                serde_json::json!({}),
            )
        })
        .collect();
    let repo = Arc::new(StubRuleRepository::with_rules(rules));
    let uc = ListRulesUseCase::new(repo);

    let input = ListRulesInput {
        page: 1,
        page_size: 3,
        rule_set_id: None,
        domain: None,
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.rules.len(), 3);
    assert_eq!(result.total_count, 5);
    assert!(result.has_next);
}

#[tokio::test]
async fn list_rules_last_page() {
    let rules: Vec<Rule> = (0..5)
        .map(|i| {
            make_rule(
                &format!("rule-{}", i),
                i + 1,
                serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
                serde_json::json!({}),
            )
        })
        .collect();
    let repo = Arc::new(StubRuleRepository::with_rules(rules));
    let uc = ListRulesUseCase::new(repo);

    let input = ListRulesInput {
        page: 2,
        page_size: 3,
        rule_set_id: None,
        domain: None,
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.rules.len(), 2);
    assert!(!result.has_next);
}

// ============================================================
// CreateRuleSet tests
// ============================================================

#[tokio::test]
async fn create_rule_set_success() {
    let repo = Arc::new(StubRuleSetRepository::new());
    let uc = CreateRuleSetUseCase::with_publisher(repo.clone(), make_publisher());

    let input = CreateRuleSetInput {
        name: "tax-rules".to_string(),
        description: "Tax calculation rules".to_string(),
        domain: "pricing".to_string(),
        evaluation_mode: "first_match".to_string(),
        default_result: serde_json::json!({"tax_rate": 0.10}),
        rule_ids: vec![],
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.name, "tax-rules");
    assert_eq!(result.domain, "pricing");
    assert_eq!(result.evaluation_mode, EvaluationMode::FirstMatch);
    assert!(result.enabled);

    let sets = repo.rule_sets.read().await;
    assert_eq!(sets.len(), 1);
}

#[tokio::test]
async fn create_rule_set_empty_name() {
    let repo = Arc::new(StubRuleSetRepository::new());
    let uc = CreateRuleSetUseCase::with_publisher(repo, make_publisher());

    let input = CreateRuleSetInput {
        name: "".to_string(),
        description: "".to_string(),
        domain: "pricing".to_string(),
        evaluation_mode: "first_match".to_string(),
        default_result: serde_json::json!({}),
        rule_ids: vec![],
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("name is required"));
}

#[tokio::test]
async fn create_rule_set_empty_domain() {
    let repo = Arc::new(StubRuleSetRepository::new());
    let uc = CreateRuleSetUseCase::with_publisher(repo, make_publisher());

    let input = CreateRuleSetInput {
        name: "test".to_string(),
        description: "".to_string(),
        domain: "".to_string(),
        evaluation_mode: "first_match".to_string(),
        default_result: serde_json::json!({}),
        rule_ids: vec![],
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("domain is required"));
}

#[tokio::test]
async fn create_rule_set_invalid_evaluation_mode() {
    let repo = Arc::new(StubRuleSetRepository::new());
    let uc = CreateRuleSetUseCase::with_publisher(repo, make_publisher());

    let input = CreateRuleSetInput {
        name: "test".to_string(),
        description: "".to_string(),
        domain: "pricing".to_string(),
        evaluation_mode: "invalid_mode".to_string(),
        default_result: serde_json::json!({}),
        rule_ids: vec![],
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("invalid evaluation_mode"));
}

#[tokio::test]
async fn create_rule_set_already_exists() {
    let existing = make_rule_set("existing-set", "pricing", EvaluationMode::FirstMatch, vec![]);
    let repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![existing]));
    let uc = CreateRuleSetUseCase::with_publisher(repo, make_publisher());

    let input = CreateRuleSetInput {
        name: "existing-set".to_string(),
        description: "".to_string(),
        domain: "pricing".to_string(),
        evaluation_mode: "first_match".to_string(),
        default_result: serde_json::json!({}),
        rule_ids: vec![],
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[tokio::test]
async fn create_rule_set_all_match_mode() {
    let repo = Arc::new(StubRuleSetRepository::new());
    let uc = CreateRuleSetUseCase::with_publisher(repo, make_publisher());

    let input = CreateRuleSetInput {
        name: "all-match-set".to_string(),
        description: "".to_string(),
        domain: "validation".to_string(),
        evaluation_mode: "all_match".to_string(),
        default_result: serde_json::json!({"valid": true}),
        rule_ids: vec![],
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.evaluation_mode, EvaluationMode::AllMatch);
}

// ============================================================
// GetRuleSet tests
// ============================================================

#[tokio::test]
async fn get_rule_set_success() {
    let rule_set = make_rule_set("my-set", "pricing", EvaluationMode::FirstMatch, vec![]);
    let rs_id = rule_set.id;
    let repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let uc = GetRuleSetUseCase::new(repo);

    let result = uc.execute(&rs_id).await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "my-set");
}

#[tokio::test]
async fn get_rule_set_not_found() {
    let repo = Arc::new(StubRuleSetRepository::new());
    let uc = GetRuleSetUseCase::new(repo);

    let result = uc.execute(&Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());
}

// ============================================================
// UpdateRuleSet tests
// ============================================================

#[tokio::test]
async fn update_rule_set_success() {
    let rule_set = make_rule_set("my-set", "pricing", EvaluationMode::FirstMatch, vec![]);
    let rs_id = rule_set.id;
    let repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let uc = UpdateRuleSetUseCase::with_publisher(repo, make_publisher());

    let new_rule_id = Uuid::new_v4();
    let input = UpdateRuleSetInput {
        id: rs_id,
        description: Some("updated desc".to_string()),
        evaluation_mode: Some("all_match".to_string()),
        default_result: Some(serde_json::json!({"action": "updated_default"})),
        rule_ids: Some(vec![new_rule_id]),
        enabled: Some(false),
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.description, "updated desc");
    assert_eq!(result.evaluation_mode, EvaluationMode::AllMatch);
    assert!(!result.enabled);
    assert_eq!(result.rule_ids, vec![new_rule_id]);
}

#[tokio::test]
async fn update_rule_set_not_found() {
    let repo = Arc::new(StubRuleSetRepository::new());
    let uc = UpdateRuleSetUseCase::with_publisher(repo, make_publisher());

    let input = UpdateRuleSetInput {
        id: Uuid::new_v4(),
        description: Some("x".to_string()),
        evaluation_mode: None,
        default_result: None,
        rule_ids: None,
        enabled: None,
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn update_rule_set_invalid_evaluation_mode() {
    let rule_set = make_rule_set("my-set", "pricing", EvaluationMode::FirstMatch, vec![]);
    let rs_id = rule_set.id;
    let repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let uc = UpdateRuleSetUseCase::with_publisher(repo, make_publisher());

    let input = UpdateRuleSetInput {
        id: rs_id,
        description: None,
        evaluation_mode: Some("bad_mode".to_string()),
        default_result: None,
        rule_ids: None,
        enabled: None,
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("invalid evaluation_mode"));
}

// ============================================================
// DeleteRuleSet tests
// ============================================================

#[tokio::test]
async fn delete_rule_set_success() {
    let rule_set = make_rule_set("my-set", "pricing", EvaluationMode::FirstMatch, vec![]);
    let rs_id = rule_set.id;
    let repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let uc = DeleteRuleSetUseCase::with_publisher(repo.clone(), make_publisher());

    uc.execute(&rs_id).await.unwrap();
    let sets = repo.rule_sets.read().await;
    assert!(sets.is_empty());
}

#[tokio::test]
async fn delete_rule_set_not_found() {
    let repo = Arc::new(StubRuleSetRepository::new());
    let uc = DeleteRuleSetUseCase::with_publisher(repo, make_publisher());

    let err = uc.execute(&Uuid::new_v4()).await.unwrap_err();
    assert!(err.to_string().contains("not found"));
}

// ============================================================
// ListRuleSets tests
// ============================================================

#[tokio::test]
async fn list_rule_sets_empty() {
    let repo = Arc::new(StubRuleSetRepository::new());
    let uc = ListRuleSetsUseCase::new(repo);

    let input = ListRuleSetsInput {
        page: 1,
        page_size: 10,
        domain: None,
    };
    let result = uc.execute(&input).await.unwrap();
    assert!(result.rule_sets.is_empty());
    assert_eq!(result.total_count, 0);
}

#[tokio::test]
async fn list_rule_sets_with_domain_filter() {
    let sets = vec![
        make_rule_set("set-1", "pricing", EvaluationMode::FirstMatch, vec![]),
        make_rule_set("set-2", "pricing", EvaluationMode::AllMatch, vec![]),
        make_rule_set("set-3", "validation", EvaluationMode::FirstMatch, vec![]),
    ];
    let repo = Arc::new(StubRuleSetRepository::with_rule_sets(sets));
    let uc = ListRuleSetsUseCase::new(repo);

    let input = ListRuleSetsInput {
        page: 1,
        page_size: 10,
        domain: Some("pricing".to_string()),
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.rule_sets.len(), 2);
    assert_eq!(result.total_count, 2);
}

// ============================================================
// Evaluate tests (core business logic)
// ============================================================

#[tokio::test]
async fn evaluate_first_match_returns_first_matching_rule() {
    let rule1 = make_rule(
        "low-priority",
        100,
        serde_json::json!({"field": "amount", "operator": "gt", "value": 0}),
        serde_json::json!({"tax_rate": 0.05}),
    );
    let rule2 = make_rule(
        "high-priority",
        1,
        serde_json::json!({"field": "amount", "operator": "gt", "value": 1000}),
        serde_json::json!({"tax_rate": 0.10}),
    );
    let rule1_id = rule1.id;
    let rule2_id = rule2.id;

    let rule_set = make_rule_set(
        "tax-calc",
        "pricing",
        EvaluationMode::FirstMatch,
        vec![rule1_id, rule2_id],
    );

    let rule_repo = Arc::new(StubRuleRepository::with_rules(vec![rule1, rule2]));
    let rule_set_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo.clone());

    let input = EvaluateInput {
        rule_set: "pricing.tax-calc".to_string(),
        input: serde_json::json!({"amount": 5000}),
        context: serde_json::json!({"user_id": "u1"}),
        dry_run: false,
    };
    let result = uc.execute(&input).await.unwrap();

    // Both rules match (amount > 0 and amount > 1000),
    // but FirstMatch returns the highest priority (lowest number) first
    assert_eq!(result.matched_rules.len(), 1);
    assert_eq!(result.matched_rules[0].name, "high-priority");
    assert_eq!(result.result, serde_json::json!({"tax_rate": 0.10}));
    assert!(!result.default_applied);

    // Verify evaluation log was written
    let logs = eval_log_repo.logs.read().await;
    assert_eq!(logs.len(), 1);
}

#[tokio::test]
async fn evaluate_all_match_returns_all_matching_rules() {
    let rule1 = make_rule(
        "rule-a",
        1,
        serde_json::json!({"field": "category", "operator": "eq", "value": "food"}),
        serde_json::json!({"discount": 0.05}),
    );
    let rule2 = make_rule(
        "rule-b",
        2,
        serde_json::json!({"field": "amount", "operator": "gt", "value": 100}),
        serde_json::json!({"bonus": true}),
    );
    let rule1_id = rule1.id;
    let rule2_id = rule2.id;

    let rule_set = make_rule_set(
        "promo",
        "pricing",
        EvaluationMode::AllMatch,
        vec![rule1_id, rule2_id],
    );

    let rule_repo = Arc::new(StubRuleRepository::with_rules(vec![rule1, rule2]));
    let rule_set_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo);

    let input = EvaluateInput {
        rule_set: "pricing.promo".to_string(),
        input: serde_json::json!({"category": "food", "amount": 200}),
        context: serde_json::json!({}),
        dry_run: false,
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.matched_rules.len(), 2);
    assert!(!result.default_applied);
    // AllMatch returns array of results
    let arr = result.result.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}

#[tokio::test]
async fn evaluate_no_match_returns_default() {
    let rule = make_rule(
        "never-match",
        1,
        serde_json::json!({"field": "status", "operator": "eq", "value": "impossible"}),
        serde_json::json!({"x": 1}),
    );
    let rule_id = rule.id;

    let mut rule_set = make_rule_set(
        "test-set",
        "test",
        EvaluationMode::FirstMatch,
        vec![rule_id],
    );
    rule_set.default_result = serde_json::json!({"action": "fallback"});

    let rule_repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let rule_set_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo);

    let input = EvaluateInput {
        rule_set: "test.test-set".to_string(),
        input: serde_json::json!({"status": "active"}),
        context: serde_json::json!({}),
        dry_run: false,
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.matched_rules.is_empty());
    assert!(result.default_applied);
    assert_eq!(result.result, serde_json::json!({"action": "fallback"}));
}

#[tokio::test]
async fn evaluate_dry_run_does_not_write_log() {
    let rule = make_rule(
        "some-rule",
        1,
        serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        serde_json::json!({"ok": true}),
    );
    let rule_id = rule.id;

    let rule_set = make_rule_set("test-set", "test", EvaluationMode::FirstMatch, vec![rule_id]);

    let rule_repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let rule_set_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo.clone());

    let input = EvaluateInput {
        rule_set: "test.test-set".to_string(),
        input: serde_json::json!({"x": "y"}),
        context: serde_json::json!({}),
        dry_run: true,
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.matched_rules.len(), 1);

    // No log should be written in dry_run mode
    let logs = eval_log_repo.logs.read().await;
    assert!(logs.is_empty(), "dry_run should not write evaluation logs");
}

#[tokio::test]
async fn evaluate_rule_set_not_found() {
    let rule_repo = Arc::new(StubRuleRepository::new());
    let rule_set_repo = Arc::new(StubRuleSetRepository::new());
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo);

    let input = EvaluateInput {
        rule_set: "pricing.nonexistent".to_string(),
        input: serde_json::json!({}),
        context: serde_json::json!({}),
        dry_run: false,
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn evaluate_invalid_rule_set_format() {
    let rule_repo = Arc::new(StubRuleRepository::new());
    let rule_set_repo = Arc::new(StubRuleSetRepository::new());
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo);

    let input = EvaluateInput {
        rule_set: "no-dot-here".to_string(),
        input: serde_json::json!({}),
        context: serde_json::json!({}),
        dry_run: false,
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("domain"));
}

#[tokio::test]
async fn evaluate_disabled_rules_are_skipped() {
    let mut disabled_rule = make_rule(
        "disabled-rule",
        1,
        serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        serde_json::json!({"should_not_appear": true}),
    );
    disabled_rule.enabled = false;
    let disabled_id = disabled_rule.id;

    let enabled_rule = make_rule(
        "enabled-rule",
        2,
        serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        serde_json::json!({"correct": true}),
    );
    let enabled_id = enabled_rule.id;

    let rule_set = make_rule_set(
        "test-set",
        "test",
        EvaluationMode::AllMatch,
        vec![disabled_id, enabled_id],
    );

    let rule_repo = Arc::new(StubRuleRepository::with_rules(vec![disabled_rule, enabled_rule]));
    let rule_set_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo);

    let input = EvaluateInput {
        rule_set: "test.test-set".to_string(),
        input: serde_json::json!({"x": "y"}),
        context: serde_json::json!({}),
        dry_run: true,
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.matched_rules.len(), 1);
    assert_eq!(result.matched_rules[0].name, "enabled-rule");
}

#[tokio::test]
async fn evaluate_rules_sorted_by_priority() {
    let rule_low = make_rule(
        "low-prio",
        100,
        serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        serde_json::json!({"prio": "low"}),
    );
    let rule_high = make_rule(
        "high-prio",
        1,
        serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        serde_json::json!({"prio": "high"}),
    );
    let rule_mid = make_rule(
        "mid-prio",
        50,
        serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        serde_json::json!({"prio": "mid"}),
    );
    let ids: Vec<Uuid> = vec![rule_low.id, rule_high.id, rule_mid.id];

    let rule_set = make_rule_set("priority-set", "test", EvaluationMode::AllMatch, ids);

    let rule_repo = Arc::new(StubRuleRepository::with_rules(vec![rule_low, rule_high, rule_mid]));
    let rule_set_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo);

    let input = EvaluateInput {
        rule_set: "test.priority-set".to_string(),
        input: serde_json::json!({"x": "y"}),
        context: serde_json::json!({}),
        dry_run: true,
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.matched_rules.len(), 3);
    assert_eq!(result.matched_rules[0].priority, 1, "highest priority first");
    assert_eq!(result.matched_rules[1].priority, 50);
    assert_eq!(result.matched_rules[2].priority, 100);
}

#[tokio::test]
async fn evaluate_complex_condition_with_combinators() {
    let rule = make_rule(
        "complex-rule",
        1,
        serde_json::json!({
            "all": [
                {"field": "item.category", "operator": "eq", "value": "food"},
                {"field": "item.price", "operator": "gt", "value": 100},
                {
                    "any": [
                        {"field": "customer.tier", "operator": "eq", "value": "gold"},
                        {"field": "customer.tier", "operator": "eq", "value": "platinum"}
                    ]
                }
            ]
        }),
        serde_json::json!({"discount": 0.15}),
    );
    let rule_id = rule.id;

    let rule_set = make_rule_set("promo", "pricing", EvaluationMode::FirstMatch, vec![rule_id]);

    let rule_repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let rule_set_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo);

    // Should match: food category, price > 100, gold tier
    let input = EvaluateInput {
        rule_set: "pricing.promo".to_string(),
        input: serde_json::json!({
            "item": {"category": "food", "price": 200},
            "customer": {"tier": "gold"}
        }),
        context: serde_json::json!({}),
        dry_run: true,
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.matched_rules.len(), 1);
    assert_eq!(result.result, serde_json::json!({"discount": 0.15}));
}

#[tokio::test]
async fn evaluate_complex_condition_no_match() {
    let rule = make_rule(
        "complex-rule",
        1,
        serde_json::json!({
            "all": [
                {"field": "item.category", "operator": "eq", "value": "food"},
                {"field": "item.price", "operator": "gt", "value": 100}
            ]
        }),
        serde_json::json!({"discount": 0.15}),
    );
    let rule_id = rule.id;

    let mut rule_set = make_rule_set("promo", "pricing", EvaluationMode::FirstMatch, vec![rule_id]);
    rule_set.default_result = serde_json::json!({"discount": 0.0});

    let rule_repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let rule_set_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo);

    // Should NOT match: food but price too low
    let input = EvaluateInput {
        rule_set: "pricing.promo".to_string(),
        input: serde_json::json!({
            "item": {"category": "food", "price": 50}
        }),
        context: serde_json::json!({}),
        dry_run: true,
    };
    let result = uc.execute(&input).await.unwrap();
    assert!(result.matched_rules.is_empty());
    assert!(result.default_applied);
    assert_eq!(result.result, serde_json::json!({"discount": 0.0}));
}

#[tokio::test]
async fn evaluate_with_in_operator() {
    let rule = make_rule(
        "region-rule",
        1,
        serde_json::json!({"field": "region", "operator": "in", "value": ["JP", "US", "UK"]}),
        serde_json::json!({"shipping": "free"}),
    );
    let rule_id = rule.id;

    let rule_set = make_rule_set("shipping", "logistics", EvaluationMode::FirstMatch, vec![rule_id]);

    let rule_repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let rule_set_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo);

    let input = EvaluateInput {
        rule_set: "logistics.shipping".to_string(),
        input: serde_json::json!({"region": "JP"}),
        context: serde_json::json!({}),
        dry_run: true,
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.matched_rules.len(), 1);
    assert_eq!(result.result, serde_json::json!({"shipping": "free"}));
}

#[tokio::test]
async fn evaluate_with_contains_operator() {
    let rule = make_rule(
        "name-rule",
        1,
        serde_json::json!({"field": "name", "operator": "contains", "value": "premium"}),
        serde_json::json!({"is_premium": true}),
    );
    let rule_id = rule.id;

    let rule_set = make_rule_set("classify", "catalog", EvaluationMode::FirstMatch, vec![rule_id]);

    let rule_repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let rule_set_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo);

    let input = EvaluateInput {
        rule_set: "catalog.classify".to_string(),
        input: serde_json::json!({"name": "premium membership plan"}),
        context: serde_json::json!({}),
        dry_run: true,
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.matched_rules.len(), 1);
}

#[tokio::test]
async fn evaluate_with_none_combinator() {
    let rule = make_rule(
        "exclusion-rule",
        1,
        serde_json::json!({
            "none": [
                {"field": "status", "operator": "eq", "value": "banned"},
                {"field": "status", "operator": "eq", "value": "suspended"}
            ]
        }),
        serde_json::json!({"allowed": true}),
    );
    let rule_id = rule.id;

    let rule_set = make_rule_set("access", "auth", EvaluationMode::FirstMatch, vec![rule_id]);

    let rule_repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let rule_set_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let uc = EvaluateUseCase::new(rule_set_repo, rule_repo, eval_log_repo);

    // "active" is neither "banned" nor "suspended" -> none = true -> match
    let input = EvaluateInput {
        rule_set: "auth.access".to_string(),
        input: serde_json::json!({"status": "active"}),
        context: serde_json::json!({}),
        dry_run: true,
    };
    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.matched_rules.len(), 1);
    assert_eq!(result.result, serde_json::json!({"allowed": true}));

    // "banned" -> none = false -> no match
    let input2 = EvaluateInput {
        rule_set: "auth.access".to_string(),
        input: serde_json::json!({"status": "banned"}),
        context: serde_json::json!({}),
        dry_run: true,
    };
    let result2 = uc.execute(&input2).await.unwrap();
    assert!(result2.matched_rules.is_empty());
}

// ============================================================
// PublishRuleSet tests
// ============================================================

#[tokio::test]
async fn publish_rule_set_success() {
    let rule_set = make_rule_set("my-set", "pricing", EvaluationMode::FirstMatch, vec![]);
    let rs_id = rule_set.id;
    let rs_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let ver_repo = Arc::new(StubRuleSetVersionRepository::new());

    let uc = PublishRuleSetUseCase::with_publisher(rs_repo.clone(), ver_repo.clone(), make_publisher());

    let result = uc.execute(&rs_id).await.unwrap();
    assert_eq!(result.published_version, 1);
    assert_eq!(result.previous_version, 0);
    assert_eq!(result.name, "my-set");

    // Verify version was saved
    let versions = ver_repo.versions.read().await;
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].version, 1);

    // Verify rule set was updated
    let sets = rs_repo.rule_sets.read().await;
    assert_eq!(sets[0].current_version, 1);
}

#[tokio::test]
async fn publish_rule_set_increments_version() {
    let mut rule_set = make_rule_set("my-set", "pricing", EvaluationMode::FirstMatch, vec![]);
    rule_set.current_version = 3;
    let rs_id = rule_set.id;
    let rs_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let ver_repo = Arc::new(StubRuleSetVersionRepository::new());

    let uc = PublishRuleSetUseCase::with_publisher(rs_repo, ver_repo, make_publisher());

    let result = uc.execute(&rs_id).await.unwrap();
    assert_eq!(result.published_version, 4);
    assert_eq!(result.previous_version, 3);
}

#[tokio::test]
async fn publish_rule_set_not_found() {
    let rs_repo = Arc::new(StubRuleSetRepository::new());
    let ver_repo = Arc::new(StubRuleSetVersionRepository::new());

    let uc = PublishRuleSetUseCase::with_publisher(rs_repo, ver_repo, make_publisher());

    let err = uc.execute(&Uuid::new_v4()).await.unwrap_err();
    assert!(err.to_string().contains("not found"));
}

// ============================================================
// RollbackRuleSet tests
// ============================================================

#[tokio::test]
async fn rollback_rule_set_success() {
    let rule_id_v1 = Uuid::new_v4();
    let rule_id_v2 = Uuid::new_v4();

    let mut rule_set = make_rule_set(
        "my-set",
        "pricing",
        EvaluationMode::FirstMatch,
        vec![rule_id_v2],
    );
    rule_set.current_version = 2;
    let rs_id = rule_set.id;

    let version1 = RuleSetVersion::new(
        rs_id,
        1,
        vec![rule_id_v1],
        serde_json::json!({"action": "v1_default"}),
        "admin".to_string(),
    );

    let rs_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let ver_repo = Arc::new(StubRuleSetVersionRepository::with_versions(vec![version1]));

    let uc = RollbackRuleSetUseCase::with_publisher(rs_repo.clone(), ver_repo, make_publisher());

    let result = uc.execute(&rs_id).await.unwrap();
    assert_eq!(result.rolled_back_to_version, 1);
    assert_eq!(result.previous_version, 2);

    // Verify rule set was restored to v1 state
    let sets = rs_repo.rule_sets.read().await;
    assert_eq!(sets[0].current_version, 1);
    assert_eq!(sets[0].rule_ids, vec![rule_id_v1]);
    assert_eq!(sets[0].default_result, serde_json::json!({"action": "v1_default"}));
}

#[tokio::test]
async fn rollback_rule_set_version_zero_error() {
    let rule_set = make_rule_set("my-set", "pricing", EvaluationMode::FirstMatch, vec![]);
    // current_version is 0, cannot rollback
    let rs_id = rule_set.id;

    let rs_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let ver_repo = Arc::new(StubRuleSetVersionRepository::new());

    let uc = RollbackRuleSetUseCase::with_publisher(rs_repo, ver_repo, make_publisher());

    let err = uc.execute(&rs_id).await.unwrap_err();
    assert!(err.to_string().contains("no previous version"));
}

#[tokio::test]
async fn rollback_rule_set_version_one_error() {
    let mut rule_set = make_rule_set("my-set", "pricing", EvaluationMode::FirstMatch, vec![]);
    rule_set.current_version = 1;
    let rs_id = rule_set.id;

    let rs_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let ver_repo = Arc::new(StubRuleSetVersionRepository::new());

    let uc = RollbackRuleSetUseCase::with_publisher(rs_repo, ver_repo, make_publisher());

    let err = uc.execute(&rs_id).await.unwrap_err();
    assert!(err.to_string().contains("no previous version"));
}

#[tokio::test]
async fn rollback_rule_set_not_found() {
    let rs_repo = Arc::new(StubRuleSetRepository::new());
    let ver_repo = Arc::new(StubRuleSetVersionRepository::new());

    let uc = RollbackRuleSetUseCase::with_publisher(rs_repo, ver_repo, make_publisher());

    let err = uc.execute(&Uuid::new_v4()).await.unwrap_err();
    assert!(err.to_string().contains("not found"));
}

// ============================================================
// ListEvaluationLogs tests
// ============================================================

#[tokio::test]
async fn list_evaluation_logs_empty() {
    let repo = Arc::new(StubEvaluationLogRepository::new());
    let uc = ListEvaluationLogsUseCase::new(repo);

    let input = ListEvaluationLogsInput {
        page: 1,
        page_size: 10,
        rule_set_name: None,
        domain: None,
        from: None,
        to: None,
    };
    let result = uc.execute(&input).await.unwrap();
    assert!(result.logs.is_empty());
    assert_eq!(result.total_count, 0);
}

#[tokio::test]
async fn list_evaluation_logs_after_evaluation() {
    // First, run an evaluation to populate logs
    let rule = make_rule(
        "log-test-rule",
        1,
        serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
        serde_json::json!({"ok": true}),
    );
    let rule_id = rule.id;

    let rule_set = make_rule_set("log-set", "test", EvaluationMode::FirstMatch, vec![rule_id]);

    let rule_repo = Arc::new(StubRuleRepository::with_rules(vec![rule]));
    let rule_set_repo = Arc::new(StubRuleSetRepository::with_rule_sets(vec![rule_set]));
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());

    let eval_uc = EvaluateUseCase::new(
        rule_set_repo,
        rule_repo,
        eval_log_repo.clone(),
    );

    // Execute evaluation (non-dry-run)
    let eval_input = EvaluateInput {
        rule_set: "test.log-set".to_string(),
        input: serde_json::json!({"x": "y"}),
        context: serde_json::json!({"trace_id": "abc"}),
        dry_run: false,
    };
    eval_uc.execute(&eval_input).await.unwrap();

    // Now list logs
    let list_uc = ListEvaluationLogsUseCase::new(eval_log_repo);
    let list_input = ListEvaluationLogsInput {
        page: 1,
        page_size: 10,
        rule_set_name: None,
        domain: None,
        from: None,
        to: None,
    };
    let result = list_uc.execute(&list_input).await.unwrap();
    assert_eq!(result.logs.len(), 1);
    assert_eq!(result.logs[0].rule_set_name, "test.log-set");
}

// ============================================================
// End-to-end workflow: create, publish, evaluate, rollback
// ============================================================

#[tokio::test]
async fn end_to_end_create_publish_evaluate_rollback() {
    let rule_repo = Arc::new(StubRuleRepository::new());
    let rule_set_repo = Arc::new(StubRuleSetRepository::new());
    let version_repo = Arc::new(StubRuleSetVersionRepository::new());
    let eval_log_repo = Arc::new(StubEvaluationLogRepository::new());
    let publisher = make_publisher();

    // 1. Create a rule
    let create_rule_uc = CreateRuleUseCase::with_publisher(rule_repo.clone(), publisher.clone());
    let rule = create_rule_uc
        .execute(&CreateRuleInput {
            name: "vip-discount".to_string(),
            description: "VIP customer discount".to_string(),
            priority: 1,
            when_condition: serde_json::json!({"field": "customer.tier", "operator": "eq", "value": "vip"}),
            then_result: serde_json::json!({"discount": 0.20}),
        })
        .await
        .unwrap();

    // 2. Create a rule set
    let create_rs_uc = CreateRuleSetUseCase::with_publisher(rule_set_repo.clone(), publisher.clone());
    let rule_set = create_rs_uc
        .execute(&CreateRuleSetInput {
            name: "discounts".to_string(),
            description: "Discount calculation".to_string(),
            domain: "pricing".to_string(),
            evaluation_mode: "first_match".to_string(),
            default_result: serde_json::json!({"discount": 0.0}),
            rule_ids: vec![rule.id],
        })
        .await
        .unwrap();

    // 3. Publish (v1)
    let publish_uc = PublishRuleSetUseCase::with_publisher(
        rule_set_repo.clone(),
        version_repo.clone(),
        publisher.clone(),
    );
    let pub_result = publish_uc.execute(&rule_set.id).await.unwrap();
    assert_eq!(pub_result.published_version, 1);

    // 4. Evaluate
    let eval_uc = EvaluateUseCase::new(
        rule_set_repo.clone(),
        rule_repo.clone(),
        eval_log_repo.clone(),
    );
    let eval_result = eval_uc
        .execute(&EvaluateInput {
            rule_set: "pricing.discounts".to_string(),
            input: serde_json::json!({"customer": {"tier": "vip"}}),
            context: serde_json::json!({}),
            dry_run: false,
        })
        .await
        .unwrap();
    assert_eq!(eval_result.matched_rules.len(), 1);
    assert_eq!(eval_result.result, serde_json::json!({"discount": 0.20}));

    // 5. Create a second rule and update rule set
    let rule2 = create_rule_uc
        .execute(&CreateRuleInput {
            name: "premium-discount".to_string(),
            description: "Premium discount".to_string(),
            priority: 2,
            when_condition: serde_json::json!({"field": "customer.tier", "operator": "eq", "value": "premium"}),
            then_result: serde_json::json!({"discount": 0.10}),
        })
        .await
        .unwrap();

    let update_rs_uc = UpdateRuleSetUseCase::with_publisher(rule_set_repo.clone(), publisher.clone());
    update_rs_uc
        .execute(&UpdateRuleSetInput {
            id: rule_set.id,
            description: None,
            evaluation_mode: None,
            default_result: None,
            rule_ids: Some(vec![rule.id, rule2.id]),
            enabled: None,
        })
        .await
        .unwrap();

    // 6. Publish (v2)
    let pub_result2 = publish_uc.execute(&rule_set.id).await.unwrap();
    assert_eq!(pub_result2.published_version, 2);

    // 7. Rollback to v1
    let rollback_uc = RollbackRuleSetUseCase::with_publisher(
        rule_set_repo.clone(),
        version_repo.clone(),
        publisher.clone(),
    );
    let rb_result = rollback_uc.execute(&rule_set.id).await.unwrap();
    assert_eq!(rb_result.rolled_back_to_version, 1);

    // 8. Verify rollback: rule set should have original rule_ids
    let get_rs_uc = GetRuleSetUseCase::new(rule_set_repo.clone());
    let current_rs = get_rs_uc.execute(&rule_set.id).await.unwrap().unwrap();
    assert_eq!(current_rs.rule_ids, vec![rule.id]);
    assert_eq!(current_rs.current_version, 1);
}
