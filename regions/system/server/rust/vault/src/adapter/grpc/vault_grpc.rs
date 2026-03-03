use std::collections::HashMap;
use std::sync::Arc;

use crate::usecase::delete_secret::{DeleteSecretError, DeleteSecretInput, DeleteSecretUseCase};
use crate::usecase::get_secret::{GetSecretError, GetSecretInput, GetSecretUseCase};
use crate::usecase::list_audit_logs::{ListAuditLogsInput, ListAuditLogsUseCase};
use crate::usecase::list_secrets::ListSecretsUseCase;
use crate::usecase::rotate_secret::{RotateSecretError, RotateSecretInput, RotateSecretUseCase};
use crate::usecase::set_secret::{SetSecretInput, SetSecretUseCase};

// --- gRPC Request/Response Types (手動定義) ---

#[derive(Debug, Clone)]
pub struct GetSecretRequest {
    pub path: String,
    pub version: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct GetSecretResponse {
    pub path: String,
    pub version: i64,
    pub data: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct SetSecretRequest {
    pub path: String,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SetSecretResponse {
    pub version: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct RotateSecretRequest {
    pub path: String,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RotateSecretResponse {
    pub path: String,
    pub new_version: i64,
    pub rotated: bool,
}

#[derive(Debug, Clone)]
pub struct DeleteSecretRequest {
    pub path: String,
    pub versions: Vec<i64>,
}

#[derive(Debug, Clone)]
pub struct DeleteSecretResponse {}

#[derive(Debug, Clone)]
pub struct ListSecretsRequest {
    pub path_prefix: String,
}

#[derive(Debug, Clone)]
pub struct ListSecretsResponse {
    pub keys: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GetSecretMetadataRequest {
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct GetSecretMetadataResponse {
    pub path: String,
    pub current_version: i64,
    pub version_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ListAuditLogsRequest {
    pub offset: i32,
    pub limit: i32,
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

// --- VaultGrpcService ---

pub struct VaultGrpcService {
    get_secret_uc: Arc<GetSecretUseCase>,
    set_secret_uc: Arc<SetSecretUseCase>,
    rotate_secret_uc: Arc<RotateSecretUseCase>,
    delete_secret_uc: Arc<DeleteSecretUseCase>,
    list_secrets_uc: Arc<ListSecretsUseCase>,
    list_audit_logs_uc: Arc<ListAuditLogsUseCase>,
}

impl VaultGrpcService {
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

    pub async fn get_secret(
        &self,
        req: GetSecretRequest,
    ) -> Result<GetSecretResponse, GrpcError> {
        if req.path.trim().is_empty() {
            return Err(GrpcError::InvalidArgument("path is required".to_string()));
        }
        let input = GetSecretInput {
            path: req.path.clone(),
            version: req.version,
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

    pub async fn set_secret(
        &self,
        req: SetSecretRequest,
    ) -> Result<SetSecretResponse, GrpcError> {
        let path = req.path.clone();
        let input = SetSecretInput {
            path: req.path,
            data: req.data,
        };

        match self.set_secret_uc.execute(&input).await {
            Ok(version) => {
                let secret = self
                    .get_secret_uc
                    .execute(&GetSecretInput {
                        path,
                        version: Some(version),
                    })
                    .await
                    .map_err(|e| GrpcError::Internal(e.to_string()))?;
                let created_at = secret
                    .get_version(Some(version))
                    .map(|sv| sv.created_at)
                    .unwrap_or_else(chrono::Utc::now);
                Ok(SetSecretResponse {
                    version,
                    created_at,
                })
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn rotate_secret(
        &self,
        req: RotateSecretRequest,
    ) -> Result<RotateSecretResponse, GrpcError> {
        let output = self
            .rotate_secret_uc
            .execute(&RotateSecretInput {
                path: req.path,
                data: req.data,
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
        let input = DeleteSecretInput {
            path: req.path,
            versions: req.versions,
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
        match self.list_secrets_uc.execute(&req.path_prefix).await {
            Ok(keys) => Ok(ListSecretsResponse { keys }),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn get_secret_metadata(
        &self,
        req: GetSecretMetadataRequest,
    ) -> Result<GetSecretMetadataResponse, GrpcError> {
        let secret = self
            .get_secret_uc
            .execute(&GetSecretInput {
                path: req.path.clone(),
                version: None,
            })
            .await
            .map_err(|e| match e {
                GetSecretError::NotFound(path) => GrpcError::NotFound(path),
                _ => GrpcError::Internal(e.to_string()),
            })?;

        Ok(GetSecretMetadataResponse {
            path: secret.path,
            current_version: secret.current_version,
            version_count: secret.versions.len() as i32,
            created_at: secret.created_at,
            updated_at: secret.updated_at,
        })
    }

    pub async fn list_audit_logs(
        &self,
        req: ListAuditLogsRequest,
    ) -> Result<ListAuditLogsResponse, GrpcError> {
        let input = ListAuditLogsInput {
            offset: req.offset.max(0) as u32,
            limit: req.limit.max(1) as u32,
        };

        let logs = self
            .list_audit_logs_uc
            .execute(&input)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;

        let entries = logs
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

        Ok(ListAuditLogsResponse { logs: entries })
    }
}

#[cfg(test)]
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
        let data = HashMap::from([("password".to_string(), "s3cret".to_string())]);
        let secret = Secret::new("app/db".to_string(), data);

        mock_store
            .expect_get()
            .returning(move |_, _| Ok(secret.clone()));

        let svc = make_service(mock_store, default_audit());
        let resp = svc
            .get_secret(GetSecretRequest {
                path: "app/db".to_string(),
                version: None,
            })
            .await
            .unwrap();

        assert_eq!(resp.path, "app/db");
        assert_eq!(resp.version, 1);
        assert_eq!(resp.data["password"], "s3cret");
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
        mock_store
            .expect_set()
            .returning(|_, _| Ok(2));
        mock_store
            .expect_get()
            .withf(|path, version| path == "app/db" && *version == Some(2))
            .returning(|path, _| {
                Ok(Secret::new(
                    path.to_string(),
                    HashMap::from([("password".to_string(), "new".to_string())]),
                )
                .update(HashMap::from([("password".to_string(), "newer".to_string())])))
            });

        let svc = make_service(mock_store, default_audit());
        let resp = svc
            .set_secret(SetSecretRequest {
                path: "app/db".to_string(),
                data: HashMap::from([("password".to_string(), "new".to_string())]),
            })
            .await
            .unwrap();

        assert_eq!(resp.version, 2);
    }

    #[tokio::test]
    async fn test_delete_secret_success() {
        let mut mock_store = MockSecretStore::new();
        mock_store
            .expect_delete()
            .returning(|_, _| Ok(()));

        let svc = make_service(mock_store, default_audit());
        let result = svc
            .delete_secret(DeleteSecretRequest {
                path: "app/db".to_string(),
                versions: vec![1],
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
                path_prefix: "app/".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(resp.keys.len(), 2);
        assert!(resp.keys.contains(&"app/db".to_string()));
        assert!(resp.keys.contains(&"app/api".to_string()));
    }
}
