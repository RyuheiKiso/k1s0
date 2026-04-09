use std::collections::HashMap;
use std::sync::Arc;

use crate::usecase::delete_secret::{DeleteSecretError, DeleteSecretInput, DeleteSecretUseCase};
use crate::usecase::get_secret::{GetSecretError, GetSecretInput, GetSecretUseCase};
use crate::usecase::list_audit_logs::{ListAuditLogsInput, ListAuditLogsUseCase};
use crate::usecase::list_secrets::ListSecretsUseCase;
use crate::usecase::rotate_secret::{RotateSecretError, RotateSecretInput, RotateSecretUseCase};
use crate::usecase::set_secret::{SetSecretInput, SetSecretUseCase};

// --- gRPC Request/Response Types (手動定義) ---

/// MED-011 対応: `tenant_id` を gRPC 層から use case 層（アクセスログ記録）へ伝播するために追加。
#[derive(Debug, Clone)]
pub struct GetSecretRequest {
    pub path: String,
    pub version: Option<i64>,
    /// gRPC ミドルウェアの Claims から抽出したテナント ID。アクセスログに記録する。
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GetSecretResponse {
    pub path: String,
    pub version: i64,
    pub data: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// MED-011 対応: `tenant_id` をアクセスログに伝播するために追加。
#[derive(Debug, Clone)]
pub struct SetSecretRequest {
    pub path: String,
    pub data: HashMap<String, String>,
    /// gRPC ミドルウェアの Claims から抽出したテナント ID。アクセスログに記録する。
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SetSecretResponse {
    pub path: String,
    pub version: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// MED-011 対応: `tenant_id` をアクセスログに伝播するために追加。
#[derive(Debug, Clone)]
pub struct RotateSecretRequest {
    pub path: String,
    pub data: HashMap<String, String>,
    /// gRPC ミドルウェアの Claims から抽出したテナント ID。アクセスログに記録する。
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RotateSecretResponse {
    pub path: String,
    pub new_version: i64,
    pub rotated: bool,
}

/// MED-011 対応: `tenant_id` をアクセスログに伝播するために追加。
#[derive(Debug, Clone)]
pub struct DeleteSecretRequest {
    pub path: String,
    pub versions: Vec<i64>,
    /// gRPC ミドルウェアの Claims から抽出したテナント ID。アクセスログに記録する。
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeleteSecretResponse {}

#[derive(Debug, Clone)]
pub struct ListSecretsRequest {
    pub prefix: String,
}

#[derive(Debug, Clone)]
pub struct ListSecretsResponse {
    pub keys: Vec<String>,
}

/// MED-011 対応: `tenant_id` をアクセスログに伝播するために追加。
#[derive(Debug, Clone)]
pub struct GetSecretMetadataRequest {
    pub path: String,
    /// gRPC ミドルウェアの Claims から抽出したテナント ID。アクセスログに記録する。
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GetSecretMetadataResponse {
    pub path: String,
    pub current_version: i64,
    pub version_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// LOW-12 監査対応: keyset ページネーションリクエスト。
#[derive(Debug, Clone)]
pub struct ListAuditLogsRequest {
    /// 前ページの最後のアイテムの id（カーソル）。None の場合は先頭ページ。
    pub after_id: Option<uuid::Uuid>,
    pub limit: u32,
}

#[derive(Debug, Clone)]
pub struct AuditLogEntry {
    pub id: String,
    pub key_path: String,
    pub action: String,
    pub actor_id: String,
    pub ip_address: String,
    pub success: bool,
    pub error_msg: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ListAuditLogsResponse {
    pub logs: Vec<AuditLogEntry>,
    /// LOW-12 監査対応: 次ページカーソル。None なら最終ページ。
    pub next_cursor: Option<uuid::Uuid>,
}

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("internal: {0}")]
    Internal(String),
}

// --- Key Path バリデーション ---

/// ADR-0109 対応: `key_path` がリクエスト元テナントのパスプレフィックスで始まることを検証する。
/// `tenant_id` が Some `で空でない場合、key_path` は必ず "{`tenant_id`}/" で始まらなければならない。
/// これにより RLS の代替として vault-db のテナント境界を application 層で強制する。
// Option<&str> の方が &Option<String> よりも慣用的（Clippy: ref_option）
fn validate_key_path_for_tenant(path: &str, tenant_id: Option<&str>) -> Result<(), GrpcError> {
    if let Some(tid) = tenant_id {
        if !tid.is_empty() {
            let expected_prefix = format!("{tid}/");
            if !path.starts_with(&expected_prefix) {
                tracing::error!(
                    key_path = %path,
                    tenant_id = %tid,
                    "key_path がテナントプレフィックスで始まっていません（ADR-0109 テナント分離違反）"
                );
                return Err(GrpcError::PermissionDenied(format!(
                    "key_path '{path}' はテナント '{tid}' のパスプレフィックス '{expected_prefix}' で始まる必要があります"
                )));
            }
        }
    }
    Ok(())
}

// --- VaultGrpcService ---

// ユースケースフィールドの命名規則として _uc サフィックスを使用する（アーキテクチャ上の意図的な設計）
#[allow(clippy::struct_field_names)]
pub struct VaultGrpcService {
    get_secret_uc: Arc<GetSecretUseCase>,
    set_secret_uc: Arc<SetSecretUseCase>,
    rotate_secret_uc: Arc<RotateSecretUseCase>,
    delete_secret_uc: Arc<DeleteSecretUseCase>,
    list_secrets_uc: Arc<ListSecretsUseCase>,
    list_audit_logs_uc: Arc<ListAuditLogsUseCase>,
}

impl VaultGrpcService {
    #[must_use]
    pub fn new(
        get_secret_uc: Arc<GetSecretUseCase>,
        set_secret_uc: Arc<SetSecretUseCase>,
        rotate_secret_uc: Arc<RotateSecretUseCase>,
        delete_secret_uc: Arc<DeleteSecretUseCase>,
        list_secrets_uc: Arc<ListSecretsUseCase>,
        list_audit_logs_uc: Arc<ListAuditLogsUseCase>,
    ) -> Self {
        Self {
            get_secret_uc,
            set_secret_uc,
            rotate_secret_uc,
            delete_secret_uc,
            list_secrets_uc,
            list_audit_logs_uc,
        }
    }

    pub async fn get_secret(&self, req: GetSecretRequest) -> Result<GetSecretResponse, GrpcError> {
        if req.path.trim().is_empty() {
            return Err(GrpcError::InvalidArgument("path is required".to_string()));
        }
        // ADR-0109 対応: key_path がテナントプレフィックスで始まることを検証する（RLS 代替措置）
        validate_key_path_for_tenant(&req.path, req.tenant_id.as_deref())?;
        // MED-011 対応: tonic_service で Claims から抽出した tenant_id をアクセスログ記録に使用する。
        let input = GetSecretInput {
            path: req.path.clone(),
            version: req.version,
            tenant_id: req.tenant_id.clone(),
        };

        match self.get_secret_uc.execute(&input).await {
            Ok(secret) => {
                let path = secret.path.clone();
                let sv = secret
                    .get_version(req.version)
                    .ok_or_else(|| GrpcError::NotFound("version not found".to_string()))?;
                let version = sv.version;
                let data = sv.value.data.clone();
                Ok(GetSecretResponse {
                    path,
                    version,
                    data,
                    created_at: sv.created_at,
                    updated_at: secret.updated_at,
                })
            }
            Err(GetSecretError::NotFound(path)) => Err(GrpcError::NotFound(path)),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn set_secret(&self, req: SetSecretRequest) -> Result<SetSecretResponse, GrpcError> {
        // ADR-0109 対応: key_path がテナントプレフィックスで始まることを検証する（RLS 代替措置）
        validate_key_path_for_tenant(&req.path, req.tenant_id.as_deref())?;
        let input = SetSecretInput {
            path: req.path,
            data: req.data,
            // MED-011 対応: tonic_service で Claims から抽出した tenant_id をアクセスログ記録に使用する。
            tenant_id: req.tenant_id,
        };

        match self.set_secret_uc.execute(&input).await {
            Ok(output) => Ok(SetSecretResponse {
                path: input.path,
                version: output.version,
                created_at: output.created_at,
            }),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn rotate_secret(
        &self,
        req: RotateSecretRequest,
    ) -> Result<RotateSecretResponse, GrpcError> {
        // ADR-0109 対応: key_path がテナントプレフィックスで始まることを検証する（RLS 代替措置）
        validate_key_path_for_tenant(&req.path, req.tenant_id.as_deref())?;
        let output = self
            .rotate_secret_uc
            .execute(&RotateSecretInput {
                path: req.path,
                data: req.data,
                // MED-011 対応: tonic_service で Claims から抽出した tenant_id をアクセスログ記録に使用する。
                tenant_id: req.tenant_id,
            })
            .await
            .map_err(|e| match e {
                RotateSecretError::NotFound(path) => GrpcError::NotFound(path),
                RotateSecretError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(RotateSecretResponse {
            path: output.path,
            new_version: output.new_version,
            rotated: output.rotated,
        })
    }

    pub async fn delete_secret(
        &self,
        req: DeleteSecretRequest,
    ) -> Result<DeleteSecretResponse, GrpcError> {
        // ADR-0109 対応: key_path がテナントプレフィックスで始まることを検証する（RLS 代替措置）
        validate_key_path_for_tenant(&req.path, req.tenant_id.as_deref())?;
        let input = DeleteSecretInput {
            path: req.path,
            versions: req.versions,
            // MED-011 対応: tonic_service で Claims から抽出した tenant_id をアクセスログ記録に使用する。
            tenant_id: req.tenant_id,
        };

        match self.delete_secret_uc.execute(&input).await {
            Ok(()) => Ok(DeleteSecretResponse {}),
            Err(DeleteSecretError::NotFound(path)) => Err(GrpcError::NotFound(path)),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn list_secrets(
        &self,
        req: ListSecretsRequest,
    ) -> Result<ListSecretsResponse, GrpcError> {
        match self.list_secrets_uc.execute(&req.prefix).await {
            Ok(keys) => Ok(ListSecretsResponse { keys }),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn get_secret_metadata(
        &self,
        req: GetSecretMetadataRequest,
    ) -> Result<GetSecretMetadataResponse, GrpcError> {
        // ADR-0109 対応: key_path がテナントプレフィックスで始まることを検証する（RLS 代替措置）
        validate_key_path_for_tenant(&req.path, req.tenant_id.as_deref())?;
        // MED-011 対応: tonic_service で Claims から抽出した tenant_id をアクセスログ記録に使用する。
        let secret = self
            .get_secret_uc
            .execute(&GetSecretInput {
                path: req.path.clone(),
                version: None,
                tenant_id: req.tenant_id.clone(),
            })
            .await
            .map_err(|e| match e {
                GetSecretError::NotFound(path) => GrpcError::NotFound(path),
                GetSecretError::Internal(_) => GrpcError::Internal(e.to_string()),
            })?;

        Ok(GetSecretMetadataResponse {
            path: secret.path,
            current_version: secret.current_version,
            // LOW-008: 安全な型変換（オーバーフロー防止）
            version_count: i32::try_from(secret.versions.len()).unwrap_or(i32::MAX),
            created_at: secret.created_at,
            updated_at: secret.updated_at,
        })
    }

    pub async fn list_audit_logs(
        &self,
        req: ListAuditLogsRequest,
    ) -> Result<ListAuditLogsResponse, GrpcError> {
        // LOW-12 監査対応: keyset ページネーション入力を構築する
        let input = ListAuditLogsInput {
            after_id: req.after_id,
            limit: req.limit.max(1),
        };

        let output = self
            .list_audit_logs_uc
            .execute(&input)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;

        let entries = output
            .logs
            .into_iter()
            .map(|log| {
                let action = match log.action {
                    crate::domain::entity::access_log::AccessAction::Read => "read",
                    crate::domain::entity::access_log::AccessAction::Write => "write",
                    crate::domain::entity::access_log::AccessAction::Delete => "delete",
                    crate::domain::entity::access_log::AccessAction::List => "list",
                }
                .to_string();

                AuditLogEntry {
                    id: log.id.to_string(),
                    key_path: log.path,
                    action,
                    actor_id: log.subject.unwrap_or_default(),
                    ip_address: log.ip_address.unwrap_or_default(),
                    success: log.success,
                    error_msg: log.error_msg,
                    created_at: log.created_at,
                }
            })
            .collect();

        Ok(ListAuditLogsResponse {
            logs: entries,
            next_cursor: output.next_cursor,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::secret::Secret;
    use crate::domain::repository::access_log_repo::MockAccessLogRepository;
    use crate::domain::repository::secret_store::MockSecretStore;
    use crate::infrastructure::kafka_producer::NoopVaultEventPublisher;

    fn make_service(
        mock_store: MockSecretStore,
        mock_audit: MockAccessLogRepository,
    ) -> VaultGrpcService {
        let store = Arc::new(mock_store);
        let audit = Arc::new(mock_audit);
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
        let rotate_uc = Arc::new(crate::usecase::RotateSecretUseCase::new(
            get_uc.clone(),
            set_uc.clone(),
            Arc::new(NoopVaultEventPublisher),
        ));

        VaultGrpcService::new(
            get_uc,
            set_uc,
            rotate_uc,
            Arc::new(DeleteSecretUseCase::new(
                store.clone(),
                audit.clone(),
                Arc::new(NoopVaultEventPublisher),
            )),
            Arc::new(ListSecretsUseCase::new(store)),
            Arc::new(ListAuditLogsUseCase::new(audit)),
        )
    }

    fn default_audit() -> MockAccessLogRepository {
        let mut mock = MockAccessLogRepository::new();
        mock.expect_record().returning(|_| Ok(()));
        mock
    }

    #[tokio::test]
    async fn test_get_secret_success() {
        let mut mock_store = MockSecretStore::new();
        // ADR-0109: key_path は "{tenant_id}/{remaining}" の形式である必要がある
        let data = HashMap::from([("password".to_string(), "s3cret".to_string())]);
        let secret = Secret::new("test-tenant/app/db".to_string(), data);

        mock_store
            .expect_get()
            .returning(move |_, _| Ok(secret.clone()));

        let svc = make_service(mock_store, default_audit());
        let resp = svc
            .get_secret(GetSecretRequest {
                path: "test-tenant/app/db".to_string(),
                version: None,
                tenant_id: Some("test-tenant".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(resp.path, "test-tenant/app/db");
        assert_eq!(resp.version, 1);
        assert_eq!(resp.data["password"], "s3cret");
    }

    #[tokio::test]
    async fn test_get_secret_invalid_tenant_prefix() {
        // ADR-0109: tenant_id と key_path のプレフィックスが一致しない場合は PermissionDenied を返す
        let mock_store = MockSecretStore::new();
        let svc = make_service(mock_store, default_audit());
        let result = svc
            .get_secret(GetSecretRequest {
                path: "other-tenant/app/db".to_string(),
                version: None,
                tenant_id: Some("test-tenant".to_string()),
            })
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::PermissionDenied(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_secret_not_found() {
        let mut mock_store = MockSecretStore::new();
        mock_store
            .expect_get()
            .returning(|_, _| Err(anyhow::anyhow!("secret not found")));

        let svc = make_service(mock_store, default_audit());
        let result = svc
            .get_secret(GetSecretRequest {
                path: "nonexistent".to_string(),
                version: None,
                tenant_id: None,
            })
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_set_secret_success() {
        let mut mock_store = MockSecretStore::new();
        mock_store.expect_set().returning(|_, _| Ok(2));
        // ADR-0109: key_path は "{tenant_id}/{remaining}" の形式である必要がある
        mock_store
            .expect_get()
            .withf(|path, version| path == "test-tenant/app/db" && *version == Some(2))
            .returning(|path, _| {
                Ok(Secret::new(
                    path.to_string(),
                    HashMap::from([("password".to_string(), "new".to_string())]),
                )
                .update(HashMap::from([(
                    "password".to_string(),
                    "newer".to_string(),
                )])))
            });

        let svc = make_service(mock_store, default_audit());
        let resp = svc
            .set_secret(SetSecretRequest {
                path: "test-tenant/app/db".to_string(),
                data: HashMap::from([("password".to_string(), "new".to_string())]),
                tenant_id: Some("test-tenant".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(resp.path, "test-tenant/app/db");
        assert_eq!(resp.version, 2);
    }

    #[tokio::test]
    async fn test_delete_secret_success() {
        let mut mock_store = MockSecretStore::new();
        mock_store.expect_delete().returning(|_, _| Ok(()));

        let svc = make_service(mock_store, default_audit());
        // ADR-0109: key_path は "{tenant_id}/{remaining}" の形式である必要がある
        let result = svc
            .delete_secret(DeleteSecretRequest {
                path: "test-tenant/app/db".to_string(),
                versions: vec![1],
                tenant_id: Some("test-tenant".to_string()),
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_secret_not_found() {
        let mut mock_store = MockSecretStore::new();
        mock_store
            .expect_delete()
            .returning(|_, _| Err(anyhow::anyhow!("secret not found")));

        let svc = make_service(mock_store, default_audit());
        let result = svc
            .delete_secret(DeleteSecretRequest {
                path: "nonexistent".to_string(),
                versions: vec![],
                tenant_id: None,
            })
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_list_secrets_success() {
        let mut mock_store = MockSecretStore::new();
        mock_store
            .expect_list()
            .returning(|_| Ok(vec!["app/db".to_string(), "app/api".to_string()]));

        let svc = make_service(mock_store, default_audit());
        let resp = svc
            .list_secrets(ListSecretsRequest {
                prefix: "app/".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(resp.keys.len(), 2);
        assert!(resp.keys.contains(&"app/db".to_string()));
        assert!(resp.keys.contains(&"app/api".to_string()));
    }
}
