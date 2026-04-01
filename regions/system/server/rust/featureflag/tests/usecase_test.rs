#![allow(clippy::unwrap_used)]
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use k1s0_featureflag_server::domain::entity::evaluation::EvaluationContext;
use k1s0_featureflag_server::domain::entity::feature_flag::{FeatureFlag, FlagRule, FlagVariant};
use k1s0_featureflag_server::domain::entity::flag_audit_log::FlagAuditLog;
use k1s0_featureflag_server::domain::repository::{FeatureFlagRepository, FlagAuditLogRepository};
use k1s0_featureflag_server::infrastructure::kafka_producer::FlagEventPublisher;
use k1s0_featureflag_server::usecase::create_flag::{
    CreateFlagError, CreateFlagInput, CreateFlagUseCase,
};
use k1s0_featureflag_server::usecase::delete_flag::{DeleteFlagError, DeleteFlagUseCase};
use k1s0_featureflag_server::usecase::evaluate_flag::{
    EvaluateFlagError, EvaluateFlagInput, EvaluateFlagUseCase,
};
use k1s0_featureflag_server::usecase::get_flag::{GetFlagError, GetFlagUseCase};
use k1s0_featureflag_server::usecase::list_flags::{ListFlagsError, ListFlagsUseCase};
use k1s0_featureflag_server::usecase::update_flag::{
    UpdateFlagError, UpdateFlagInput, UpdateFlagUseCase,
};
use k1s0_featureflag_server::usecase::watch_feature_flag::{
    FeatureFlagChangeEvent, WatchFeatureFlagUseCase,
};

// ---------------------------------------------------------------------------
// Stub: In-memory FeatureFlagRepository
// ---------------------------------------------------------------------------

struct StubFlagRepository {
    flags: RwLock<Vec<FeatureFlag>>,
    force_error: Option<String>,
}

impl StubFlagRepository {
    fn new() -> Self {
        Self {
            flags: RwLock::new(Vec::new()),
            force_error: None,
        }
    }

    fn with_flags(flags: Vec<FeatureFlag>) -> Self {
        Self {
            flags: RwLock::new(flags),
            force_error: None,
        }
    }

    fn with_error(msg: &str) -> Self {
        Self {
            flags: RwLock::new(Vec::new()),
            force_error: Some(msg.to_string()),
        }
    }
}

/// STATIC-CRITICAL-001 監査対応: StubFlagRepository の全メソッドで tenant_id を受け取る。
/// インメモリ実装のためテナント分離は行わないが、シグネチャを正しく合わせる。
#[async_trait]
impl FeatureFlagRepository for StubFlagRepository {
    async fn find_by_key(&self, _tenant_id: Uuid, flag_key: &str) -> anyhow::Result<FeatureFlag> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let flags = self.flags.read().await;
        flags
            .iter()
            .find(|f| f.flag_key == flag_key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("flag not found: {}", flag_key))
    }

    async fn find_all(&self, _tenant_id: Uuid) -> anyhow::Result<Vec<FeatureFlag>> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        Ok(self.flags.read().await.clone())
    }

    async fn create(&self, _tenant_id: Uuid, flag: &FeatureFlag) -> anyhow::Result<()> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        self.flags.write().await.push(flag.clone());
        Ok(())
    }

    async fn update(&self, _tenant_id: Uuid, flag: &FeatureFlag) -> anyhow::Result<()> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let mut flags = self.flags.write().await;
        if let Some(existing) = flags.iter_mut().find(|f| f.id == flag.id) {
            *existing = flag.clone();
            Ok(())
        } else {
            Err(anyhow::anyhow!("flag not found"))
        }
    }

    async fn delete(&self, _tenant_id: Uuid, id: &Uuid) -> anyhow::Result<bool> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let mut flags = self.flags.write().await;
        let len_before = flags.len();
        flags.retain(|f| f.id != *id);
        Ok(flags.len() < len_before)
    }

    async fn exists_by_key(&self, _tenant_id: Uuid, flag_key: &str) -> anyhow::Result<bool> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let flags = self.flags.read().await;
        Ok(flags.iter().any(|f| f.flag_key == flag_key))
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory FlagAuditLogRepository
// ---------------------------------------------------------------------------

struct StubAuditLogRepository {
    logs: RwLock<Vec<FlagAuditLog>>,
    force_error: Option<String>,
}

impl StubAuditLogRepository {
    fn new() -> Self {
        Self {
            logs: RwLock::new(Vec::new()),
            force_error: None,
        }
    }

    #[allow(dead_code)]
    fn with_error(msg: &str) -> Self {
        Self {
            logs: RwLock::new(Vec::new()),
            force_error: Some(msg.to_string()),
        }
    }
}

#[async_trait]
impl FlagAuditLogRepository for StubAuditLogRepository {
    async fn create(&self, log: &FlagAuditLog) -> anyhow::Result<()> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        self.logs.write().await.push(log.clone());
        Ok(())
    }

    async fn list_by_flag_id(
        &self,
        flag_id: &Uuid,
        limit: i64,
        _offset: i64,
    ) -> anyhow::Result<Vec<FlagAuditLog>> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let logs = self.logs.read().await;
        let result: Vec<FlagAuditLog> = logs
            .iter()
            .filter(|l| l.flag_id == *flag_id)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory FlagEventPublisher
// ---------------------------------------------------------------------------

struct StubEventPublisher {
    events: RwLock<Vec<String>>,
    force_error: Option<String>,
}

impl StubEventPublisher {
    fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            force_error: None,
        }
    }

    #[allow(dead_code)]
    fn with_error(msg: &str) -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            force_error: Some(msg.to_string()),
        }
    }
}

#[async_trait]
impl FlagEventPublisher for StubEventPublisher {
    async fn publish_flag_changed(
        &self,
        flag_key: &str,
        _enabled: bool,
        _actor_user_id: Option<String>,
        _before: Option<serde_json::Value>,
        _after: serde_json::Value,
    ) -> anyhow::Result<()> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        self.events.write().await.push(flag_key.to_string());
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// システムテナントUUID: テスト共通のフォールバックテナントID
fn system_tenant() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
}

/// STATIC-CRITICAL-001 監査対応: テスト用フラグはシステムテナントで作成する。
fn make_flag(key: &str, enabled: bool) -> FeatureFlag {
    FeatureFlag::new(system_tenant(), key.to_string(), format!("{} description", key), enabled)
}

fn make_flag_with_id(id: Uuid, key: &str, enabled: bool) -> FeatureFlag {
    let mut flag = FeatureFlag::new(system_tenant(), key.to_string(), format!("{} description", key), enabled);
    flag.id = id;
    flag
}

fn make_flag_with_variants(key: &str, enabled: bool, variants: Vec<FlagVariant>) -> FeatureFlag {
    let mut flag = make_flag(key, enabled);
    flag.variants = variants;
    flag
}

fn make_flag_with_rules(
    key: &str,
    enabled: bool,
    variants: Vec<FlagVariant>,
    rules: Vec<FlagRule>,
) -> FeatureFlag {
    let mut flag = make_flag_with_variants(key, enabled, variants);
    flag.rules = rules;
    flag
}

fn make_context(user_id: Option<&str>, attrs: HashMap<String, String>) -> EvaluationContext {
    EvaluationContext {
        user_id: user_id.map(|s| s.to_string()),
        tenant_id: None,
        attributes: attrs,
    }
}

fn make_stubs() -> (
    Arc<StubFlagRepository>,
    Arc<StubEventPublisher>,
    Arc<StubAuditLogRepository>,
) {
    (
        Arc::new(StubFlagRepository::new()),
        Arc::new(StubEventPublisher::new()),
        Arc::new(StubAuditLogRepository::new()),
    )
}

fn make_stubs_with_flags(
    flags: Vec<FeatureFlag>,
) -> (
    Arc<StubFlagRepository>,
    Arc<StubEventPublisher>,
    Arc<StubAuditLogRepository>,
) {
    (
        Arc::new(StubFlagRepository::with_flags(flags)),
        Arc::new(StubEventPublisher::new()),
        Arc::new(StubAuditLogRepository::new()),
    )
}

// ===========================================================================
// CreateFlag tests
// ===========================================================================

#[tokio::test]
async fn create_flag_success() {
    let (repo, publisher, audit) = make_stubs();
    let uc = CreateFlagUseCase::new(repo.clone(), publisher, audit);

    let input = CreateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.dark-mode".to_string(),
        description: "Dark mode toggle".to_string(),
        enabled: true,
        variants: vec![FlagVariant {
            name: "on".to_string(),
            value: "true".to_string(),
            weight: 100,
        }],
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.flag_key, "feature.dark-mode");
    assert_eq!(result.description, "Dark mode toggle");
    assert!(result.enabled);
    assert_eq!(result.variants.len(), 1);
    assert_eq!(result.variants[0].name, "on");

    // Verify persisted
    let stored = repo.flags.read().await;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].flag_key, "feature.dark-mode");
}

#[tokio::test]
async fn create_flag_already_exists() {
    let existing = make_flag("feature.existing", true);
    let (repo, publisher, audit) = make_stubs_with_flags(vec![existing]);
    let uc = CreateFlagUseCase::new(repo, publisher, audit);

    let input = CreateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.existing".to_string(),
        description: "dup".to_string(),
        enabled: true,
        variants: vec![],
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreateFlagError::AlreadyExists(key) => assert_eq!(key, "feature.existing"),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn create_flag_validation_empty_key() {
    let (repo, publisher, audit) = make_stubs();
    let uc = CreateFlagUseCase::new(repo, publisher, audit);

    let input = CreateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "  ".to_string(),
        description: "desc".to_string(),
        enabled: true,
        variants: vec![],
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreateFlagError::Internal(msg) => assert!(msg.contains("flag_key")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn create_flag_validation_key_too_long() {
    let (repo, publisher, audit) = make_stubs();
    let uc = CreateFlagUseCase::new(repo, publisher, audit);

    let input = CreateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "a".repeat(129),
        description: "desc".to_string(),
        enabled: true,
        variants: vec![],
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreateFlagError::Internal(msg) => assert!(msg.contains("128")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn create_flag_validation_negative_weight() {
    let (repo, publisher, audit) = make_stubs();
    let uc = CreateFlagUseCase::new(repo, publisher, audit);

    let input = CreateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.test".to_string(),
        description: "desc".to_string(),
        enabled: true,
        variants: vec![FlagVariant {
            name: "on".to_string(),
            value: "true".to_string(),
            weight: -1,
        }],
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreateFlagError::Internal(msg) => assert!(msg.contains("non-negative")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn create_flag_no_variants_is_valid() {
    let (repo, publisher, audit) = make_stubs();
    let uc = CreateFlagUseCase::new(repo, publisher, audit);

    let input = CreateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.simple".to_string(),
        description: "Simple boolean flag".to_string(),
        enabled: false,
        variants: vec![],
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.flag_key, "feature.simple");
    assert!(result.variants.is_empty());
}

#[tokio::test]
async fn create_flag_repo_error() {
    let repo = Arc::new(StubFlagRepository::with_error("connection refused"));
    let publisher = Arc::new(StubEventPublisher::new());
    let audit = Arc::new(StubAuditLogRepository::new());
    let uc = CreateFlagUseCase::new(repo, publisher, audit);

    let input = CreateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.test".to_string(),
        description: "desc".to_string(),
        enabled: true,
        variants: vec![],
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreateFlagError::Internal(msg) => assert!(msg.contains("connection refused")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn create_flag_audit_log_recorded() {
    let (repo, publisher, audit) = make_stubs();
    let uc = CreateFlagUseCase::new(repo, publisher, audit.clone());

    let input = CreateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.audited".to_string(),
        description: "Audited flag".to_string(),
        enabled: true,
        variants: vec![],
    };
    uc.execute(&input).await.unwrap();

    let logs = audit.logs.read().await;
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].flag_key, "feature.audited");
    assert_eq!(logs[0].action, "CREATED");
    assert!(logs[0].before_json.is_none());
    assert!(logs[0].after_json.is_some());
}

#[tokio::test]
async fn create_flag_event_published() {
    let (repo, publisher, audit) = make_stubs();
    let uc = CreateFlagUseCase::new(repo, publisher.clone(), audit);

    let input = CreateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.published".to_string(),
        description: "desc".to_string(),
        enabled: true,
        variants: vec![],
    };
    uc.execute(&input).await.unwrap();

    let events = publisher.events.read().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], "feature.published");
}

#[tokio::test]
async fn create_flag_with_watch_sender() {
    let (repo, publisher, audit) = make_stubs();
    let (tx, _) = tokio::sync::broadcast::channel::<FeatureFlagChangeEvent>(16);
    let mut rx = tx.subscribe();

    let uc = CreateFlagUseCase::new(repo, publisher, audit).with_watch_sender(tx);

    let input = CreateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.watched".to_string(),
        description: "Watched flag".to_string(),
        enabled: true,
        variants: vec![],
    };
    uc.execute(&input).await.unwrap();

    let event = rx.recv().await.unwrap();
    assert_eq!(event.flag_key, "feature.watched");
    assert_eq!(event.change_type, "CREATED");
    assert!(event.enabled);
}

// ===========================================================================
// GetFlag tests
// ===========================================================================

#[tokio::test]
async fn get_flag_found() {
    let flag = make_flag("feature.dark-mode", true);
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = GetFlagUseCase::new(repo);

    let result = uc.execute(system_tenant(), "feature.dark-mode").await.unwrap();
    assert_eq!(result.flag_key, "feature.dark-mode");
    assert!(result.enabled);
}

#[tokio::test]
async fn get_flag_not_found() {
    let repo = Arc::new(StubFlagRepository::new());
    let uc = GetFlagUseCase::new(repo);

    let err = uc.execute(system_tenant(), "nonexistent").await.unwrap_err();

    match err {
        GetFlagError::NotFound(key) => assert_eq!(key, "nonexistent"),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn get_flag_repo_error() {
    let repo = Arc::new(StubFlagRepository::with_error("db timeout"));
    let uc = GetFlagUseCase::new(repo);

    let err = uc.execute(system_tenant(), "any-key").await.unwrap_err();

    match err {
        GetFlagError::Internal(msg) => assert!(msg.contains("db timeout")),
        e => panic!("unexpected error: {:?}", e),
    }
}

// ===========================================================================
// UpdateFlag tests
// ===========================================================================

#[tokio::test]
async fn update_flag_success_partial() {
    let flag = make_flag("feature.update-me", true);
    let (repo, publisher, audit) = make_stubs_with_flags(vec![flag]);
    let uc = UpdateFlagUseCase::new(repo.clone(), publisher, audit);

    let input = UpdateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.update-me".to_string(),
        enabled: Some(false),
        description: Some("Updated description".to_string()),
        variants: None,
        rules: None,
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.flag_key, "feature.update-me");
    assert!(!result.enabled);
    assert_eq!(result.description, "Updated description");

    // Verify persisted
    let stored = repo.flags.read().await;
    assert!(!stored[0].enabled);
    assert_eq!(stored[0].description, "Updated description");
}

#[tokio::test]
async fn update_flag_success_add_variants_and_rules() {
    let flag = make_flag("feature.complex", true);
    let (repo, publisher, audit) = make_stubs_with_flags(vec![flag]);
    let uc = UpdateFlagUseCase::new(repo, publisher, audit);

    let input = UpdateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.complex".to_string(),
        enabled: None,
        description: None,
        variants: Some(vec![
            FlagVariant {
                name: "control".to_string(),
                value: "false".to_string(),
                weight: 50,
            },
            FlagVariant {
                name: "treatment".to_string(),
                value: "true".to_string(),
                weight: 50,
            },
        ]),
        rules: Some(vec![FlagRule {
            attribute: "tenant_id".to_string(),
            operator: "eq".to_string(),
            value: "premium-tenant".to_string(),
            variant: "treatment".to_string(),
        }]),
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.variants.len(), 2);
    assert_eq!(result.rules.len(), 1);
    assert_eq!(result.rules[0].attribute, "tenant_id");
}

#[tokio::test]
async fn update_flag_not_found() {
    let (repo, publisher, audit) = make_stubs();
    let uc = UpdateFlagUseCase::new(repo, publisher, audit);

    let input = UpdateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "nonexistent".to_string(),
        enabled: Some(true),
        description: None,
        variants: None,
        rules: None,
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        UpdateFlagError::NotFound(key) => assert_eq!(key, "nonexistent"),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn update_flag_invalid_variants() {
    let flag = make_flag("feature.bad-variant", true);
    let (repo, publisher, audit) = make_stubs_with_flags(vec![flag]);
    let uc = UpdateFlagUseCase::new(repo, publisher, audit);

    let input = UpdateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.bad-variant".to_string(),
        enabled: None,
        description: None,
        variants: Some(vec![FlagVariant {
            name: "bad".to_string(),
            value: "true".to_string(),
            weight: -5,
        }]),
        rules: None,
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        UpdateFlagError::Internal(msg) => assert!(msg.contains("non-negative")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn update_flag_audit_log_before_and_after() {
    let flag = make_flag("feature.audited-update", true);
    let (repo, publisher, audit) = make_stubs_with_flags(vec![flag]);
    let uc = UpdateFlagUseCase::new(repo, publisher, audit.clone());

    let input = UpdateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.audited-update".to_string(),
        enabled: Some(false),
        description: None,
        variants: None,
        rules: None,
    };
    uc.execute(&input).await.unwrap();

    let logs = audit.logs.read().await;
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, "UPDATED");
    assert!(logs[0].before_json.is_some());
    assert!(logs[0].after_json.is_some());

    let before = logs[0].before_json.as_ref().unwrap();
    assert_eq!(before["enabled"], true);

    let after = logs[0].after_json.as_ref().unwrap();
    assert_eq!(after["enabled"], false);
}

#[tokio::test]
async fn update_flag_with_watch_sender() {
    let flag = make_flag("feature.watch-update", true);
    let (repo, publisher, audit) = make_stubs_with_flags(vec![flag]);
    let (tx, _) = tokio::sync::broadcast::channel::<FeatureFlagChangeEvent>(16);
    let mut rx = tx.subscribe();

    let uc = UpdateFlagUseCase::new(repo, publisher, audit).with_watch_sender(tx);

    let input = UpdateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.watch-update".to_string(),
        enabled: Some(false),
        description: None,
        variants: None,
        rules: None,
    };
    uc.execute(&input).await.unwrap();

    let event = rx.recv().await.unwrap();
    assert_eq!(event.flag_key, "feature.watch-update");
    assert_eq!(event.change_type, "UPDATED");
    assert!(!event.enabled);
}

#[tokio::test]
async fn update_flag_repo_error() {
    let repo = Arc::new(StubFlagRepository::with_error("update failed"));
    let publisher = Arc::new(StubEventPublisher::new());
    let audit = Arc::new(StubAuditLogRepository::new());
    let uc = UpdateFlagUseCase::new(repo, publisher, audit);

    let input = UpdateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "any".to_string(),
        enabled: None,
        description: None,
        variants: None,
        rules: None,
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        UpdateFlagError::Internal(msg) => assert!(msg.contains("update failed")),
        e => panic!("unexpected error: {:?}", e),
    }
}

// ===========================================================================
// DeleteFlag tests
// ===========================================================================

#[tokio::test]
async fn delete_flag_success() {
    let id = Uuid::new_v4();
    let flag = make_flag_with_id(id, "feature.delete-me", true);
    let (repo, publisher, audit) = make_stubs_with_flags(vec![flag]);
    let uc = DeleteFlagUseCase::new(repo.clone(), publisher, audit);

    uc.execute(system_tenant(), &id).await.unwrap();

    let stored = repo.flags.read().await;
    assert!(stored.is_empty());
}

#[tokio::test]
async fn delete_flag_not_found() {
    let (repo, publisher, audit) = make_stubs();
    let uc = DeleteFlagUseCase::new(repo, publisher, audit);
    let id = Uuid::new_v4();

    let err = uc.execute(system_tenant(), &id).await.unwrap_err();

    match err {
        DeleteFlagError::NotFound(found_id) => assert_eq!(found_id, id),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn delete_flag_repo_error() {
    let repo = Arc::new(StubFlagRepository::with_error("disk full"));
    let publisher = Arc::new(StubEventPublisher::new());
    let audit = Arc::new(StubAuditLogRepository::new());
    let uc = DeleteFlagUseCase::new(repo, publisher, audit);

    let err = uc.execute(system_tenant(), &Uuid::new_v4()).await.unwrap_err();

    match err {
        DeleteFlagError::Internal(msg) => assert!(msg.contains("disk full")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn delete_flag_audit_log_recorded() {
    let id = Uuid::new_v4();
    let flag = make_flag_with_id(id, "feature.audited-delete", true);
    let (repo, publisher, audit) = make_stubs_with_flags(vec![flag]);
    let uc = DeleteFlagUseCase::new(repo, publisher, audit.clone());

    uc.execute(system_tenant(), &id).await.unwrap();

    let logs = audit.logs.read().await;
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, "DELETED");
    assert!(logs[0].before_json.is_some());
    assert!(logs[0].after_json.is_none());
}

#[tokio::test]
async fn delete_flag_with_watch_sender() {
    let id = Uuid::new_v4();
    let flag = make_flag_with_id(id, "feature.watch-delete", true);
    let (repo, publisher, audit) = make_stubs_with_flags(vec![flag]);
    let (tx, _) = tokio::sync::broadcast::channel::<FeatureFlagChangeEvent>(16);
    let mut rx = tx.subscribe();

    let uc = DeleteFlagUseCase::new(repo, publisher, audit).with_watch_sender(tx);

    uc.execute(system_tenant(), &id).await.unwrap();

    let event = rx.recv().await.unwrap();
    assert_eq!(event.flag_key, "feature.watch-delete");
    assert_eq!(event.change_type, "DELETED");
}

// ===========================================================================
// ListFlags tests
// ===========================================================================

#[tokio::test]
async fn list_flags_empty() {
    let repo = Arc::new(StubFlagRepository::new());
    let uc = ListFlagsUseCase::new(repo);

    let flags = uc.execute(system_tenant()).await.unwrap();
    assert!(flags.is_empty());
}

#[tokio::test]
async fn list_flags_with_results() {
    let f1 = make_flag("feature.a", true);
    let f2 = make_flag("feature.b", false);
    let f3 = make_flag("feature.c", true);
    let repo = Arc::new(StubFlagRepository::with_flags(vec![f1, f2, f3]));
    let uc = ListFlagsUseCase::new(repo);

    let flags = uc.execute(system_tenant()).await.unwrap();
    assert_eq!(flags.len(), 3);
}

#[tokio::test]
async fn list_flags_repo_error() {
    let repo = Arc::new(StubFlagRepository::with_error("db error"));
    let uc = ListFlagsUseCase::new(repo);

    let err = uc.execute(system_tenant()).await.unwrap_err();

    match err {
        ListFlagsError::Internal(msg) => assert!(msg.contains("db error")),
    }
}

// ===========================================================================
// EvaluateFlag tests — core rule evaluation logic
// ===========================================================================

#[tokio::test]
async fn evaluate_flag_disabled_returns_false() {
    let flag = make_flag("feature.disabled", false);
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.disabled".to_string(),
        context: make_context(Some("user-1"), HashMap::new()),
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(!result.enabled);
    assert!(result.variant.is_none());
    assert_eq!(result.reason, "flag is disabled");
}

#[tokio::test]
async fn evaluate_flag_enabled_with_variant() {
    let flag = make_flag_with_variants(
        "feature.enabled",
        true,
        vec![FlagVariant {
            name: "on".to_string(),
            value: "true".to_string(),
            weight: 100,
        }],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.enabled".to_string(),
        context: make_context(Some("user-1"), HashMap::new()),
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
    assert_eq!(result.variant, Some("on".to_string()));
    assert_eq!(result.reason, "flag is enabled");
}

#[tokio::test]
async fn evaluate_flag_not_found() {
    let repo = Arc::new(StubFlagRepository::new());
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "nonexistent".to_string(),
        context: make_context(None, HashMap::new()),
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        EvaluateFlagError::FlagNotFound(key) => assert_eq!(key, "nonexistent"),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn evaluate_flag_repo_error() {
    let repo = Arc::new(StubFlagRepository::with_error("connection lost"));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "any".to_string(),
        context: make_context(None, HashMap::new()),
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        EvaluateFlagError::Internal(msg) => assert!(msg.contains("connection lost")),
        e => panic!("unexpected error: {:?}", e),
    }
}

// --- Rule evaluation: operator "eq" ---

#[tokio::test]
async fn evaluate_flag_rule_eq_match() {
    let flag = make_flag_with_rules(
        "feature.eq-rule",
        true,
        vec![
            FlagVariant {
                name: "control".to_string(),
                value: "false".to_string(),
                weight: 50,
            },
            FlagVariant {
                name: "treatment".to_string(),
                value: "true".to_string(),
                weight: 50,
            },
        ],
        vec![FlagRule {
            attribute: "user_id".to_string(),
            operator: "eq".to_string(),
            value: "vip-user".to_string(),
            variant: "treatment".to_string(),
        }],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.eq-rule".to_string(),
        context: make_context(Some("vip-user"), HashMap::new()),
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
    assert_eq!(result.variant, Some("treatment".to_string()));
}

#[tokio::test]
async fn evaluate_flag_rule_eq_no_match_falls_to_weighted() {
    let flag = make_flag_with_rules(
        "feature.eq-nomatch",
        true,
        vec![FlagVariant {
            name: "control".to_string(),
            value: "false".to_string(),
            weight: 100,
        }],
        vec![FlagRule {
            attribute: "user_id".to_string(),
            operator: "eq".to_string(),
            value: "vip-user".to_string(),
            variant: "control".to_string(),
        }],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.eq-nomatch".to_string(),
        context: make_context(Some("regular-user"), HashMap::new()),
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
    // Falls to weighted selection since rule doesn't match
    assert!(result.variant.is_some());
}

// --- Rule evaluation: operator "contains" ---

#[tokio::test]
async fn evaluate_flag_rule_contains_match() {
    let flag = make_flag_with_rules(
        "feature.contains-rule",
        true,
        vec![FlagVariant {
            name: "beta".to_string(),
            value: "true".to_string(),
            weight: 100,
        }],
        vec![FlagRule {
            attribute: "user_id".to_string(),
            operator: "contains".to_string(),
            value: "beta".to_string(),
            variant: "beta".to_string(),
        }],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.contains-rule".to_string(),
        context: make_context(Some("beta-tester-42"), HashMap::new()),
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
    assert_eq!(result.variant, Some("beta".to_string()));
}

#[tokio::test]
async fn evaluate_flag_rule_contains_no_match() {
    let flag = make_flag_with_rules(
        "feature.contains-nomatch",
        true,
        vec![FlagVariant {
            name: "stable".to_string(),
            value: "false".to_string(),
            weight: 100,
        }],
        vec![FlagRule {
            attribute: "user_id".to_string(),
            operator: "contains".to_string(),
            value: "beta".to_string(),
            variant: "stable".to_string(),
        }],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.contains-nomatch".to_string(),
        context: make_context(Some("production-user"), HashMap::new()),
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
    // Rule didn't match, falls back to weighted variant selection
    assert_eq!(result.variant, Some("stable".to_string()));
}

// --- Rule evaluation: operator "in" ---

#[tokio::test]
async fn evaluate_flag_rule_in_match() {
    let flag = make_flag_with_rules(
        "feature.in-rule",
        true,
        vec![FlagVariant {
            name: "premium".to_string(),
            value: "true".to_string(),
            weight: 100,
        }],
        vec![FlagRule {
            attribute: "tenant_id".to_string(),
            operator: "in".to_string(),
            value: "tenant-a, tenant-b, tenant-c".to_string(),
            variant: "premium".to_string(),
        }],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.in-rule".to_string(),
        context: EvaluationContext {
            user_id: None,
            tenant_id: Some("tenant-b".to_string()),
            attributes: HashMap::new(),
        },
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
    assert_eq!(result.variant, Some("premium".to_string()));
}

#[tokio::test]
async fn evaluate_flag_rule_in_no_match() {
    let flag = make_flag_with_rules(
        "feature.in-nomatch",
        true,
        vec![FlagVariant {
            name: "free".to_string(),
            value: "false".to_string(),
            weight: 100,
        }],
        vec![FlagRule {
            attribute: "tenant_id".to_string(),
            operator: "in".to_string(),
            value: "tenant-a, tenant-b".to_string(),
            variant: "free".to_string(),
        }],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.in-nomatch".to_string(),
        context: EvaluationContext {
            user_id: None,
            tenant_id: Some("tenant-x".to_string()),
            attributes: HashMap::new(),
        },
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
    // Falls to weighted since rule didn't match
    assert!(result.variant.is_some());
}

// --- Rule evaluation: custom attributes ---

#[tokio::test]
async fn evaluate_flag_rule_custom_attribute() {
    let mut attrs = HashMap::new();
    attrs.insert("plan".to_string(), "enterprise".to_string());

    let flag = make_flag_with_rules(
        "feature.custom-attr",
        true,
        vec![FlagVariant {
            name: "enterprise".to_string(),
            value: "true".to_string(),
            weight: 100,
        }],
        vec![FlagRule {
            attribute: "plan".to_string(),
            operator: "eq".to_string(),
            value: "enterprise".to_string(),
            variant: "enterprise".to_string(),
        }],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.custom-attr".to_string(),
        context: make_context(Some("user-1"), attrs),
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
    assert_eq!(result.variant, Some("enterprise".to_string()));
}

// --- Rule evaluation: unknown operator ---

#[tokio::test]
async fn evaluate_flag_rule_unknown_operator_skips() {
    let flag = make_flag_with_rules(
        "feature.unknown-op",
        true,
        vec![FlagVariant {
            name: "default".to_string(),
            value: "true".to_string(),
            weight: 100,
        }],
        vec![FlagRule {
            attribute: "user_id".to_string(),
            operator: "regex".to_string(), // unsupported
            value: ".*".to_string(),
            variant: "default".to_string(),
        }],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.unknown-op".to_string(),
        context: make_context(Some("user-1"), HashMap::new()),
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
    // Unknown operator doesn't match, falls to weighted selection
    assert_eq!(result.variant, Some("default".to_string()));
}

// --- Rule evaluation: missing attribute in context ---

#[tokio::test]
async fn evaluate_flag_rule_missing_attribute_skips() {
    let flag = make_flag_with_rules(
        "feature.missing-attr",
        true,
        vec![FlagVariant {
            name: "fallback".to_string(),
            value: "true".to_string(),
            weight: 100,
        }],
        vec![FlagRule {
            attribute: "region".to_string(),
            operator: "eq".to_string(),
            value: "us-east".to_string(),
            variant: "fallback".to_string(),
        }],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.missing-attr".to_string(),
        context: make_context(Some("user-1"), HashMap::new()), // no "region" attribute
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
    // Rule skipped because attribute not in context, falls to weighted
    assert_eq!(result.variant, Some("fallback".to_string()));
}

// --- Rule evaluation: multiple rules, first match wins ---

#[tokio::test]
async fn evaluate_flag_multiple_rules_first_match_wins() {
    let flag = make_flag_with_rules(
        "feature.multi-rule",
        true,
        vec![
            FlagVariant {
                name: "vip".to_string(),
                value: "true".to_string(),
                weight: 50,
            },
            FlagVariant {
                name: "beta".to_string(),
                value: "true".to_string(),
                weight: 50,
            },
        ],
        vec![
            FlagRule {
                attribute: "user_id".to_string(),
                operator: "eq".to_string(),
                value: "vip-user".to_string(),
                variant: "vip".to_string(),
            },
            FlagRule {
                attribute: "user_id".to_string(),
                operator: "contains".to_string(),
                value: "vip".to_string(),
                variant: "beta".to_string(),
            },
        ],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.multi-rule".to_string(),
        context: make_context(Some("vip-user"), HashMap::new()),
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
    // First rule matches exactly, so "vip" variant is returned
    assert_eq!(result.variant, Some("vip".to_string()));
}

// --- Rule evaluation: rule variant not in variants list ---

#[tokio::test]
async fn evaluate_flag_rule_variant_not_in_variants_list_skips() {
    let flag = make_flag_with_rules(
        "feature.orphan-variant",
        true,
        vec![FlagVariant {
            name: "real".to_string(),
            value: "true".to_string(),
            weight: 100,
        }],
        vec![FlagRule {
            attribute: "user_id".to_string(),
            operator: "eq".to_string(),
            value: "user-1".to_string(),
            variant: "nonexistent-variant".to_string(), // not in variants list
        }],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.orphan-variant".to_string(),
        context: make_context(Some("user-1"), HashMap::new()),
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.enabled);
    // Rule matched but variant not in variants list, so skipped; falls to weighted
    assert_eq!(result.variant, Some("real".to_string()));
}

// --- Weighted variant selection: deterministic ---

#[tokio::test]
async fn evaluate_flag_weighted_selection_deterministic() {
    let flag = make_flag_with_variants(
        "feature.weighted",
        true,
        vec![
            FlagVariant {
                name: "a".to_string(),
                value: "a".to_string(),
                weight: 50,
            },
            FlagVariant {
                name: "b".to_string(),
                value: "b".to_string(),
                weight: 50,
            },
        ],
    );
    let repo = Arc::new(StubFlagRepository::with_flags(vec![flag]));
    let uc = EvaluateFlagUseCase::new(repo);

    // Same user_id should always produce the same variant (deterministic hashing)
    let input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.weighted".to_string(),
        context: make_context(Some("consistent-user"), HashMap::new()),
    };
    let result1 = uc.execute(&input).await.unwrap();
    let result2 = uc.execute(&input).await.unwrap();

    assert_eq!(result1.variant, result2.variant);
}

// ===========================================================================
// WatchFeatureFlag tests
// ===========================================================================

#[tokio::test]
async fn watch_subscribe_and_notify() {
    let (uc, _tx) = WatchFeatureFlagUseCase::new();
    let mut rx = uc.subscribe();

    uc.notify(FeatureFlagChangeEvent {
        flag_key: "feature.watched".to_string(),
        change_type: "UPDATED".to_string(),
        enabled: true,
        description: "Watched flag".to_string(),
    });

    let event = rx.recv().await.unwrap();
    assert_eq!(event.flag_key, "feature.watched");
    assert_eq!(event.change_type, "UPDATED");
    assert!(event.enabled);
}

#[tokio::test]
async fn watch_multiple_subscribers() {
    let (uc, _tx) = WatchFeatureFlagUseCase::new();
    let mut rx1 = uc.subscribe();
    let mut rx2 = uc.subscribe();

    uc.notify(FeatureFlagChangeEvent {
        flag_key: "feature.multi-sub".to_string(),
        change_type: "CREATED".to_string(),
        enabled: false,
        description: "Multi subscriber".to_string(),
    });

    let e1 = rx1.recv().await.unwrap();
    let e2 = rx2.recv().await.unwrap();
    assert_eq!(e1.flag_key, e2.flag_key);
    assert_eq!(e1.change_type, "CREATED");
}

#[tokio::test]
async fn watch_closed_channel() {
    let (tx, _) = tokio::sync::broadcast::channel::<FeatureFlagChangeEvent>(4);
    let mut rx = tx.subscribe();
    drop(tx);
    assert!(rx.recv().await.is_err());
}

// ===========================================================================
// End-to-end workflow: create -> get -> update -> evaluate -> list -> delete
// ===========================================================================

#[tokio::test]
async fn flag_crud_workflow() {
    let repo = Arc::new(StubFlagRepository::new());
    let publisher = Arc::new(StubEventPublisher::new());
    let audit = Arc::new(StubAuditLogRepository::new());

    // 1. Create
    let create_uc = CreateFlagUseCase::new(repo.clone(), publisher.clone(), audit.clone());
    let create_input = CreateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.workflow".to_string(),
        description: "Workflow test flag".to_string(),
        enabled: true,
        variants: vec![FlagVariant {
            name: "on".to_string(),
            value: "true".to_string(),
            weight: 100,
        }],
    };
    let created = create_uc.execute(&create_input).await.unwrap();
    let flag_id = created.id;

    // 2. Get
    let get_uc = GetFlagUseCase::new(repo.clone());
    let fetched = get_uc.execute(system_tenant(), "feature.workflow").await.unwrap();
    assert_eq!(fetched.flag_key, "feature.workflow");
    assert!(fetched.enabled);

    // 3. Evaluate (enabled => should get variant)
    let eval_uc = EvaluateFlagUseCase::new(repo.clone());
    let eval_input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.workflow".to_string(),
        context: make_context(Some("user-1"), HashMap::new()),
    };
    let eval_result = eval_uc.execute(&eval_input).await.unwrap();
    assert!(eval_result.enabled);
    assert_eq!(eval_result.variant, Some("on".to_string()));

    // 4. Update (disable)
    let update_uc = UpdateFlagUseCase::new(repo.clone(), publisher.clone(), audit.clone());
    let update_input = UpdateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.workflow".to_string(),
        enabled: Some(false),
        description: Some("Disabled workflow flag".to_string()),
        variants: None,
        rules: None,
    };
    let updated = update_uc.execute(&update_input).await.unwrap();
    assert!(!updated.enabled);
    assert_eq!(updated.description, "Disabled workflow flag");

    // 5. Evaluate again (disabled => false)
    let eval_result2 = eval_uc.execute(&eval_input).await.unwrap();
    assert!(!eval_result2.enabled);
    assert!(eval_result2.variant.is_none());

    // 6. List
    let list_uc = ListFlagsUseCase::new(repo.clone());
    let flags = list_uc.execute(system_tenant()).await.unwrap();
    assert_eq!(flags.len(), 1);

    // 7. Delete
    let delete_uc = DeleteFlagUseCase::new(repo.clone(), publisher, audit.clone());
    delete_uc.execute(system_tenant(), &flag_id).await.unwrap();

    // 8. Verify deleted
    let err = get_uc.execute(system_tenant(), "feature.workflow").await.unwrap_err();
    assert!(matches!(err, GetFlagError::NotFound(_)));

    // 9. Verify audit trail
    let logs = audit.logs.read().await;
    assert_eq!(logs.len(), 3); // CREATED, UPDATED, DELETED
    assert_eq!(logs[0].action, "CREATED");
    assert_eq!(logs[1].action, "UPDATED");
    assert_eq!(logs[2].action, "DELETED");
}

#[tokio::test]
async fn flag_evaluate_with_rules_workflow() {
    let repo = Arc::new(StubFlagRepository::new());
    let publisher = Arc::new(StubEventPublisher::new());
    let audit = Arc::new(StubAuditLogRepository::new());

    // 1. Create flag with variants
    let create_uc = CreateFlagUseCase::new(repo.clone(), publisher.clone(), audit.clone());
    let create_input = CreateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.ab-test".to_string(),
        description: "A/B test".to_string(),
        enabled: true,
        variants: vec![
            FlagVariant {
                name: "control".to_string(),
                value: "false".to_string(),
                weight: 50,
            },
            FlagVariant {
                name: "treatment".to_string(),
                value: "true".to_string(),
                weight: 50,
            },
        ],
    };
    create_uc.execute(&create_input).await.unwrap();

    // 2. Add targeting rules via update
    let update_uc = UpdateFlagUseCase::new(repo.clone(), publisher, audit);
    let update_input = UpdateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.ab-test".to_string(),
        enabled: None,
        description: None,
        variants: None,
        rules: Some(vec![FlagRule {
            attribute: "tenant_id".to_string(),
            operator: "in".to_string(),
            value: "premium-1, premium-2".to_string(),
            variant: "treatment".to_string(),
        }]),
    };
    update_uc.execute(&update_input).await.unwrap();

    // 3. Evaluate for premium tenant => treatment
    let eval_uc = EvaluateFlagUseCase::new(repo.clone());
    let eval_input = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.ab-test".to_string(),
        context: EvaluationContext {
            user_id: Some("user-1".to_string()),
            tenant_id: Some("premium-1".to_string()),
            attributes: HashMap::new(),
        },
    };
    let result = eval_uc.execute(&eval_input).await.unwrap();
    assert!(result.enabled);
    assert_eq!(result.variant, Some("treatment".to_string()));

    // 4. Evaluate for non-premium tenant => weighted selection (either control or treatment)
    let eval_input2 = EvaluateFlagInput {
        tenant_id: system_tenant(),
        flag_key: "feature.ab-test".to_string(),
        context: EvaluationContext {
            user_id: Some("user-2".to_string()),
            tenant_id: Some("free-tenant".to_string()),
            attributes: HashMap::new(),
        },
    };
    let result2 = eval_uc.execute(&eval_input2).await.unwrap();
    assert!(result2.enabled);
    assert!(result2.variant.is_some());
    // Variant is deterministic based on user_id hash
    let variant_name = result2.variant.unwrap();
    assert!(variant_name == "control" || variant_name == "treatment");
}
