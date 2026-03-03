use std::collections::HashMap;
use std::sync::Arc;

use crate::usecase::get_secret::{GetSecretError, GetSecretInput, GetSecretUseCase};
use crate::usecase::set_secret::{SetSecretError, SetSecretInput, SetSecretUseCase};

#[derive(Debug, Clone)]
pub struct RotateSecretInput {
    pub path: String,
    pub data: HashMap<String, String>,
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
}

impl RotateSecretUseCase {
    pub fn new(get_secret_uc: Arc<GetSecretUseCase>, set_secret_uc: Arc<SetSecretUseCase>) -> Self {
        Self {
            get_secret_uc,
            set_secret_uc,
        }
    }

    pub async fn execute(&self, input: &RotateSecretInput) -> Result<RotateSecretOutput, RotateSecretError> {
        let current = self
            .get_secret_uc
            .execute(&GetSecretInput {
                path: input.path.clone(),
                version: None,
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
            })
            .await
            .map_err(|e| match e {
                SetSecretError::Internal(msg) => RotateSecretError::Internal(msg),
            })?;

        if new_version <= current.current_version {
            return Err(RotateSecretError::Internal(format!(
                "invalid rotated version: {} <= {}",
                new_version, current.current_version
            )));
        }

        Ok(RotateSecretOutput {
            path: input.path.clone(),
            new_version,
            rotated: true,
        })
    }
}
