use std::collections::HashMap;
use std::sync::Arc;

use crate::usecase::delete_secret::{DeleteSecretError, DeleteSecretInput, DeleteSecretUseCase};
use crate::usecase::get_secret::{GetSecretError, GetSecretInput, GetSecretUseCase};
use crate::usecase::list_secrets::ListSecretsUseCase;
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
}

#[derive(Debug, Clone)]
pub struct SetSecretRequest {
    pub path: String,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SetSecretResponse {
    pub version: i64,
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

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("internal: {0}")]
    Internal(String),
}

// --- VaultGrpcService ---

pub struct VaultGrpcService {
    get_secret_uc: Arc<GetSecretUseCase>,
    set_secret_uc: Arc<SetSecretUseCase>,
    delete_secret_uc: Arc<DeleteSecretUseCase>,
    list_secrets_uc: Arc<ListSecretsUseCase>,
}

impl VaultGrpcService {
    pub fn new(
        get_secret_uc: Arc<GetSecretUseCase>,
        set_secret_uc: Arc<SetSecretUseCase>,
        delete_secret_uc: Arc<DeleteSecretUseCase>,
        list_secrets_uc: Arc<ListSecretsUseCase>,
    ) -> Self {
        Self {
            get_secret_uc,
            set_secret_uc,
            delete_secret_uc,
            list_secrets_uc,
        }
    }

    pub async fn get_secret(
        &self,
        req: GetSecretRequest,
    ) -> Result<GetSecretResponse, GrpcError> {
        let input = GetSecretInput {
            path: req.path.clone(),
            version: req.version,
        };

        match self.get_secret_uc.execute(&input).await {
            Ok(secret) => {
                let sv = secret
                    .get_version(req.version)
                    .ok_or_else(|| GrpcError::NotFound("version not found".to_string()))?;
                let version = sv.version;
                let data = sv.value.data.clone();
                Ok(GetSecretResponse {
                    path: secret.path,
                    version,
                    data,
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
        let input = SetSecretInput {
            path: req.path,
            data: req.data,
        };

        match self.set_secret_uc.execute(&input).await {
            Ok(version) => Ok(SetSecretResponse { version }),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::secret::Secret;
    use crate::domain::repository::access_log_repo::MockAccessLogRepository;
    use crate::domain::repository::secret_store::MockSecretStore;

    fn make_service(
        mock_store: MockSecretStore,
        mock_audit: MockAccessLogRepository,
    ) -> VaultGrpcService {
        let store = Arc::new(mock_store);
        let audit = Arc::new(mock_audit);

        VaultGrpcService::new(
            Arc::new(GetSecretUseCase::new(store.clone(), audit.clone())),
            Arc::new(SetSecretUseCase::new(store.clone(), audit.clone())),
            Arc::new(DeleteSecretUseCase::new(store.clone(), audit.clone())),
            Arc::new(ListSecretsUseCase::new(store)),
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
