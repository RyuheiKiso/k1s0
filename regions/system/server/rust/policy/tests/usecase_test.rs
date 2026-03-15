use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use k1s0_policy_server::domain::entity::policy::Policy;
use k1s0_policy_server::domain::entity::policy_bundle::PolicyBundle;
use k1s0_policy_server::domain::repository::{PolicyBundleRepository, PolicyRepository};
use k1s0_policy_server::usecase::create_bundle::{CreateBundleInput, CreateBundleUseCase};
use k1s0_policy_server::usecase::create_policy::{
    CreatePolicyError, CreatePolicyInput, CreatePolicyUseCase,
};
use k1s0_policy_server::usecase::delete_policy::{DeletePolicyError, DeletePolicyUseCase};
use k1s0_policy_server::usecase::evaluate_policy::{
    EvaluatePolicyError, EvaluatePolicyInput, EvaluatePolicyUseCase,
};
use k1s0_policy_server::usecase::get_bundle::{GetBundleError, GetBundleUseCase};
use k1s0_policy_server::usecase::get_policy::GetPolicyUseCase;
use k1s0_policy_server::usecase::list_bundles::ListBundlesUseCase;
use k1s0_policy_server::usecase::list_policies::{ListPoliciesInput, ListPoliciesUseCase};
use k1s0_policy_server::usecase::update_policy::{
    UpdatePolicyError, UpdatePolicyInput, UpdatePolicyUseCase,
};

// ---------------------------------------------------------------------------
// Stub: In-memory PolicyRepository
// ---------------------------------------------------------------------------

struct StubPolicyRepository {
    policies: RwLock<Vec<Policy>>,
    /// When set, all mutable operations return this error.
    force_error: Option<String>,
}

impl StubPolicyRepository {
    fn new() -> Self {
        Self {
            policies: RwLock::new(Vec::new()),
            force_error: None,
        }
    }

    fn with_policies(policies: Vec<Policy>) -> Self {
        Self {
            policies: RwLock::new(policies),
            force_error: None,
        }
    }

    fn with_error(msg: &str) -> Self {
        Self {
            policies: RwLock::new(Vec::new()),
            force_error: Some(msg.to_string()),
        }
    }
}

#[async_trait]
impl PolicyRepository for StubPolicyRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Policy>> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let policies = self.policies.read().await;
        Ok(policies.iter().find(|p| p.id == *id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Policy>> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        Ok(self.policies.read().await.clone())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        bundle_id: Option<Uuid>,
        enabled_only: bool,
    ) -> anyhow::Result<(Vec<Policy>, u64)> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let policies = self.policies.read().await;
        let filtered: Vec<Policy> = policies
            .iter()
            .filter(|p| {
                if enabled_only && !p.enabled {
                    return false;
                }
                if let Some(bid) = &bundle_id {
                    if p.bundle_id.as_ref() != Some(bid) {
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
        let page_items = if start < filtered.len() {
            filtered[start..end].to_vec()
        } else {
            vec![]
        };
        Ok((page_items, total))
    }

    async fn create(&self, policy: &Policy) -> anyhow::Result<()> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        self.policies.write().await.push(policy.clone());
        Ok(())
    }

    async fn update(&self, policy: &Policy) -> anyhow::Result<()> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let mut policies = self.policies.write().await;
        if let Some(existing) = policies.iter_mut().find(|p| p.id == policy.id) {
            *existing = policy.clone();
            Ok(())
        } else {
            Err(anyhow::anyhow!("policy not found"))
        }
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let mut policies = self.policies.write().await;
        let len_before = policies.len();
        policies.retain(|p| p.id != *id);
        Ok(policies.len() < len_before)
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let policies = self.policies.read().await;
        Ok(policies.iter().any(|p| p.name == name))
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory PolicyBundleRepository
// ---------------------------------------------------------------------------

struct StubBundleRepository {
    bundles: RwLock<Vec<PolicyBundle>>,
    force_error: Option<String>,
}

impl StubBundleRepository {
    fn new() -> Self {
        Self {
            bundles: RwLock::new(Vec::new()),
            force_error: None,
        }
    }

    fn with_bundles(bundles: Vec<PolicyBundle>) -> Self {
        Self {
            bundles: RwLock::new(bundles),
            force_error: None,
        }
    }

    fn with_error(msg: &str) -> Self {
        Self {
            bundles: RwLock::new(Vec::new()),
            force_error: Some(msg.to_string()),
        }
    }
}

#[async_trait]
impl PolicyBundleRepository for StubBundleRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<PolicyBundle>> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let bundles = self.bundles.read().await;
        Ok(bundles.iter().find(|b| b.id == *id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<PolicyBundle>> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        Ok(self.bundles.read().await.clone())
    }

    async fn create(&self, bundle: &PolicyBundle) -> anyhow::Result<()> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        self.bundles.write().await.push(bundle.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let mut bundles = self.bundles.write().await;
        let len_before = bundles.len();
        bundles.retain(|b| b.id != *id);
        Ok(bundles.len() < len_before)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_policy(name: &str, enabled: bool) -> Policy {
    let now = Utc::now();
    Policy {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: format!("{} description", name),
        rego_content: format!("package {}\ndefault allow = true", name),
        package_path: format!("k1s0.system.{}", name),
        bundle_id: None,
        version: 1,
        enabled,
        created_at: now,
        updated_at: now,
    }
}

fn make_policy_with_id(id: Uuid, name: &str, enabled: bool) -> Policy {
    let now = Utc::now();
    Policy {
        id,
        name: name.to_string(),
        description: format!("{} description", name),
        rego_content: format!("package {}\ndefault allow = true", name),
        package_path: format!("k1s0.system.{}", name),
        bundle_id: None,
        version: 1,
        enabled,
        created_at: now,
        updated_at: now,
    }
}

fn make_bundle(name: &str) -> PolicyBundle {
    PolicyBundle::new(
        name.to_string(),
        Some(format!("{} desc", name)),
        true,
        vec![],
    )
}

fn make_bundle_with_id(id: Uuid, name: &str) -> PolicyBundle {
    let now = Utc::now();
    PolicyBundle {
        id,
        name: name.to_string(),
        description: Some(format!("{} desc", name)),
        enabled: true,
        policy_ids: vec![],
        created_at: now,
        updated_at: now,
    }
}

// ===========================================================================
// CreatePolicy tests
// ===========================================================================

#[tokio::test]
async fn create_policy_success() {
    let repo = Arc::new(StubPolicyRepository::new());
    let uc = CreatePolicyUseCase::new(repo.clone());

    let input = CreatePolicyInput {
        name: "allow-read".to_string(),
        description: "Allow read access".to_string(),
        rego_content: "package authz\ndefault allow = true".to_string(),
        package_path: " k1s0.system.authz ".to_string(),
        bundle_id: Some(Uuid::new_v4()),
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.name, "allow-read");
    assert_eq!(result.package_path, "k1s0.system.authz"); // normalized (trimmed)
    assert!(result.bundle_id.is_some());
    assert_eq!(result.version, 1);
    assert!(result.enabled);

    // Verify persisted
    let stored = repo.policies.read().await;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].name, "allow-read");
}

#[tokio::test]
async fn create_policy_already_exists() {
    let existing = make_policy("existing-policy", true);
    let repo = Arc::new(StubPolicyRepository::with_policies(vec![existing]));
    let uc = CreatePolicyUseCase::new(repo);

    let input = CreatePolicyInput {
        name: "existing-policy".to_string(),
        description: "dup".to_string(),
        rego_content: "package authz".to_string(),
        package_path: "k1s0.system.authz".to_string(),
        bundle_id: None,
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreatePolicyError::AlreadyExists(name) => assert_eq!(name, "existing-policy"),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn create_policy_validation_empty_name() {
    let repo = Arc::new(StubPolicyRepository::new());
    let uc = CreatePolicyUseCase::new(repo);

    let input = CreatePolicyInput {
        name: "  ".to_string(),
        description: "desc".to_string(),
        rego_content: "package authz".to_string(),
        package_path: "path".to_string(),
        bundle_id: None,
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreatePolicyError::Validation(msg) => assert!(msg.contains("name")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn create_policy_validation_name_too_long() {
    let repo = Arc::new(StubPolicyRepository::new());
    let uc = CreatePolicyUseCase::new(repo);

    let input = CreatePolicyInput {
        name: "a".repeat(129),
        description: "desc".to_string(),
        rego_content: "package authz".to_string(),
        package_path: "path".to_string(),
        bundle_id: None,
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreatePolicyError::Validation(msg) => assert!(msg.contains("128")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn create_policy_validation_empty_rego() {
    let repo = Arc::new(StubPolicyRepository::new());
    let uc = CreatePolicyUseCase::new(repo);

    let input = CreatePolicyInput {
        name: "valid-name".to_string(),
        description: "desc".to_string(),
        rego_content: "  ".to_string(),
        package_path: "path".to_string(),
        bundle_id: None,
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreatePolicyError::Validation(msg) => assert!(msg.contains("rego_content")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn create_policy_validation_missing_package_declaration() {
    let repo = Arc::new(StubPolicyRepository::new());
    let uc = CreatePolicyUseCase::new(repo);

    let input = CreatePolicyInput {
        name: "valid-name".to_string(),
        description: "desc".to_string(),
        rego_content: "default allow = true".to_string(),
        package_path: "path".to_string(),
        bundle_id: None,
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreatePolicyError::Validation(msg) => assert!(msg.contains("package declaration")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn create_policy_repo_error() {
    let repo = Arc::new(StubPolicyRepository::with_error("connection refused"));
    let uc = CreatePolicyUseCase::new(repo);

    let input = CreatePolicyInput {
        name: "some-policy".to_string(),
        description: "desc".to_string(),
        rego_content: "package authz".to_string(),
        package_path: "path".to_string(),
        bundle_id: None,
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreatePolicyError::Internal(msg) => assert!(msg.contains("connection refused")),
        e => panic!("unexpected error: {:?}", e),
    }
}

// ===========================================================================
// GetPolicy tests
// ===========================================================================

#[tokio::test]
async fn get_policy_found() {
    let id = Uuid::new_v4();
    let policy = make_policy_with_id(id, "test-policy", true);
    let repo = Arc::new(StubPolicyRepository::with_policies(vec![policy]));
    let uc = GetPolicyUseCase::new(repo);

    let result = uc.execute(&id).await.unwrap();
    assert!(result.is_some());
    let p = result.unwrap();
    assert_eq!(p.id, id);
    assert_eq!(p.name, "test-policy");
}

#[tokio::test]
async fn get_policy_not_found() {
    let repo = Arc::new(StubPolicyRepository::new());
    let uc = GetPolicyUseCase::new(repo);

    let result = uc.execute(&Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn get_policy_repo_error() {
    let repo = Arc::new(StubPolicyRepository::with_error("db timeout"));
    let uc = GetPolicyUseCase::new(repo);

    let err = uc.execute(&Uuid::new_v4()).await.unwrap_err();
    assert!(err.to_string().contains("db timeout"));
}

// ===========================================================================
// ListPolicies tests
// ===========================================================================

#[tokio::test]
async fn list_policies_empty() {
    let repo = Arc::new(StubPolicyRepository::new());
    let uc = ListPoliciesUseCase::new(repo);

    let input = ListPoliciesInput {
        page: 1,
        page_size: 10,
        bundle_id: None,
        enabled_only: false,
    };
    let output = uc.execute(&input).await.unwrap();

    assert!(output.policies.is_empty());
    assert_eq!(output.total_count, 0);
    assert!(!output.has_next);
}

#[tokio::test]
async fn list_policies_with_results() {
    let p1 = make_policy("policy-a", true);
    let p2 = make_policy("policy-b", true);
    let p3 = make_policy("policy-c", false);
    let repo = Arc::new(StubPolicyRepository::with_policies(vec![p1, p2, p3]));
    let uc = ListPoliciesUseCase::new(repo);

    let input = ListPoliciesInput {
        page: 1,
        page_size: 10,
        bundle_id: None,
        enabled_only: false,
    };
    let output = uc.execute(&input).await.unwrap();

    assert_eq!(output.policies.len(), 3);
    assert_eq!(output.total_count, 3);
    assert!(!output.has_next);
}

#[tokio::test]
async fn list_policies_enabled_only() {
    let p1 = make_policy("enabled-1", true);
    let p2 = make_policy("disabled-1", false);
    let p3 = make_policy("enabled-2", true);
    let repo = Arc::new(StubPolicyRepository::with_policies(vec![p1, p2, p3]));
    let uc = ListPoliciesUseCase::new(repo);

    let input = ListPoliciesInput {
        page: 1,
        page_size: 10,
        bundle_id: None,
        enabled_only: true,
    };
    let output = uc.execute(&input).await.unwrap();

    assert_eq!(output.policies.len(), 2);
    assert_eq!(output.total_count, 2);
    assert!(output.policies.iter().all(|p| p.enabled));
}

#[tokio::test]
async fn list_policies_filter_by_bundle_id() {
    let bundle_id = Uuid::new_v4();
    let mut p1 = make_policy("in-bundle", true);
    p1.bundle_id = Some(bundle_id);
    let p2 = make_policy("no-bundle", true);
    let repo = Arc::new(StubPolicyRepository::with_policies(vec![p1, p2]));
    let uc = ListPoliciesUseCase::new(repo);

    let input = ListPoliciesInput {
        page: 1,
        page_size: 10,
        bundle_id: Some(bundle_id),
        enabled_only: false,
    };
    let output = uc.execute(&input).await.unwrap();

    assert_eq!(output.policies.len(), 1);
    assert_eq!(output.policies[0].name, "in-bundle");
}

#[tokio::test]
async fn list_policies_pagination_has_next() {
    let mut policies = Vec::new();
    for i in 0..5 {
        policies.push(make_policy(&format!("policy-{}", i), true));
    }
    let repo = Arc::new(StubPolicyRepository::with_policies(policies));
    let uc = ListPoliciesUseCase::new(repo);

    let input = ListPoliciesInput {
        page: 1,
        page_size: 3,
        bundle_id: None,
        enabled_only: false,
    };
    let output = uc.execute(&input).await.unwrap();

    assert_eq!(output.policies.len(), 3);
    assert_eq!(output.total_count, 5);
    assert!(output.has_next);
}

#[tokio::test]
async fn list_policies_pagination_last_page() {
    let mut policies = Vec::new();
    for i in 0..5 {
        policies.push(make_policy(&format!("policy-{}", i), true));
    }
    let repo = Arc::new(StubPolicyRepository::with_policies(policies));
    let uc = ListPoliciesUseCase::new(repo);

    let input = ListPoliciesInput {
        page: 2,
        page_size: 3,
        bundle_id: None,
        enabled_only: false,
    };
    let output = uc.execute(&input).await.unwrap();

    assert_eq!(output.policies.len(), 2);
    assert_eq!(output.total_count, 5);
    // page=2, page_size=3 => 2*3=6 >= 5 => no next
    assert!(!output.has_next);
}

#[tokio::test]
async fn list_policies_repo_error() {
    let repo = Arc::new(StubPolicyRepository::with_error("db error"));
    let uc = ListPoliciesUseCase::new(repo);

    let input = ListPoliciesInput {
        page: 1,
        page_size: 10,
        bundle_id: None,
        enabled_only: false,
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("db error"));
}

// ===========================================================================
// UpdatePolicy tests
// ===========================================================================

#[tokio::test]
async fn update_policy_success_partial() {
    let id = Uuid::new_v4();
    let policy = make_policy_with_id(id, "update-me", true);
    let repo = Arc::new(StubPolicyRepository::with_policies(vec![policy]));
    let uc = UpdatePolicyUseCase::new(repo.clone());

    let input = UpdatePolicyInput {
        id,
        description: Some("New description".to_string()),
        rego_content: None,
        enabled: Some(false),
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.description, "New description");
    assert!(!result.enabled);
    assert_eq!(result.version, 2);
    // rego_content should remain unchanged
    assert!(result.rego_content.contains("package"));

    // Verify persisted
    let stored = repo.policies.read().await;
    assert_eq!(stored[0].version, 2);
}

#[tokio::test]
async fn update_policy_success_all_fields() {
    let id = Uuid::new_v4();
    let policy = make_policy_with_id(id, "update-all", true);
    let repo = Arc::new(StubPolicyRepository::with_policies(vec![policy]));
    let uc = UpdatePolicyUseCase::new(repo);

    let input = UpdatePolicyInput {
        id,
        description: Some("Updated desc".to_string()),
        rego_content: Some("package updated\ndefault allow = false".to_string()),
        enabled: Some(false),
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.description, "Updated desc");
    assert_eq!(
        result.rego_content,
        "package updated\ndefault allow = false"
    );
    assert!(!result.enabled);
    assert_eq!(result.version, 2);
}

#[tokio::test]
async fn update_policy_no_changes() {
    let id = Uuid::new_v4();
    let policy = make_policy_with_id(id, "no-change", true);
    let repo = Arc::new(StubPolicyRepository::with_policies(vec![policy]));
    let uc = UpdatePolicyUseCase::new(repo);

    let input = UpdatePolicyInput {
        id,
        description: None,
        rego_content: None,
        enabled: None,
    };
    let result = uc.execute(&input).await.unwrap();

    // Version still increments per the usecase logic
    assert_eq!(result.version, 2);
    assert_eq!(result.name, "no-change");
}

#[tokio::test]
async fn update_policy_not_found() {
    let repo = Arc::new(StubPolicyRepository::new());
    let uc = UpdatePolicyUseCase::new(repo);
    let id = Uuid::new_v4();

    let input = UpdatePolicyInput {
        id,
        description: Some("nope".to_string()),
        rego_content: None,
        enabled: None,
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        UpdatePolicyError::NotFound(found_id) => assert_eq!(found_id, id),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn update_policy_repo_error() {
    let repo = Arc::new(StubPolicyRepository::with_error("update failed"));
    let uc = UpdatePolicyUseCase::new(repo);

    let input = UpdatePolicyInput {
        id: Uuid::new_v4(),
        description: None,
        rego_content: None,
        enabled: None,
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        UpdatePolicyError::Internal(msg) => assert!(msg.contains("update failed")),
        e => panic!("unexpected error: {:?}", e),
    }
}

// ===========================================================================
// DeletePolicy tests
// ===========================================================================

#[tokio::test]
async fn delete_policy_success() {
    let id = Uuid::new_v4();
    let policy = make_policy_with_id(id, "delete-me", true);
    let repo = Arc::new(StubPolicyRepository::with_policies(vec![policy]));
    let uc = DeletePolicyUseCase::new(repo.clone());

    uc.execute(&id).await.unwrap();

    let stored = repo.policies.read().await;
    assert!(stored.is_empty());
}

#[tokio::test]
async fn delete_policy_not_found() {
    let repo = Arc::new(StubPolicyRepository::new());
    let uc = DeletePolicyUseCase::new(repo);
    let id = Uuid::new_v4();

    let err = uc.execute(&id).await.unwrap_err();

    match err {
        DeletePolicyError::NotFound(found_id) => assert_eq!(found_id, id),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn delete_policy_repo_error() {
    let repo = Arc::new(StubPolicyRepository::with_error("disk full"));
    let uc = DeletePolicyUseCase::new(repo);

    let err = uc.execute(&Uuid::new_v4()).await.unwrap_err();

    match err {
        DeletePolicyError::Internal(msg) => assert!(msg.contains("disk full")),
        e => panic!("unexpected error: {:?}", e),
    }
}

// ===========================================================================
// EvaluatePolicy tests (fallback path, no OPA)
// ===========================================================================

#[tokio::test]
async fn evaluate_policy_enabled_allows() {
    let id = Uuid::new_v4();
    let policy = make_policy_with_id(id, "allow-all", true);
    let repo = Arc::new(StubPolicyRepository::with_policies(vec![policy]));
    let uc = EvaluatePolicyUseCase::new(repo, None);

    let input = EvaluatePolicyInput {
        policy_id: Some(id),
        package_path: String::new(),
        input: serde_json::json!({"action": "read"}),
    };
    let output = uc.execute(&input).await.unwrap();

    assert!(output.allowed);
    assert_eq!(output.reason.as_deref(), Some("policy is enabled"));
    assert!(!output.cached);
}

#[tokio::test]
async fn evaluate_policy_disabled_denies() {
    let id = Uuid::new_v4();
    let policy = make_policy_with_id(id, "deny-all", false);
    let repo = Arc::new(StubPolicyRepository::with_policies(vec![policy]));
    let uc = EvaluatePolicyUseCase::new(repo, None);

    let input = EvaluatePolicyInput {
        policy_id: Some(id),
        package_path: String::new(),
        input: serde_json::json!({"action": "write"}),
    };
    let output = uc.execute(&input).await.unwrap();

    assert!(!output.allowed);
    assert_eq!(output.reason.as_deref(), Some("policy is disabled"));
}

#[tokio::test]
async fn evaluate_policy_not_found() {
    let repo = Arc::new(StubPolicyRepository::new());
    let uc = EvaluatePolicyUseCase::new(repo, None);
    let id = Uuid::new_v4();

    let input = EvaluatePolicyInput {
        policy_id: Some(id),
        package_path: String::new(),
        input: serde_json::json!({}),
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        EvaluatePolicyError::NotFound(found_id) => assert_eq!(found_id, id),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn evaluate_policy_no_opa_no_policy_id() {
    let repo = Arc::new(StubPolicyRepository::new());
    let uc = EvaluatePolicyUseCase::new(repo, None);

    let input = EvaluatePolicyInput {
        policy_id: None,
        package_path: "some.path".to_string(),
        input: serde_json::json!({}),
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        EvaluatePolicyError::Internal(msg) => {
            assert!(msg.contains("no OPA client configured"));
        }
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn evaluate_policy_uses_stored_package_path() {
    let id = Uuid::new_v4();
    let mut policy = make_policy_with_id(id, "path-test", true);
    policy.package_path = "stored.package.path".to_string();
    let repo = Arc::new(StubPolicyRepository::with_policies(vec![policy]));
    let uc = EvaluatePolicyUseCase::new(repo, None);

    let input = EvaluatePolicyInput {
        policy_id: Some(id),
        package_path: "caller.provided.path".to_string(),
        input: serde_json::json!({}),
    };
    let output = uc.execute(&input).await.unwrap();

    // The usecase should use the stored package_path, not the caller-supplied one
    assert_eq!(output.package_path, "stored.package.path");
}

#[tokio::test]
async fn evaluate_policy_repo_error() {
    let repo = Arc::new(StubPolicyRepository::with_error("connection lost"));
    let uc = EvaluatePolicyUseCase::new(repo, None);

    let input = EvaluatePolicyInput {
        policy_id: Some(Uuid::new_v4()),
        package_path: String::new(),
        input: serde_json::json!({}),
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        EvaluatePolicyError::Internal(msg) => assert!(msg.contains("connection lost")),
        e => panic!("unexpected error: {:?}", e),
    }
}

// ===========================================================================
// CreateBundle tests
// ===========================================================================

#[tokio::test]
async fn create_bundle_success() {
    let repo = Arc::new(StubBundleRepository::new());
    let uc = CreateBundleUseCase::new(repo.clone());

    let policy_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
    let input = CreateBundleInput {
        name: "security-bundle".to_string(),
        description: Some("Security policies".to_string()),
        enabled: Some(true),
        policy_ids: policy_ids.clone(),
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.name, "security-bundle");
    assert_eq!(result.description.as_deref(), Some("Security policies"));
    assert!(result.enabled);
    assert_eq!(result.policy_ids.len(), 2);

    // Verify persisted
    let stored = repo.bundles.read().await;
    assert_eq!(stored.len(), 1);
}

#[tokio::test]
async fn create_bundle_default_enabled() {
    let repo = Arc::new(StubBundleRepository::new());
    let uc = CreateBundleUseCase::new(repo);

    let input = CreateBundleInput {
        name: "default-enabled".to_string(),
        description: None,
        enabled: None, // should default to true
        policy_ids: vec![],
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
}

#[tokio::test]
async fn create_bundle_disabled() {
    let repo = Arc::new(StubBundleRepository::new());
    let uc = CreateBundleUseCase::new(repo);

    let input = CreateBundleInput {
        name: "disabled-bundle".to_string(),
        description: None,
        enabled: Some(false),
        policy_ids: vec![],
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(!result.enabled);
}

#[tokio::test]
async fn create_bundle_repo_error() {
    let repo = Arc::new(StubBundleRepository::with_error("insert failed"));
    let uc = CreateBundleUseCase::new(repo);

    let input = CreateBundleInput {
        name: "fail-bundle".to_string(),
        description: None,
        enabled: None,
        policy_ids: vec![],
    };
    let err = uc.execute(&input).await.unwrap_err();
    assert!(err.to_string().contains("insert failed"));
}

// ===========================================================================
// GetBundle tests
// ===========================================================================

#[tokio::test]
async fn get_bundle_found() {
    let id = Uuid::new_v4();
    let bundle = make_bundle_with_id(id, "my-bundle");
    let repo = Arc::new(StubBundleRepository::with_bundles(vec![bundle]));
    let uc = GetBundleUseCase::new(repo);

    let result = uc.execute(&id).await.unwrap();
    assert_eq!(result.id, id);
    assert_eq!(result.name, "my-bundle");
}

#[tokio::test]
async fn get_bundle_not_found() {
    let repo = Arc::new(StubBundleRepository::new());
    let uc = GetBundleUseCase::new(repo);

    let err = uc.execute(&Uuid::new_v4()).await.unwrap_err();
    assert!(matches!(err, GetBundleError::NotFound(_)));
}

#[tokio::test]
async fn get_bundle_repo_error() {
    let repo = Arc::new(StubBundleRepository::with_error("query timeout"));
    let uc = GetBundleUseCase::new(repo);

    let err = uc.execute(&Uuid::new_v4()).await.unwrap_err();
    match err {
        GetBundleError::Internal(msg) => assert!(msg.contains("query timeout")),
        e => panic!("unexpected error: {:?}", e),
    }
}

// ===========================================================================
// ListBundles tests
// ===========================================================================

#[tokio::test]
async fn list_bundles_empty() {
    let repo = Arc::new(StubBundleRepository::new());
    let uc = ListBundlesUseCase::new(repo);

    let result = uc.execute().await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn list_bundles_with_results() {
    let b1 = make_bundle("bundle-a");
    let b2 = make_bundle("bundle-b");
    let repo = Arc::new(StubBundleRepository::with_bundles(vec![b1, b2]));
    let uc = ListBundlesUseCase::new(repo);

    let result = uc.execute().await.unwrap();
    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn list_bundles_repo_error() {
    let repo = Arc::new(StubBundleRepository::with_error("network error"));
    let uc = ListBundlesUseCase::new(repo);

    let err = uc.execute().await.unwrap_err();
    assert!(err.to_string().contains("network error"));
}

// ===========================================================================
// End-to-end workflow: create -> get -> update -> list -> delete -> verify
// ===========================================================================

#[tokio::test]
async fn policy_crud_workflow() {
    let repo = Arc::new(StubPolicyRepository::new());

    // 1. Create
    let create_uc = CreatePolicyUseCase::new(repo.clone());
    let create_input = CreatePolicyInput {
        name: "workflow-policy".to_string(),
        description: "Workflow test".to_string(),
        rego_content: "package workflow\ndefault allow = true".to_string(),
        package_path: "k1s0.system.workflow".to_string(),
        bundle_id: None,
    };
    let created = create_uc.execute(&create_input).await.unwrap();
    let policy_id = created.id;

    // 2. Get
    let get_uc = GetPolicyUseCase::new(repo.clone());
    let fetched = get_uc.execute(&policy_id).await.unwrap().unwrap();
    assert_eq!(fetched.name, "workflow-policy");
    assert_eq!(fetched.version, 1);

    // 3. Update
    let update_uc = UpdatePolicyUseCase::new(repo.clone());
    let update_input = UpdatePolicyInput {
        id: policy_id,
        description: Some("Updated workflow".to_string()),
        rego_content: None,
        enabled: Some(false),
    };
    let updated = update_uc.execute(&update_input).await.unwrap();
    assert_eq!(updated.version, 2);
    assert!(!updated.enabled);

    // 4. List
    let list_uc = ListPoliciesUseCase::new(repo.clone());
    let list_input = ListPoliciesInput {
        page: 1,
        page_size: 10,
        bundle_id: None,
        enabled_only: false,
    };
    let list_output = list_uc.execute(&list_input).await.unwrap();
    assert_eq!(list_output.total_count, 1);

    // 5. Evaluate (disabled => denied)
    let eval_uc = EvaluatePolicyUseCase::new(repo.clone(), None);
    let eval_input = EvaluatePolicyInput {
        policy_id: Some(policy_id),
        package_path: String::new(),
        input: serde_json::json!({"action": "read"}),
    };
    let eval_output = eval_uc.execute(&eval_input).await.unwrap();
    assert!(!eval_output.allowed);

    // 6. Delete
    let delete_uc = DeletePolicyUseCase::new(repo.clone());
    delete_uc.execute(&policy_id).await.unwrap();

    // 7. Verify deleted
    let after_delete = get_uc.execute(&policy_id).await.unwrap();
    assert!(after_delete.is_none());
}

#[tokio::test]
async fn bundle_crud_workflow() {
    let repo = Arc::new(StubBundleRepository::new());

    // 1. Create
    let create_uc = CreateBundleUseCase::new(repo.clone());
    let input = CreateBundleInput {
        name: "workflow-bundle".to_string(),
        description: Some("Test bundle".to_string()),
        enabled: Some(true),
        policy_ids: vec![Uuid::new_v4()],
    };
    let created = create_uc.execute(&input).await.unwrap();
    let bundle_id = created.id;

    // 2. Get
    let get_uc = GetBundleUseCase::new(repo.clone());
    let fetched = get_uc.execute(&bundle_id).await.unwrap();
    assert_eq!(fetched.name, "workflow-bundle");

    // 3. List
    let list_uc = ListBundlesUseCase::new(repo.clone());
    let bundles = list_uc.execute().await.unwrap();
    assert_eq!(bundles.len(), 1);
}
