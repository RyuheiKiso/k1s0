#![allow(clippy::unwrap_used)]
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use k1s0_vault_server::domain::entity::access_log::{AccessAction, SecretAccessLog};
use k1s0_vault_server::domain::entity::secret::Secret;
use k1s0_vault_server::domain::repository::{AccessLogRepository, SecretStore};
use k1s0_vault_server::infrastructure::kafka_producer::{
    NoopVaultEventPublisher, VaultAccessEvent, VaultEventPublisher, VaultSecretRotatedEvent,
};
use k1s0_vault_server::usecase::delete_secret::{DeleteSecretInput, DeleteSecretUseCase};
use k1s0_vault_server::usecase::get_secret::{GetSecretError, GetSecretInput, GetSecretUseCase};
use k1s0_vault_server::usecase::list_audit_logs::{ListAuditLogsInput, ListAuditLogsUseCase};
use k1s0_vault_server::usecase::list_secrets::ListSecretsUseCase;
use k1s0_vault_server::usecase::rotate_secret::{
    RotateSecretError, RotateSecretInput, RotateSecretUseCase,
};
use k1s0_vault_server::usecase::set_secret::{SetSecretError, SetSecretInput, SetSecretUseCase};

// ============================================================================
// Test Stub: In-Memory SecretStore
// ============================================================================

struct StubSecretStore {
    secrets: RwLock<HashMap<String, Secret>>,
    should_fail: bool,
}

impl StubSecretStore {
    fn new() -> Self {
        Self {
            secrets: RwLock::new(HashMap::new()),
            should_fail: false,
        }
    }

    fn with_error() -> Self {
        Self {
            secrets: RwLock::new(HashMap::new()),
            should_fail: true,
        }
    }

    fn with_secrets(secrets: Vec<Secret>) -> Self {
        let map: HashMap<String, Secret> =
            secrets.into_iter().map(|s| (s.path.clone(), s)).collect();
        Self {
            secrets: RwLock::new(map),
            should_fail: false,
        }
    }
}

#[async_trait]
impl SecretStore for StubSecretStore {
    async fn get(&self, path: &str, _version: Option<i64>) -> anyhow::Result<Secret> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let store = self.secrets.read().await;
        store
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("secret not found: {}", path))
    }

    async fn set(&self, path: &str, data: HashMap<String, String>) -> anyhow::Result<i64> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let mut store = self.secrets.write().await;
        let version = if let Some(existing) = store.remove(path) {
            let new_version = existing.current_version + 1;
            let updated = existing.update(data);
            store.insert(path.to_string(), updated);
            new_version
        } else {
            let secret = Secret::new(path.to_string(), data);
            store.insert(path.to_string(), secret);
            1
        };
        Ok(version)
    }

    async fn delete(&self, path: &str, _versions: Vec<i64>) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let mut store = self.secrets.write().await;
        if store.remove(path).is_some() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("secret not found: {}", path))
        }
    }

    async fn list(&self, path_prefix: &str) -> anyhow::Result<Vec<String>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let store = self.secrets.read().await;
        let paths: Vec<String> = store
            .keys()
            .filter(|k| k.starts_with(path_prefix))
            .cloned()
            .collect();
        Ok(paths)
    }

    async fn exists(&self, path: &str) -> anyhow::Result<bool> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let store = self.secrets.read().await;
        Ok(store.contains_key(path))
    }
}

// ============================================================================
// Test Stub: In-Memory AccessLogRepository
// ============================================================================

struct StubAccessLogRepository {
    logs: RwLock<Vec<SecretAccessLog>>,
    should_fail: bool,
}

impl StubAccessLogRepository {
    fn new() -> Self {
        Self {
            logs: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_error() -> Self {
        Self {
            logs: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }

    fn with_logs(logs: Vec<SecretAccessLog>) -> Self {
        Self {
            logs: RwLock::new(logs),
            should_fail: false,
        }
    }

    async fn recorded_logs(&self) -> Vec<SecretAccessLog> {
        self.logs.read().await.clone()
    }
}

#[async_trait]
impl AccessLogRepository for StubAccessLogRepository {
    async fn record(&self, log: &SecretAccessLog) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("audit log backend unavailable"));
        }
        self.logs.write().await.push(log.clone());
        Ok(())
    }

    // LOW-12 監査対応: keyset ページネーションシグネチャに対応。
    // スタブでは after_id の位置を線形探索してカーソルベースのページングを模倣する。
    async fn list(
        &self,
        after_id: Option<uuid::Uuid>,
        limit: u32,
    ) -> anyhow::Result<(Vec<SecretAccessLog>, Option<uuid::Uuid>)> {
        if self.should_fail {
            return Err(anyhow::anyhow!("audit log backend unavailable"));
        }
        let logs = self.logs.read().await;
        let start = if let Some(cursor) = after_id {
            logs.iter()
                .position(|l| l.id == cursor)
                .map(|pos| pos + 1)
                .unwrap_or(0)
        } else {
            0
        };
        let page: Vec<SecretAccessLog> = logs
            .iter()
            .skip(start)
            .take(limit as usize + 1)
            .cloned()
            .collect();
        let next_cursor = if page.len() > limit as usize {
            page.get(limit as usize - 1).map(|l| l.id)
        } else {
            None
        };
        let result = page.into_iter().take(limit as usize).collect();
        Ok((result, next_cursor))
    }
}

// ============================================================================
// Test Stub: In-Memory VaultEventPublisher
// ============================================================================

struct StubEventPublisher {
    access_events: RwLock<Vec<String>>,
    rotated_events: RwLock<Vec<String>>,
    should_fail: bool,
}

impl StubEventPublisher {
    fn new() -> Self {
        Self {
            access_events: RwLock::new(Vec::new()),
            rotated_events: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_error() -> Self {
        Self {
            access_events: RwLock::new(Vec::new()),
            rotated_events: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }
}

#[async_trait]
impl VaultEventPublisher for StubEventPublisher {
    async fn publish_secret_accessed(&self, event: &VaultAccessEvent) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("event publish failed"));
        }
        self.access_events
            .write()
            .await
            .push(format!("{}:{}", event.key_path, event.action));
        Ok(())
    }

    async fn publish_secret_rotated(&self, event: &VaultSecretRotatedEvent) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("event publish failed"));
        }
        self.rotated_events.write().await.push(format!(
            "{}:{}→{}",
            event.key_path, event.old_version, event.new_version
        ));
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ============================================================================
// Helper: build common test data
// ============================================================================

fn make_secret(path: &str, key: &str, value: &str) -> Secret {
    let data = HashMap::from([(key.to_string(), value.to_string())]);
    Secret::new(path.to_string(), data)
}

fn make_secret_data(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// ============================================================================
// SetSecretUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_set_secret_success_creates_new_secret() {
    let store = Arc::new(StubSecretStore::new());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = SetSecretUseCase::new(store.clone(), audit.clone(), publisher);

    let input = SetSecretInput {
        path: "app/db/password".to_string(),
        data: make_secret_data(&[("password", "s3cret")]),
        tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok(), "expected success, got: {:?}", result.err());

    let output = result.unwrap();
    assert_eq!(output.version, 1);

    // Verify the secret was persisted
    let stored = store.secrets.read().await;
    assert!(stored.contains_key("app/db/password"));
    assert_eq!(stored["app/db/password"].current_version, 1);
}

#[tokio::test]
async fn test_set_secret_success_updates_existing_secret() {
    let existing = make_secret("app/db/password", "password", "old-pass");
    let store = Arc::new(StubSecretStore::with_secrets(vec![existing]));
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = SetSecretUseCase::new(store.clone(), audit.clone(), publisher);

    let input = SetSecretInput {
        path: "app/db/password".to_string(),
        data: make_secret_data(&[("password", "new-pass")]),
        tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.version, 2);

    // Verify version was incremented
    let stored = store.secrets.read().await;
    assert_eq!(stored["app/db/password"].current_version, 2);
    assert_eq!(stored["app/db/password"].versions.len(), 2);
}

#[tokio::test]
async fn test_set_secret_records_audit_log_on_success() {
    let store = Arc::new(StubSecretStore::new());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = SetSecretUseCase::new(store, audit.clone(), publisher);

    let input = SetSecretInput {
        path: "app/api/key".to_string(),
        data: make_secret_data(&[("key", "abc123")]),
        tenant_id: None,
    };

    let _ = uc.execute(&input).await;

    let logs = audit.recorded_logs().await;
    assert!(
        !logs.is_empty(),
        "expected audit log to be recorded on success"
    );
    assert_eq!(logs[0].path, "app/api/key");
    assert_eq!(logs[0].action, AccessAction::Write);
    assert!(logs[0].success);
}

#[tokio::test]
async fn test_set_secret_store_error_returns_internal() {
    let store = Arc::new(StubSecretStore::with_error());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = SetSecretUseCase::new(store, audit.clone(), publisher);

    let input = SetSecretInput {
        path: "app/db/password".to_string(),
        data: make_secret_data(&[("password", "s3cret")]),
        tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        SetSecretError::Internal(msg) => {
            assert!(msg.contains("unavailable"), "unexpected msg: {}", msg);
        }
    }
}

#[tokio::test]
async fn test_set_secret_records_audit_log_on_failure() {
    let store = Arc::new(StubSecretStore::with_error());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = SetSecretUseCase::new(store, audit.clone(), publisher);

    let input = SetSecretInput {
        path: "app/db/password".to_string(),
        data: make_secret_data(&[("password", "s3cret")]),
        tenant_id: None,
    };

    let _ = uc.execute(&input).await;

    let logs = audit.recorded_logs().await;
    assert!(
        !logs.is_empty(),
        "expected audit log to be recorded even on failure"
    );
    assert!(!logs[0].success);
    assert!(logs[0].error_msg.is_some());
}

#[tokio::test]
async fn test_set_secret_with_multiple_key_value_pairs() {
    let store = Arc::new(StubSecretStore::new());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = SetSecretUseCase::new(store.clone(), audit, publisher);

    let input = SetSecretInput {
        path: "app/config".to_string(),
        data: make_secret_data(&[
            ("db_host", "localhost"),
            ("db_port", "5432"),
            ("db_name", "mydb"),
        ]),
        tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let stored = store.secrets.read().await;
    let secret = &stored["app/config"];
    let version_data = &secret.versions[0].value.data;
    assert_eq!(version_data.len(), 3);
    assert_eq!(version_data["db_host"], "localhost");
    assert_eq!(version_data["db_port"], "5432");
    assert_eq!(version_data["db_name"], "mydb");
}

// ============================================================================
// GetSecretUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_get_secret_success() {
    let secret = make_secret("app/db/password", "password", "s3cret");
    let store = Arc::new(StubSecretStore::with_secrets(vec![secret]));
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = GetSecretUseCase::new(store, audit.clone(), publisher);

    let input = GetSecretInput {
        path: "app/db/password".to_string(),
        version: None,
    tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let secret = result.unwrap();
    assert_eq!(secret.path, "app/db/password");
    assert_eq!(secret.versions[0].value.data["password"], "s3cret");

    // Verify audit log
    let logs = audit.recorded_logs().await;
    assert!(!logs.is_empty());
    assert_eq!(logs[0].action, AccessAction::Read);
    assert!(logs[0].success);
}

#[tokio::test]
async fn test_get_secret_not_found() {
    let store = Arc::new(StubSecretStore::new());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = GetSecretUseCase::new(store, audit.clone(), publisher);

    let input = GetSecretInput {
        path: "nonexistent/path".to_string(),
        version: None,
    tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        GetSecretError::NotFound(path) => assert_eq!(path, "nonexistent/path"),
        e => panic!("expected NotFound, got: {:?}", e),
    }

    // Verify failure audit log
    let logs = audit.recorded_logs().await;
    assert!(!logs.is_empty());
    assert!(!logs[0].success);
}

#[tokio::test]
async fn test_get_secret_internal_error() {
    let store = Arc::new(StubSecretStore::with_error());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = GetSecretUseCase::new(store, audit, publisher);

    let input = GetSecretInput {
        path: "app/db/password".to_string(),
        version: None,
    tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        GetSecretError::Internal(msg) => {
            assert!(msg.contains("unavailable"), "unexpected msg: {}", msg);
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_get_secret_records_audit_on_failure() {
    let store = Arc::new(StubSecretStore::new()); // empty store => not found
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = GetSecretUseCase::new(store, audit.clone(), publisher);

    let input = GetSecretInput {
        path: "missing".to_string(),
        version: None,
    tenant_id: None,
    };

    let _ = uc.execute(&input).await;

    let logs = audit.recorded_logs().await;
    assert!(!logs.is_empty());
    assert_eq!(logs[0].path, "missing");
    assert!(!logs[0].success);
    assert!(logs[0].error_msg.is_some());
}

// ============================================================================
// DeleteSecretUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_delete_secret_success() {
    let secret = make_secret("app/db/password", "password", "s3cret");
    let store = Arc::new(StubSecretStore::with_secrets(vec![secret]));
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = DeleteSecretUseCase::new(store.clone(), audit.clone(), publisher);

    let input = DeleteSecretInput {
        path: "app/db/password".to_string(),
        versions: vec![1],
    tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    // Verify the secret was removed
    let stored = store.secrets.read().await;
    assert!(!stored.contains_key("app/db/password"));

    // Verify audit log
    let logs = audit.recorded_logs().await;
    assert!(!logs.is_empty());
    assert_eq!(logs[0].action, AccessAction::Delete);
    assert!(logs[0].success);
}

#[tokio::test]
async fn test_delete_secret_not_found() {
    let store = Arc::new(StubSecretStore::new());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = DeleteSecretUseCase::new(store, audit.clone(), publisher);

    let input = DeleteSecretInput {
        path: "nonexistent".to_string(),
        versions: vec![],
    tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        k1s0_vault_server::usecase::delete_secret::DeleteSecretError::NotFound(path) => {
            assert_eq!(path, "nonexistent");
        }
        e => panic!("expected NotFound, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_delete_secret_internal_error() {
    let store = Arc::new(StubSecretStore::with_error());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = DeleteSecretUseCase::new(store, audit, publisher);

    let input = DeleteSecretInput {
        path: "app/db/password".to_string(),
        versions: vec![1],
    tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        k1s0_vault_server::usecase::delete_secret::DeleteSecretError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_delete_secret_records_audit_on_failure() {
    let store = Arc::new(StubSecretStore::new());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let uc = DeleteSecretUseCase::new(store, audit.clone(), publisher);

    let input = DeleteSecretInput {
        path: "missing".to_string(),
        versions: vec![1],
    tenant_id: None,
    };

    let _ = uc.execute(&input).await;

    let logs = audit.recorded_logs().await;
    assert!(!logs.is_empty());
    assert!(!logs[0].success);
    assert!(logs[0].error_msg.is_some());
}

// ============================================================================
// ListSecretsUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_list_secrets_success_multiple() {
    let secrets = vec![
        make_secret("app/db/password", "password", "s3cret"),
        make_secret("app/api/key", "key", "abc123"),
        make_secret("infra/cache/token", "token", "xyz"),
    ];
    let store = Arc::new(StubSecretStore::with_secrets(secrets));

    let uc = ListSecretsUseCase::new(store);

    let result = uc.execute("app/").await;
    assert!(result.is_ok());

    let mut paths = result.unwrap();
    paths.sort();
    assert_eq!(paths.len(), 2);
    assert!(paths.contains(&"app/api/key".to_string()));
    assert!(paths.contains(&"app/db/password".to_string()));
}

#[tokio::test]
async fn test_list_secrets_empty_result() {
    let store = Arc::new(StubSecretStore::new());

    let uc = ListSecretsUseCase::new(store);

    let result = uc.execute("nonexistent/").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_list_secrets_all_with_empty_prefix() {
    let secrets = vec![make_secret("a/b", "k", "v"), make_secret("c/d", "k", "v")];
    let store = Arc::new(StubSecretStore::with_secrets(secrets));

    let uc = ListSecretsUseCase::new(store);

    // Empty prefix matches nothing because paths don't start with ""
    // Actually they do start with "" in starts_with, so all should match
    let result = uc.execute("").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

#[tokio::test]
async fn test_list_secrets_store_error() {
    let store = Arc::new(StubSecretStore::with_error());

    let uc = ListSecretsUseCase::new(store);

    let result = uc.execute("app/").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        k1s0_vault_server::usecase::list_secrets::ListSecretsError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
    }
}

// ============================================================================
// ListAuditLogsUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_list_audit_logs_success() {
    let logs = vec![
        SecretAccessLog::new(
            "app/db".to_string(),
            AccessAction::Read,
            Some("user-1".to_string()),
            true,
        ),
        SecretAccessLog::new(
            "app/api".to_string(),
            AccessAction::Write,
            Some("user-2".to_string()),
            true,
        ),
    ];
    let repo = Arc::new(StubAccessLogRepository::with_logs(logs));

    let uc = ListAuditLogsUseCase::new(repo);

    // LOW-12 監査対応: keyset ページネーション — after_id=None で先頭ページを取得する
    let result = uc
        .execute(&ListAuditLogsInput {
            after_id: None,
            limit: 20,
        })
        .await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.logs.len(), 2);
    assert!(output.next_cursor.is_none());
}

#[tokio::test]
async fn test_list_audit_logs_empty() {
    let repo = Arc::new(StubAccessLogRepository::new());

    let uc = ListAuditLogsUseCase::new(repo);

    let result = uc
        .execute(&ListAuditLogsInput {
            after_id: None,
            limit: 20,
        })
        .await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.logs.is_empty());
    assert!(output.next_cursor.is_none());
}

#[tokio::test]
async fn test_list_audit_logs_with_keyset_pagination() {
    // LOW-12 監査対応: keyset ページネーションのテスト。
    // カーソル（next_cursor）を使って次のページを取得することを検証する。
    let logs = vec![
        SecretAccessLog::new("path/1".to_string(), AccessAction::Read, None, true),
        SecretAccessLog::new("path/2".to_string(), AccessAction::Write, None, true),
        SecretAccessLog::new("path/3".to_string(), AccessAction::Delete, None, true),
        SecretAccessLog::new("path/4".to_string(), AccessAction::List, None, true),
        SecretAccessLog::new("path/5".to_string(), AccessAction::Read, None, false),
    ];
    let repo = Arc::new(StubAccessLogRepository::with_logs(logs));

    let uc = ListAuditLogsUseCase::new(repo);

    // 先頭ページ: after_id=None, limit=2
    let result = uc
        .execute(&ListAuditLogsInput {
            after_id: None,
            limit: 2,
        })
        .await;
    assert!(result.is_ok());
    let page1 = result.unwrap();
    assert_eq!(page1.logs.len(), 2);
    assert_eq!(page1.logs[0].path, "path/1");
    assert_eq!(page1.logs[1].path, "path/2");
    // 次ページが存在するためカーソルが返される
    assert!(page1.next_cursor.is_some());

    // 2ページ目: 先頭ページのカーソルを使用する
    let result = uc
        .execute(&ListAuditLogsInput {
            after_id: page1.next_cursor,
            limit: 2,
        })
        .await;
    assert!(result.is_ok());
    let page2 = result.unwrap();
    assert_eq!(page2.logs.len(), 2);
    assert_eq!(page2.logs[0].path, "path/3");
    assert_eq!(page2.logs[1].path, "path/4");
    assert!(page2.next_cursor.is_some());

    // 最終ページ: 残り1件のみ
    let result = uc
        .execute(&ListAuditLogsInput {
            after_id: page2.next_cursor,
            limit: 2,
        })
        .await;
    assert!(result.is_ok());
    let page3 = result.unwrap();
    assert_eq!(page3.logs.len(), 1);
    assert_eq!(page3.logs[0].path, "path/5");
    // 次ページなし
    assert!(page3.next_cursor.is_none());
}

#[tokio::test]
async fn test_list_audit_logs_repository_error() {
    let repo = Arc::new(StubAccessLogRepository::with_error());

    let uc = ListAuditLogsUseCase::new(repo);

    let result = uc
        .execute(&ListAuditLogsInput {
            after_id: None,
            limit: 20,
        })
        .await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("audit log backend unavailable"));
}

// ============================================================================
// RotateSecretUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_rotate_secret_success() {
    let secret = make_secret("app/db/password", "password", "old-pass");
    let store = Arc::new(StubSecretStore::with_secrets(vec![secret]));
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(StubEventPublisher::new());

    let get_uc = Arc::new(GetSecretUseCase::new(
        store.clone(),
        audit.clone(),
        Arc::new(NoopVaultEventPublisher),
    ));
    let set_uc = Arc::new(SetSecretUseCase::new(
        store.clone(),
        audit.clone(),
        Arc::new(NoopVaultEventPublisher),
    ));

    let uc = RotateSecretUseCase::new(get_uc, set_uc, publisher.clone());

    let input = RotateSecretInput {
        path: "app/db/password".to_string(),
        data: make_secret_data(&[("password", "new-rotated-pass")]),
        tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok(), "expected success, got: {:?}", result.err());

    let output = result.unwrap();
    assert_eq!(output.path, "app/db/password");
    assert_eq!(output.new_version, 2);
    assert!(output.rotated);

    // Verify the store was updated
    let stored = store.secrets.read().await;
    assert_eq!(stored["app/db/password"].current_version, 2);

    // Verify rotation event was published
    let events = publisher.rotated_events.read().await;
    assert_eq!(events.len(), 1);
    assert!(events[0].contains("app/db/password"));
}

#[tokio::test]
async fn test_rotate_secret_not_found() {
    let store = Arc::new(StubSecretStore::new());
    let audit = Arc::new(StubAccessLogRepository::new());

    let get_uc = Arc::new(GetSecretUseCase::new(
        store.clone(),
        audit.clone(),
        Arc::new(NoopVaultEventPublisher),
    ));
    let set_uc = Arc::new(SetSecretUseCase::new(
        store.clone(),
        audit.clone(),
        Arc::new(NoopVaultEventPublisher),
    ));

    let uc = RotateSecretUseCase::new(get_uc, set_uc, Arc::new(NoopVaultEventPublisher));

    let input = RotateSecretInput {
        path: "nonexistent".to_string(),
        data: make_secret_data(&[("key", "value")]),
        tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        RotateSecretError::NotFound(path) => assert_eq!(path, "nonexistent"),
        e => panic!("expected NotFound, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_rotate_secret_event_publish_failure() {
    let secret = make_secret("app/db/password", "password", "old-pass");
    let store = Arc::new(StubSecretStore::with_secrets(vec![secret]));
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(StubEventPublisher::with_error());

    let get_uc = Arc::new(GetSecretUseCase::new(
        store.clone(),
        audit.clone(),
        Arc::new(NoopVaultEventPublisher),
    ));
    let set_uc = Arc::new(SetSecretUseCase::new(
        store.clone(),
        audit.clone(),
        Arc::new(NoopVaultEventPublisher),
    ));

    let uc = RotateSecretUseCase::new(get_uc, set_uc, publisher);

    let input = RotateSecretInput {
        path: "app/db/password".to_string(),
        data: make_secret_data(&[("password", "new-pass")]),
        tenant_id: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        RotateSecretError::Internal(msg) => {
            assert!(
                msg.contains("event publish failed"),
                "unexpected msg: {}",
                msg
            );
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

// ============================================================================
// Cross-cutting / Integration-style usecase tests
// ============================================================================

#[tokio::test]
async fn test_set_then_get_secret_roundtrip() {
    let store = Arc::new(StubSecretStore::new());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let set_uc = SetSecretUseCase::new(store.clone(), audit.clone(), publisher.clone());
    let get_uc = GetSecretUseCase::new(store.clone(), audit.clone(), publisher.clone());

    // Set
    let set_input = SetSecretInput {
        path: "app/db/password".to_string(),
        data: make_secret_data(&[("password", "roundtrip-value")]),
        tenant_id: None,
    };
    let set_result = set_uc.execute(&set_input).await;
    assert!(set_result.is_ok());
    assert_eq!(set_result.unwrap().version, 1);

    // Get
    let get_input = GetSecretInput {
        path: "app/db/password".to_string(),
        version: None,
    tenant_id: None,
    };
    let get_result = get_uc.execute(&get_input).await;
    assert!(get_result.is_ok());

    let secret = get_result.unwrap();
    assert_eq!(secret.path, "app/db/password");
    assert_eq!(secret.versions[0].value.data["password"], "roundtrip-value");
}

#[tokio::test]
async fn test_set_then_delete_then_get_returns_not_found() {
    let store = Arc::new(StubSecretStore::new());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let set_uc = SetSecretUseCase::new(store.clone(), audit.clone(), publisher.clone());
    let delete_uc = DeleteSecretUseCase::new(store.clone(), audit.clone(), publisher.clone());
    let get_uc = GetSecretUseCase::new(store.clone(), audit.clone(), publisher.clone());

    // Set
    let set_input = SetSecretInput {
        path: "app/temp/secret".to_string(),
        data: make_secret_data(&[("key", "value")]),
        tenant_id: None,
    };
    assert!(set_uc.execute(&set_input).await.is_ok());

    // Delete
    let delete_input = DeleteSecretInput {
        path: "app/temp/secret".to_string(),
        versions: vec![1],
    tenant_id: None,
    };
    assert!(delete_uc.execute(&delete_input).await.is_ok());

    // Get should fail
    let get_input = GetSecretInput {
        path: "app/temp/secret".to_string(),
        version: None,
    tenant_id: None,
    };
    let result = get_uc.execute(&get_input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        GetSecretError::NotFound(_) => {}
        e => panic!("expected NotFound, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_list_after_multiple_sets() {
    let store = Arc::new(StubSecretStore::new());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let set_uc = SetSecretUseCase::new(store.clone(), audit.clone(), publisher.clone());
    let list_uc = ListSecretsUseCase::new(store.clone());

    // Set multiple secrets
    for name in &["db/password", "db/host", "api/key", "cache/token"] {
        let input = SetSecretInput {
            path: format!("app/{}", name),
            data: make_secret_data(&[("value", "test")]),
            tenant_id: None,
        };
        assert!(set_uc.execute(&input).await.is_ok());
    }

    // List db secrets
    let result = list_uc.execute("app/db/").await;
    assert!(result.is_ok());
    let mut paths = result.unwrap();
    paths.sort();
    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0], "app/db/host");
    assert_eq!(paths[1], "app/db/password");

    // List all app secrets
    let result = list_uc.execute("app/").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 4);
}

#[tokio::test]
async fn test_audit_trail_across_operations() {
    let store = Arc::new(StubSecretStore::new());
    let audit = Arc::new(StubAccessLogRepository::new());
    let publisher = Arc::new(NoopVaultEventPublisher);

    let set_uc = SetSecretUseCase::new(store.clone(), audit.clone(), publisher.clone());
    let get_uc = GetSecretUseCase::new(store.clone(), audit.clone(), publisher.clone());
    let delete_uc = DeleteSecretUseCase::new(store.clone(), audit.clone(), publisher.clone());

    // Set
    let _ = set_uc
        .execute(&SetSecretInput {
            path: "app/key".to_string(),
            data: make_secret_data(&[("k", "v")]),
            tenant_id: None,
        })
        .await;

    // Get
    let _ = get_uc
        .execute(&GetSecretInput {
            path: "app/key".to_string(),
            version: None,
        tenant_id: None,
        })
        .await;

    // Delete
    let _ = delete_uc
        .execute(&DeleteSecretInput {
            path: "app/key".to_string(),
            versions: vec![1],
        tenant_id: None,
        })
        .await;

    let logs = audit.recorded_logs().await;
    // set produces a Write log, get produces a Read log, delete produces a Delete log
    assert!(
        logs.len() >= 3,
        "expected at least 3 audit logs, got {}",
        logs.len()
    );

    let actions: Vec<&AccessAction> = logs.iter().map(|l| &l.action).collect();
    assert!(actions.contains(&&AccessAction::Write));
    assert!(actions.contains(&&AccessAction::Read));
    assert!(actions.contains(&&AccessAction::Delete));
}
