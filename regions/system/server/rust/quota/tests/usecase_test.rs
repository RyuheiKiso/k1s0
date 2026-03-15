use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use k1s0_quota_server::domain::entity::quota::{Period, QuotaPolicy, SubjectType};
use k1s0_quota_server::domain::repository::{
    CheckAndIncrementResult, QuotaPolicyRepository, QuotaUsageRepository,
};
use k1s0_quota_server::infrastructure::kafka_producer::{
    QuotaEventPublisher, QuotaExceededEvent, QuotaThresholdReachedEvent,
};
use k1s0_quota_server::usecase::create_quota_policy::{
    CreateQuotaPolicyError, CreateQuotaPolicyInput, CreateQuotaPolicyUseCase,
};
use k1s0_quota_server::usecase::delete_quota_policy::{
    DeleteQuotaPolicyError, DeleteQuotaPolicyUseCase,
};
use k1s0_quota_server::usecase::get_quota_policy::{GetQuotaPolicyError, GetQuotaPolicyUseCase};
use k1s0_quota_server::usecase::get_quota_usage::{GetQuotaUsageError, GetQuotaUsageUseCase};
use k1s0_quota_server::usecase::increment_quota_usage::{
    IncrementQuotaUsageError, IncrementQuotaUsageInput, IncrementQuotaUsageUseCase,
};
use k1s0_quota_server::usecase::list_quota_policies::{
    ListQuotaPoliciesError, ListQuotaPoliciesInput, ListQuotaPoliciesUseCase,
};
use k1s0_quota_server::usecase::reset_quota_usage::{
    ResetQuotaUsageError, ResetQuotaUsageInput, ResetQuotaUsageUseCase,
};
use k1s0_quota_server::usecase::update_quota_policy::{
    UpdateQuotaPolicyError, UpdateQuotaPolicyInput, UpdateQuotaPolicyUseCase,
};

// ---------------------------------------------------------------------------
// Stub: In-memory QuotaPolicyRepository
// ---------------------------------------------------------------------------

struct StubPolicyRepository {
    policies: RwLock<HashMap<String, QuotaPolicy>>,
    should_fail: bool,
}

impl StubPolicyRepository {
    fn new() -> Self {
        Self {
            policies: RwLock::new(HashMap::new()),
            should_fail: false,
        }
    }

    fn with_error() -> Self {
        Self {
            policies: RwLock::new(HashMap::new()),
            should_fail: true,
        }
    }

    async fn with_policy(policy: &QuotaPolicy) -> Self {
        let mut map = HashMap::new();
        map.insert(policy.id.clone(), policy.clone());
        Self {
            policies: RwLock::new(map),
            should_fail: false,
        }
    }

    async fn with_policies(policies: &[QuotaPolicy]) -> Self {
        let mut map = HashMap::new();
        for p in policies {
            map.insert(p.id.clone(), p.clone());
        }
        Self {
            policies: RwLock::new(map),
            should_fail: false,
        }
    }
}

#[async_trait]
impl QuotaPolicyRepository for StubPolicyRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<QuotaPolicy>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("db connection error"));
        }
        let policies = self.policies.read().await;
        Ok(policies.get(id).cloned())
    }

    async fn find_all(&self, page: u32, page_size: u32) -> anyhow::Result<(Vec<QuotaPolicy>, u64)> {
        if self.should_fail {
            return Err(anyhow::anyhow!("db connection error"));
        }
        let policies = self.policies.read().await;
        let all: Vec<QuotaPolicy> = policies.values().cloned().collect();
        let total = all.len() as u64;
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let items: Vec<QuotaPolicy> = all
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((items, total))
    }

    async fn create(&self, policy: &QuotaPolicy) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("db connection error"));
        }
        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy.clone());
        Ok(())
    }

    async fn update(&self, policy: &QuotaPolicy) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("db connection error"));
        }
        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        if self.should_fail {
            return Err(anyhow::anyhow!("db connection error"));
        }
        let mut policies = self.policies.write().await;
        Ok(policies.remove(id).is_some())
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory QuotaUsageRepository
// ---------------------------------------------------------------------------

struct StubUsageRepository {
    usages: RwLock<HashMap<String, u64>>,
    should_fail: bool,
}

impl StubUsageRepository {
    fn new() -> Self {
        Self {
            usages: RwLock::new(HashMap::new()),
            should_fail: false,
        }
    }

    fn with_error() -> Self {
        Self {
            usages: RwLock::new(HashMap::new()),
            should_fail: true,
        }
    }

    async fn with_usage(quota_id: &str, used: u64) -> Self {
        let mut map = HashMap::new();
        map.insert(quota_id.to_string(), used);
        Self {
            usages: RwLock::new(map),
            should_fail: false,
        }
    }
}

#[async_trait]
impl QuotaUsageRepository for StubUsageRepository {
    async fn get_usage(&self, quota_id: &str) -> anyhow::Result<Option<u64>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("redis connection error"));
        }
        let usages = self.usages.read().await;
        Ok(usages.get(quota_id).copied())
    }

    async fn increment(&self, quota_id: &str, amount: u64) -> anyhow::Result<u64> {
        if self.should_fail {
            return Err(anyhow::anyhow!("redis connection error"));
        }
        let mut usages = self.usages.write().await;
        let entry = usages.entry(quota_id.to_string()).or_insert(0);
        *entry += amount;
        Ok(*entry)
    }

    async fn reset(&self, quota_id: &str) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("redis connection error"));
        }
        let mut usages = self.usages.write().await;
        usages.insert(quota_id.to_string(), 0);
        Ok(())
    }

    async fn check_and_increment(
        &self,
        quota_id: &str,
        amount: u64,
        limit: u64,
    ) -> anyhow::Result<CheckAndIncrementResult> {
        if self.should_fail {
            return Err(anyhow::anyhow!("redis connection error"));
        }
        let mut usages = self.usages.write().await;
        let current = usages.entry(quota_id.to_string()).or_insert(0);
        if *current + amount > limit {
            Ok(CheckAndIncrementResult {
                used: *current,
                allowed: false,
            })
        } else {
            *current += amount;
            Ok(CheckAndIncrementResult {
                used: *current,
                allowed: true,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory QuotaEventPublisher
// ---------------------------------------------------------------------------

struct StubEventPublisher {
    exceeded_events: RwLock<Vec<QuotaExceededEvent>>,
    threshold_events: RwLock<Vec<QuotaThresholdReachedEvent>>,
}

impl StubEventPublisher {
    fn new() -> Self {
        Self {
            exceeded_events: RwLock::new(Vec::new()),
            threshold_events: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl QuotaEventPublisher for StubEventPublisher {
    async fn publish_quota_exceeded(&self, event: &QuotaExceededEvent) -> anyhow::Result<()> {
        self.exceeded_events.write().await.push(event.clone());
        Ok(())
    }

    async fn publish_threshold_reached(
        &self,
        event: &QuotaThresholdReachedEvent,
    ) -> anyhow::Result<()> {
        self.threshold_events.write().await.push(event.clone());
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sample_policy() -> QuotaPolicy {
    QuotaPolicy::new(
        "Standard Plan".to_string(),
        SubjectType::Tenant,
        "tenant-abc".to_string(),
        10000,
        Period::Daily,
        true,
        Some(80),
    )
}

fn sample_policy_monthly() -> QuotaPolicy {
    QuotaPolicy::new(
        "Monthly Plan".to_string(),
        SubjectType::User,
        "user-1".to_string(),
        5000,
        Period::Monthly,
        true,
        None,
    )
}

fn sample_disabled_policy() -> QuotaPolicy {
    QuotaPolicy::new(
        "Disabled Plan".to_string(),
        SubjectType::ApiKey,
        "key-1".to_string(),
        1000,
        Period::Daily,
        false,
        None,
    )
}

// ===========================================================================
// CreateQuotaPolicyUseCase
// ===========================================================================

mod create_quota_policy {
    use super::*;

    #[tokio::test]
    async fn success_creates_policy_with_correct_fields() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = CreateQuotaPolicyUseCase::new(repo.clone());

        let input = CreateQuotaPolicyInput {
            name: "Standard Plan".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "tenant-abc".to_string(),
            limit: 10000,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: Some(80),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let policy = result.unwrap();
        assert!(policy.id.starts_with("quota_"));
        assert_eq!(policy.name, "Standard Plan");
        assert_eq!(policy.subject_type, SubjectType::Tenant);
        assert_eq!(policy.subject_id, "tenant-abc");
        assert_eq!(policy.limit, 10000);
        assert_eq!(policy.period, Period::Daily);
        assert!(policy.enabled);
        assert_eq!(policy.alert_threshold_percent, Some(80));

        // Verify persisted in repository
        let stored = repo.policies.read().await;
        assert!(stored.contains_key(&policy.id));
    }

    #[tokio::test]
    async fn success_with_user_subject_type() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = CreateQuotaPolicyUseCase::new(repo);

        let input = CreateQuotaPolicyInput {
            name: "User Quota".to_string(),
            subject_type: "user".to_string(),
            subject_id: "user-123".to_string(),
            limit: 500,
            period: "monthly".to_string(),
            enabled: false,
            alert_threshold_percent: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let policy = result.unwrap();
        assert_eq!(policy.subject_type, SubjectType::User);
        assert_eq!(policy.period, Period::Monthly);
        assert!(!policy.enabled);
        assert_eq!(policy.alert_threshold_percent, None);
    }

    #[tokio::test]
    async fn success_with_api_key_subject_type() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = CreateQuotaPolicyUseCase::new(repo);

        let input = CreateQuotaPolicyInput {
            name: "API Key Quota".to_string(),
            subject_type: "api_key".to_string(),
            subject_id: "key-xyz".to_string(),
            limit: 100,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: Some(90),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().subject_type, SubjectType::ApiKey);
    }

    #[tokio::test]
    async fn error_invalid_subject_type() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = CreateQuotaPolicyUseCase::new(repo);

        let input = CreateQuotaPolicyInput {
            name: "test".to_string(),
            subject_type: "organization".to_string(),
            subject_id: "id".to_string(),
            limit: 100,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateQuotaPolicyError::Validation(msg) => assert!(msg.contains("subject_type")),
            e => panic!("expected Validation error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_invalid_period() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = CreateQuotaPolicyUseCase::new(repo);

        let input = CreateQuotaPolicyInput {
            name: "test".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "id".to_string(),
            limit: 100,
            period: "weekly".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateQuotaPolicyError::Validation(msg) => assert!(msg.contains("period")),
            e => panic!("expected Validation error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_zero_limit() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = CreateQuotaPolicyUseCase::new(repo);

        let input = CreateQuotaPolicyInput {
            name: "test".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "id".to_string(),
            limit: 0,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateQuotaPolicyError::Validation(msg) => assert!(msg.contains("limit")),
            e => panic!("expected Validation error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_alert_threshold_over_100() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = CreateQuotaPolicyUseCase::new(repo);

        let input = CreateQuotaPolicyInput {
            name: "test".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "id".to_string(),
            limit: 100,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: Some(101),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateQuotaPolicyError::Validation(msg) => {
                assert!(msg.contains("alert_threshold_percent"))
            }
            e => panic!("expected Validation error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_repository_failure() {
        let repo = Arc::new(StubPolicyRepository::with_error());
        let uc = CreateQuotaPolicyUseCase::new(repo);

        let input = CreateQuotaPolicyInput {
            name: "test".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "id".to_string(),
            limit: 100,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateQuotaPolicyError::Internal(msg) => assert!(msg.contains("db connection error")),
            e => panic!("expected Internal error, got: {:?}", e),
        }
    }
}

// ===========================================================================
// GetQuotaPolicyUseCase
// ===========================================================================

mod get_quota_policy {
    use super::*;

    #[tokio::test]
    async fn success_returns_existing_policy() {
        let policy = sample_policy();
        let repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let uc = GetQuotaPolicyUseCase::new(repo);

        let result = uc.execute(&policy.id).await;
        assert!(result.is_ok());

        let found = result.unwrap();
        assert_eq!(found.id, policy.id);
        assert_eq!(found.name, "Standard Plan");
        assert_eq!(found.subject_type, SubjectType::Tenant);
    }

    #[tokio::test]
    async fn error_not_found() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = GetQuotaPolicyUseCase::new(repo);

        let result = uc.execute("nonexistent-id").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetQuotaPolicyError::NotFound(id) => assert_eq!(id, "nonexistent-id"),
            e => panic!("expected NotFound error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_repository_failure() {
        let repo = Arc::new(StubPolicyRepository::with_error());
        let uc = GetQuotaPolicyUseCase::new(repo);

        let result = uc.execute("some-id").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetQuotaPolicyError::Internal(msg) => assert!(msg.contains("db connection error")),
            e => panic!("expected Internal error, got: {:?}", e),
        }
    }
}

// ===========================================================================
// UpdateQuotaPolicyUseCase
// ===========================================================================

mod update_quota_policy {
    use super::*;

    #[tokio::test]
    async fn success_updates_all_fields() {
        let policy = sample_policy();
        let policy_id = policy.id.clone();
        let repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let uc = UpdateQuotaPolicyUseCase::new(repo.clone());

        let input = UpdateQuotaPolicyInput {
            id: policy_id.clone(),
            name: "Updated Plan".to_string(),
            subject_type: "user".to_string(),
            subject_id: "user-999".to_string(),
            limit: 20000,
            period: "monthly".to_string(),
            enabled: false,
            alert_threshold_percent: Some(95),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.id, policy_id);
        assert_eq!(updated.name, "Updated Plan");
        assert_eq!(updated.subject_type, SubjectType::User);
        assert_eq!(updated.subject_id, "user-999");
        assert_eq!(updated.limit, 20000);
        assert_eq!(updated.period, Period::Monthly);
        assert!(!updated.enabled);
        assert_eq!(updated.alert_threshold_percent, Some(95));

        // Verify persisted
        let stored = repo.policies.read().await;
        let persisted = stored.get(&policy_id).unwrap();
        assert_eq!(persisted.name, "Updated Plan");
    }

    #[tokio::test]
    async fn error_not_found() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = UpdateQuotaPolicyUseCase::new(repo);

        let input = UpdateQuotaPolicyInput {
            id: "nonexistent".to_string(),
            name: "test".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "id".to_string(),
            limit: 100,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UpdateQuotaPolicyError::NotFound(id) => assert_eq!(id, "nonexistent"),
            e => panic!("expected NotFound error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_invalid_subject_type() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = UpdateQuotaPolicyUseCase::new(repo);

        let input = UpdateQuotaPolicyInput {
            id: "some-id".to_string(),
            name: "test".to_string(),
            subject_type: "invalid".to_string(),
            subject_id: "id".to_string(),
            limit: 100,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UpdateQuotaPolicyError::Validation(msg) => assert!(msg.contains("subject_type")),
            e => panic!("expected Validation error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_invalid_period() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = UpdateQuotaPolicyUseCase::new(repo);

        let input = UpdateQuotaPolicyInput {
            id: "some-id".to_string(),
            name: "test".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "id".to_string(),
            limit: 100,
            period: "yearly".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UpdateQuotaPolicyError::Validation(msg) => assert!(msg.contains("period")),
            e => panic!("expected Validation error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_zero_limit() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = UpdateQuotaPolicyUseCase::new(repo);

        let input = UpdateQuotaPolicyInput {
            id: "some-id".to_string(),
            name: "test".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "id".to_string(),
            limit: 0,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UpdateQuotaPolicyError::Validation(msg) => assert!(msg.contains("limit")),
            e => panic!("expected Validation error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_repository_failure_on_find() {
        let repo = Arc::new(StubPolicyRepository::with_error());
        let uc = UpdateQuotaPolicyUseCase::new(repo);

        let input = UpdateQuotaPolicyInput {
            id: "some-id".to_string(),
            name: "test".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "id".to_string(),
            limit: 100,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UpdateQuotaPolicyError::Internal(msg) => assert!(msg.contains("db connection error")),
            e => panic!("expected Internal error, got: {:?}", e),
        }
    }
}

// ===========================================================================
// DeleteQuotaPolicyUseCase
// ===========================================================================

mod delete_quota_policy {
    use super::*;

    #[tokio::test]
    async fn success_deletes_existing_policy() {
        let policy = sample_policy();
        let policy_id = policy.id.clone();
        let repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let uc = DeleteQuotaPolicyUseCase::new(repo.clone());

        let result = uc.execute(&policy_id).await;
        assert!(result.is_ok());

        // Verify removed from repository
        let stored = repo.policies.read().await;
        assert!(!stored.contains_key(&policy_id));
    }

    #[tokio::test]
    async fn error_not_found() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = DeleteQuotaPolicyUseCase::new(repo);

        let result = uc.execute("nonexistent-id").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteQuotaPolicyError::NotFound(id) => assert_eq!(id, "nonexistent-id"),
            e => panic!("expected NotFound error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_repository_failure() {
        let repo = Arc::new(StubPolicyRepository::with_error());
        let uc = DeleteQuotaPolicyUseCase::new(repo);

        let result = uc.execute("some-id").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteQuotaPolicyError::Internal(msg) => assert!(msg.contains("db connection error")),
            e => panic!("expected Internal error, got: {:?}", e),
        }
    }
}

// ===========================================================================
// ListQuotaPoliciesUseCase
// ===========================================================================

mod list_quota_policies {
    use super::*;

    #[tokio::test]
    async fn success_returns_all_policies() {
        let p1 = sample_policy();
        let p2 = sample_policy_monthly();
        let repo = Arc::new(StubPolicyRepository::with_policies(&[p1, p2]).await);
        let uc = ListQuotaPoliciesUseCase::new(repo);

        let input = ListQuotaPoliciesInput {
            page: 1,
            page_size: 20,
            subject_type: None,
            subject_id: None,
            enabled_only: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.quotas.len(), 2);
        assert_eq!(output.total_count, 2);
        assert!(!output.has_next);
        assert_eq!(output.page, 1);
        assert_eq!(output.page_size, 20);
    }

    #[tokio::test]
    async fn success_empty_list() {
        let repo = Arc::new(StubPolicyRepository::new());
        let uc = ListQuotaPoliciesUseCase::new(repo);

        let input = ListQuotaPoliciesInput {
            page: 1,
            page_size: 20,
            subject_type: None,
            subject_id: None,
            enabled_only: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.quotas.len(), 0);
        assert_eq!(output.total_count, 0);
    }

    #[tokio::test]
    async fn success_filtered_by_subject_type() {
        let p1 = sample_policy(); // Tenant
        let p2 = sample_policy_monthly(); // User
        let repo = Arc::new(StubPolicyRepository::with_policies(&[p1, p2]).await);
        let uc = ListQuotaPoliciesUseCase::new(repo);

        let input = ListQuotaPoliciesInput {
            page: 1,
            page_size: 20,
            subject_type: Some("tenant".to_string()),
            subject_id: None,
            enabled_only: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.total_count, 1);
        assert_eq!(output.quotas[0].subject_type, SubjectType::Tenant);
    }

    #[tokio::test]
    async fn success_filtered_by_subject_id() {
        let p1 = sample_policy(); // subject_id: tenant-abc
        let p2 = sample_policy_monthly(); // subject_id: user-1
        let repo = Arc::new(StubPolicyRepository::with_policies(&[p1, p2]).await);
        let uc = ListQuotaPoliciesUseCase::new(repo);

        let input = ListQuotaPoliciesInput {
            page: 1,
            page_size: 20,
            subject_type: None,
            subject_id: Some("user-1".to_string()),
            enabled_only: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.total_count, 1);
        assert_eq!(output.quotas[0].subject_id, "user-1");
    }

    #[tokio::test]
    async fn success_filtered_by_enabled_only() {
        let p1 = sample_policy(); // enabled: true
        let p2 = sample_disabled_policy(); // enabled: false
        let repo = Arc::new(StubPolicyRepository::with_policies(&[p1, p2]).await);
        let uc = ListQuotaPoliciesUseCase::new(repo);

        let input = ListQuotaPoliciesInput {
            page: 1,
            page_size: 20,
            subject_type: None,
            subject_id: None,
            enabled_only: Some(true),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.total_count, 1);
        assert!(output.quotas[0].enabled);
    }

    #[tokio::test]
    async fn error_repository_failure() {
        let repo = Arc::new(StubPolicyRepository::with_error());
        let uc = ListQuotaPoliciesUseCase::new(repo);

        let input = ListQuotaPoliciesInput {
            page: 1,
            page_size: 20,
            subject_type: None,
            subject_id: None,
            enabled_only: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ListQuotaPoliciesError::Internal(msg) => assert!(msg.contains("db connection error")),
        }
    }
}

// ===========================================================================
// GetQuotaUsageUseCase
// ===========================================================================

mod get_quota_usage {
    use super::*;

    #[tokio::test]
    async fn success_returns_usage_under_limit() {
        let policy = sample_policy(); // limit: 10000
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_usage(&policy.id, 7500).await);
        let uc = GetQuotaUsageUseCase::new(policy_repo, usage_repo);

        let result = uc.execute(&policy.id).await;
        assert!(result.is_ok());

        let usage = result.unwrap();
        assert_eq!(usage.used, 7500);
        assert_eq!(usage.remaining, 2500);
        assert!(!usage.exceeded);
        assert!((usage.usage_percent - 75.0).abs() < f64::EPSILON);
        assert_eq!(usage.quota_id, policy.id);
    }

    #[tokio::test]
    async fn success_zero_usage_when_no_data() {
        let policy = sample_policy(); // limit: 10000
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::new()); // no usage data
        let uc = GetQuotaUsageUseCase::new(policy_repo, usage_repo);

        let result = uc.execute(&policy.id).await;
        assert!(result.is_ok());

        let usage = result.unwrap();
        assert_eq!(usage.used, 0);
        assert_eq!(usage.remaining, 10000);
        assert!(!usage.exceeded);
    }

    #[tokio::test]
    async fn success_at_limit() {
        let policy = sample_policy(); // limit: 10000
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_usage(&policy.id, 10000).await);
        let uc = GetQuotaUsageUseCase::new(policy_repo, usage_repo);

        let result = uc.execute(&policy.id).await;
        assert!(result.is_ok());

        let usage = result.unwrap();
        assert_eq!(usage.used, 10000);
        assert_eq!(usage.remaining, 0);
        assert!(usage.exceeded);
    }

    #[tokio::test]
    async fn success_monthly_policy() {
        let policy = sample_policy_monthly(); // Monthly, limit: 5000
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_usage(&policy.id, 2000).await);
        let uc = GetQuotaUsageUseCase::new(policy_repo, usage_repo);

        let result = uc.execute(&policy.id).await;
        assert!(result.is_ok());

        let usage = result.unwrap();
        assert_eq!(usage.used, 2000);
        assert_eq!(usage.remaining, 3000);
        assert_eq!(usage.period, Period::Monthly);
    }

    #[tokio::test]
    async fn error_policy_not_found() {
        let policy_repo = Arc::new(StubPolicyRepository::new());
        let usage_repo = Arc::new(StubUsageRepository::new());
        let uc = GetQuotaUsageUseCase::new(policy_repo, usage_repo);

        let result = uc.execute("nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetQuotaUsageError::NotFound(id) => assert_eq!(id, "nonexistent"),
            e => panic!("expected NotFound error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_policy_repo_failure() {
        let policy_repo = Arc::new(StubPolicyRepository::with_error());
        let usage_repo = Arc::new(StubUsageRepository::new());
        let uc = GetQuotaUsageUseCase::new(policy_repo, usage_repo);

        let result = uc.execute("some-id").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetQuotaUsageError::Internal(msg) => assert!(msg.contains("db connection error")),
            e => panic!("expected Internal error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_usage_repo_failure() {
        let policy = sample_policy();
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_error());
        let uc = GetQuotaUsageUseCase::new(policy_repo, usage_repo);

        let result = uc.execute(&policy.id).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetQuotaUsageError::Internal(msg) => assert!(msg.contains("redis connection error")),
            e => panic!("expected Internal error, got: {:?}", e),
        }
    }
}

// ===========================================================================
// IncrementQuotaUsageUseCase
// ===========================================================================

mod increment_quota_usage {
    use super::*;

    #[tokio::test]
    async fn success_increments_within_limit() {
        let policy = sample_policy(); // limit: 10000, threshold: 80%
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_usage(&policy.id, 5000).await);
        let event_pub = Arc::new(StubEventPublisher::new());
        let uc = IncrementQuotaUsageUseCase::new(policy_repo, usage_repo, event_pub.clone());

        let input = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 100,
            request_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let inc = result.unwrap();
        assert_eq!(inc.used, 5100);
        assert_eq!(inc.remaining, 4900);
        assert!(!inc.exceeded);
        assert!(inc.allowed);

        // No events should have been published
        assert!(event_pub.exceeded_events.read().await.is_empty());
    }

    #[tokio::test]
    async fn success_increment_from_zero() {
        let policy = sample_policy(); // limit: 10000
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::new()); // starts at 0
        let event_pub = Arc::new(StubEventPublisher::new());
        let uc = IncrementQuotaUsageUseCase::new(policy_repo, usage_repo, event_pub);

        let input = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 1,
            request_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let inc = result.unwrap();
        assert_eq!(inc.used, 1);
        assert_eq!(inc.remaining, 9999);
        assert!(inc.allowed);
    }

    #[tokio::test]
    async fn error_quota_exceeded() {
        let policy = sample_policy(); // limit: 10000
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_usage(&policy.id, 10000).await);
        let event_pub = Arc::new(StubEventPublisher::new());
        let uc = IncrementQuotaUsageUseCase::new(policy_repo, usage_repo, event_pub.clone());

        let input = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 1,
            request_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            IncrementQuotaUsageError::Exceeded {
                used,
                limit,
                quota_id,
                ..
            } => {
                assert_eq!(used, 10000);
                assert_eq!(limit, 10000);
                assert_eq!(quota_id, policy.id);
            }
            e => panic!("expected Exceeded error, got: {:?}", e),
        }

        // Exceeded event should have been published
        let exceeded = event_pub.exceeded_events.read().await;
        assert_eq!(exceeded.len(), 1);
        assert_eq!(exceeded[0].event_type, "QUOTA_EXCEEDED");
    }

    #[tokio::test]
    async fn error_exceeds_limit_with_large_amount() {
        let policy = sample_policy(); // limit: 10000
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_usage(&policy.id, 9990).await);
        let event_pub = Arc::new(StubEventPublisher::new());
        let uc = IncrementQuotaUsageUseCase::new(policy_repo, usage_repo, event_pub);

        let input = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 20, // 9990 + 20 > 10000
            request_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            IncrementQuotaUsageError::Exceeded { .. } => {}
            e => panic!("expected Exceeded error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn success_exactly_at_limit() {
        let policy = sample_policy(); // limit: 10000
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_usage(&policy.id, 9999).await);
        let event_pub = Arc::new(StubEventPublisher::new());
        let uc = IncrementQuotaUsageUseCase::new(policy_repo, usage_repo, event_pub);

        let input = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 1, // 9999 + 1 = 10000 (exactly at limit)
            request_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let inc = result.unwrap();
        assert_eq!(inc.used, 10000);
        assert_eq!(inc.remaining, 0);
        assert!(inc.allowed);
    }

    #[tokio::test]
    async fn success_threshold_event_published_when_crossing() {
        // Create policy with limit=100, threshold=80%
        let policy = QuotaPolicy::new(
            "Threshold Test".to_string(),
            SubjectType::Tenant,
            "tenant-thr".to_string(),
            100,
            Period::Daily,
            true,
            Some(80),
        );
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        // Current usage is 79, increment by 1 => 80 => crosses 80% threshold
        let usage_repo = Arc::new(StubUsageRepository::with_usage(&policy.id, 79).await);
        let event_pub = Arc::new(StubEventPublisher::new());
        let uc = IncrementQuotaUsageUseCase::new(policy_repo, usage_repo, event_pub.clone());

        let input = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 1,
            request_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let threshold_events = event_pub.threshold_events.read().await;
        assert_eq!(threshold_events.len(), 1);
        assert_eq!(threshold_events[0].event_type, "QUOTA_THRESHOLD_REACHED");
        assert_eq!(threshold_events[0].alert_threshold_percent, 80);
    }

    #[tokio::test]
    async fn success_no_threshold_event_when_already_above() {
        // Policy: limit=100, threshold=80%. Usage already at 85, increment by 1 => 86.
        // Since prev was already above 80%, no threshold event should fire.
        let policy = QuotaPolicy::new(
            "Above Threshold".to_string(),
            SubjectType::Tenant,
            "tenant-above".to_string(),
            100,
            Period::Daily,
            true,
            Some(80),
        );
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_usage(&policy.id, 85).await);
        let event_pub = Arc::new(StubEventPublisher::new());
        let uc = IncrementQuotaUsageUseCase::new(policy_repo, usage_repo, event_pub.clone());

        let input = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 1,
            request_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let threshold_events = event_pub.threshold_events.read().await;
        assert!(threshold_events.is_empty());
    }

    #[tokio::test]
    async fn error_policy_not_found() {
        let policy_repo = Arc::new(StubPolicyRepository::new());
        let usage_repo = Arc::new(StubUsageRepository::new());
        let event_pub = Arc::new(StubEventPublisher::new());
        let uc = IncrementQuotaUsageUseCase::new(policy_repo, usage_repo, event_pub);

        let input = IncrementQuotaUsageInput {
            quota_id: "nonexistent".to_string(),
            amount: 1,
            request_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            IncrementQuotaUsageError::NotFound(id) => assert_eq!(id, "nonexistent"),
            e => panic!("expected NotFound error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_policy_repo_failure() {
        let policy_repo = Arc::new(StubPolicyRepository::with_error());
        let usage_repo = Arc::new(StubUsageRepository::new());
        let event_pub = Arc::new(StubEventPublisher::new());
        let uc = IncrementQuotaUsageUseCase::new(policy_repo, usage_repo, event_pub);

        let input = IncrementQuotaUsageInput {
            quota_id: "some-id".to_string(),
            amount: 1,
            request_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            IncrementQuotaUsageError::Internal(msg) => {
                assert!(msg.contains("db connection error"))
            }
            e => panic!("expected Internal error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_usage_repo_failure() {
        let policy = sample_policy();
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_error());
        let event_pub = Arc::new(StubEventPublisher::new());
        let uc = IncrementQuotaUsageUseCase::new(policy_repo, usage_repo, event_pub);

        let input = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 1,
            request_id: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            IncrementQuotaUsageError::Internal(msg) => {
                assert!(msg.contains("redis connection error"))
            }
            e => panic!("expected Internal error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn success_with_new_without_publisher() {
        let policy = sample_policy();
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_usage(&policy.id, 100).await);
        let uc = IncrementQuotaUsageUseCase::new_without_publisher(policy_repo, usage_repo);

        let input = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 1,
            request_id: Some("req-123".to_string()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().used, 101);
    }
}

// ===========================================================================
// ResetQuotaUsageUseCase
// ===========================================================================

mod reset_quota_usage {
    use super::*;

    #[tokio::test]
    async fn success_resets_usage_to_zero() {
        let policy = sample_policy();
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_usage(&policy.id, 5000).await);
        let uc = ResetQuotaUsageUseCase::new(policy_repo, usage_repo.clone());

        let input = ResetQuotaUsageInput {
            quota_id: policy.id.clone(),
            reason: "plan upgrade".to_string(),
            reset_by: "admin@example.com".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.quota_id, policy.id);
        assert_eq!(output.used, 0);
        assert_eq!(output.reset_by, "admin@example.com");
        assert!(!output.reset_at.is_empty());

        // Verify usage was actually reset in the repository
        let current = usage_repo.usages.read().await;
        assert_eq!(*current.get(&policy.id).unwrap(), 0);
    }

    #[tokio::test]
    async fn error_empty_reason() {
        let policy_repo = Arc::new(StubPolicyRepository::new());
        let usage_repo = Arc::new(StubUsageRepository::new());
        let uc = ResetQuotaUsageUseCase::new(policy_repo, usage_repo);

        let input = ResetQuotaUsageInput {
            quota_id: "some-id".to_string(),
            reason: "".to_string(),
            reset_by: "admin".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ResetQuotaUsageError::Validation(msg) => assert!(msg.contains("reason")),
            e => panic!("expected Validation error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_policy_not_found() {
        let policy_repo = Arc::new(StubPolicyRepository::new());
        let usage_repo = Arc::new(StubUsageRepository::new());
        let uc = ResetQuotaUsageUseCase::new(policy_repo, usage_repo);

        let input = ResetQuotaUsageInput {
            quota_id: "nonexistent".to_string(),
            reason: "test reason".to_string(),
            reset_by: "admin".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ResetQuotaUsageError::NotFound(id) => assert_eq!(id, "nonexistent"),
            e => panic!("expected NotFound error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_policy_repo_failure() {
        let policy_repo = Arc::new(StubPolicyRepository::with_error());
        let usage_repo = Arc::new(StubUsageRepository::new());
        let uc = ResetQuotaUsageUseCase::new(policy_repo, usage_repo);

        let input = ResetQuotaUsageInput {
            quota_id: "some-id".to_string(),
            reason: "test reason".to_string(),
            reset_by: "admin".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ResetQuotaUsageError::Internal(msg) => assert!(msg.contains("db connection error")),
            e => panic!("expected Internal error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn error_usage_repo_failure() {
        let policy = sample_policy();
        let policy_repo = Arc::new(StubPolicyRepository::with_policy(&policy).await);
        let usage_repo = Arc::new(StubUsageRepository::with_error());
        let uc = ResetQuotaUsageUseCase::new(policy_repo, usage_repo);

        let input = ResetQuotaUsageInput {
            quota_id: policy.id.clone(),
            reason: "test reason".to_string(),
            reset_by: "admin".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ResetQuotaUsageError::Internal(msg) => assert!(msg.contains("redis connection error")),
            e => panic!("expected Internal error, got: {:?}", e),
        }
    }
}

// ===========================================================================
// End-to-end flow: Create -> Increment -> GetUsage -> Reset -> GetUsage
// ===========================================================================

mod e2e_flow {
    use super::*;

    #[tokio::test]
    async fn full_quota_lifecycle() {
        let policy_repo = Arc::new(StubPolicyRepository::new());
        let usage_repo = Arc::new(StubUsageRepository::new());
        let event_pub = Arc::new(StubEventPublisher::new());

        // 1. Create a policy
        let create_uc = CreateQuotaPolicyUseCase::new(policy_repo.clone());
        let create_input = CreateQuotaPolicyInput {
            name: "E2E Plan".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "tenant-e2e".to_string(),
            limit: 100,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: Some(80),
        };
        let policy = create_uc.execute(&create_input).await.unwrap();

        // 2. Get the policy
        let get_uc = GetQuotaPolicyUseCase::new(policy_repo.clone());
        let fetched = get_uc.execute(&policy.id).await.unwrap();
        assert_eq!(fetched.name, "E2E Plan");

        // 3. Increment usage
        let inc_uc = IncrementQuotaUsageUseCase::new(
            policy_repo.clone(),
            usage_repo.clone(),
            event_pub.clone(),
        );
        let inc_input = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 50,
            request_id: None,
        };
        let inc_result = inc_uc.execute(&inc_input).await.unwrap();
        assert_eq!(inc_result.used, 50);
        assert!(inc_result.allowed);

        // 4. Get usage
        let usage_uc = GetQuotaUsageUseCase::new(policy_repo.clone(), usage_repo.clone());
        let usage = usage_uc.execute(&policy.id).await.unwrap();
        assert_eq!(usage.used, 50);
        assert_eq!(usage.remaining, 50);

        // 5. Increment to cross threshold (80%)
        let inc_input2 = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 30, // 50 + 30 = 80 => exactly at 80% threshold
            request_id: None,
        };
        let inc_result2 = inc_uc.execute(&inc_input2).await.unwrap();
        assert_eq!(inc_result2.used, 80);

        // Threshold event should be published
        let threshold_events = event_pub.threshold_events.read().await;
        assert_eq!(threshold_events.len(), 1);
        drop(threshold_events);

        // 6. Try to exceed limit
        let inc_input3 = IncrementQuotaUsageInput {
            quota_id: policy.id.clone(),
            amount: 21, // 80 + 21 = 101 > 100
            request_id: None,
        };
        let exceed_result = inc_uc.execute(&inc_input3).await;
        assert!(exceed_result.is_err());

        // 7. Reset usage
        let reset_uc = ResetQuotaUsageUseCase::new(policy_repo.clone(), usage_repo.clone());
        let reset_input = ResetQuotaUsageInput {
            quota_id: policy.id.clone(),
            reason: "billing cycle reset".to_string(),
            reset_by: "system".to_string(),
        };
        let reset_result = reset_uc.execute(&reset_input).await.unwrap();
        assert_eq!(reset_result.used, 0);

        // 8. Verify usage is reset
        let usage_after = usage_uc.execute(&policy.id).await.unwrap();
        assert_eq!(usage_after.used, 0);
        assert_eq!(usage_after.remaining, 100);

        // 9. Update the policy
        let update_uc = UpdateQuotaPolicyUseCase::new(policy_repo.clone());
        let update_input = UpdateQuotaPolicyInput {
            id: policy.id.clone(),
            name: "E2E Plan v2".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "tenant-e2e".to_string(),
            limit: 200,
            period: "monthly".to_string(),
            enabled: true,
            alert_threshold_percent: Some(90),
        };
        let updated = update_uc.execute(&update_input).await.unwrap();
        assert_eq!(updated.limit, 200);
        assert_eq!(updated.period, Period::Monthly);

        // 10. Delete the policy
        let delete_uc = DeleteQuotaPolicyUseCase::new(policy_repo.clone());
        delete_uc.execute(&policy.id).await.unwrap();

        // 11. Verify policy is gone
        let get_result = get_uc.execute(&policy.id).await;
        assert!(matches!(get_result, Err(GetQuotaPolicyError::NotFound(_))));
    }
}
