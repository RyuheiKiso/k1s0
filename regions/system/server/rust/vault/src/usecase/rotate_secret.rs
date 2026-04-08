use std::collections::HashMap;
use std::sync::Arc;

use crate::infrastructure::kafka_producer::{VaultEventPublisher, VaultSecretRotatedEvent};
use crate::usecase::get_secret::{GetSecretError, GetSecretInput, GetSecretUseCase};
use crate::usecase::set_secret::{SetSecretError, SetSecretInput, SetSecretUseCase};

/// MED-011 対応: `tenant_id` をアクセスログに記録するために追加。
#[derive(Debug, Clone)]
pub struct RotateSecretInput {
    pub path: String,
    pub data: HashMap<String, String>,
    /// gRPC 層で Claims から抽出したテナント ID。get/set アクセスログに伝播する。
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RotateSecretOutput {
    pub path: String,
    pub new_version: i64,
    pub rotated: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum RotateSecretError {
    #[error("secret not found: {0}")]
    NotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct RotateSecretUseCase {
    get_secret_uc: Arc<GetSecretUseCase>,
    set_secret_uc: Arc<SetSecretUseCase>,
    event_publisher: Arc<dyn VaultEventPublisher>,
}

impl RotateSecretUseCase {
    pub fn new(
        get_secret_uc: Arc<GetSecretUseCase>,
        set_secret_uc: Arc<SetSecretUseCase>,
        event_publisher: Arc<dyn VaultEventPublisher>,
    ) -> Self {
        Self {
            get_secret_uc,
            set_secret_uc,
            event_publisher,
        }
    }

    pub async fn execute(
        &self,
        input: &RotateSecretInput,
    ) -> Result<RotateSecretOutput, RotateSecretError> {
        // MED-011 対応: tenant_id を get/set アクセスログの両方に伝播する。
        let current = self
            .get_secret_uc
            .execute(&GetSecretInput {
                path: input.path.clone(),
                version: None,
                tenant_id: input.tenant_id.clone(),
            })
            .await
            .map_err(|e| match e {
                GetSecretError::NotFound(path) => RotateSecretError::NotFound(path),
                GetSecretError::Internal(msg) => RotateSecretError::Internal(msg),
            })?;

        let new_version = self
            .set_secret_uc
            .execute(&SetSecretInput {
                path: input.path.clone(),
                data: input.data.clone(),
                tenant_id: input.tenant_id.clone(),
            })
            .await
            .map_err(|e| match e {
                SetSecretError::Internal(msg) => RotateSecretError::Internal(msg),
            })?
            .version;

        if new_version <= current.current_version {
            return Err(RotateSecretError::Internal(format!(
                "invalid rotated version: {} <= {}",
                new_version, current.current_version
            )));
        }

        self.event_publisher
            .publish_secret_rotated(&VaultSecretRotatedEvent {
                key_path: input.path.clone(),
                old_version: current.current_version,
                new_version,
                actor_id: "system".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
            .await
            .map_err(|e| RotateSecretError::Internal(e.to_string()))?;

        Ok(RotateSecretOutput {
            path: input.path.clone(),
            new_version,
            rotated: true,
        })
    }
}
