use std::sync::Arc;

use crate::domain::entity::api_registration::ApiSchemaVersion;
use crate::domain::repository::ApiSchemaVersionRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetSchemaVersionError {
    #[error("schema version not found: {name}@{version}")]
    NotFound { name: String, version: u32 },
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetSchemaVersionUseCase {
    version_repo: Arc<dyn ApiSchemaVersionRepository>,
}

impl GetSchemaVersionUseCase {
    pub fn new(version_repo: Arc<dyn ApiSchemaVersionRepository>) -> Self {
        Self { version_repo }
    }

    pub async fn execute(
        &self,
        name: &str,
        version: u32,
    ) -> Result<ApiSchemaVersion, GetSchemaVersionError> {
        self.version_repo
            .find_by_name_and_version(name, version)
            .await
            .map_err(|e| GetSchemaVersionError::Internal(e.to_string()))?
            .ok_or_else(|| GetSchemaVersionError::NotFound {
                name: name.to_string(),
                version,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::api_registration::SchemaType;
    use crate::domain::repository::api_repository::MockApiSchemaVersionRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockApiSchemaVersionRepository::new();
        mock.expect_find_by_name_and_version()
            .returning(|_, _| {
                Ok(Some(ApiSchemaVersion::new(
                    "test-api".to_string(),
                    2,
                    SchemaType::OpenApi,
                    "openapi: 3.0.3".to_string(),
                    "user-001".to_string(),
                )))
            });

        let uc = GetSchemaVersionUseCase::new(Arc::new(mock));
        let result = uc.execute("test-api", 2).await;
        assert!(result.is_ok());
        let version = result.unwrap();
        assert_eq!(version.name, "test-api");
        assert_eq!(version.version, 2);
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockApiSchemaVersionRepository::new();
        mock.expect_find_by_name_and_version()
            .returning(|_, _| Ok(None));

        let uc = GetSchemaVersionUseCase::new(Arc::new(mock));
        let result = uc.execute("test-api", 99).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetSchemaVersionError::NotFound { name, version } => {
                assert_eq!(name, "test-api");
                assert_eq!(version, 99);
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn error() {
        let mut mock = MockApiSchemaVersionRepository::new();
        mock.expect_find_by_name_and_version()
            .returning(|_, _| Err(anyhow::anyhow!("db error")));

        let uc = GetSchemaVersionUseCase::new(Arc::new(mock));
        let result = uc.execute("test-api", 1).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetSchemaVersionError::Internal(msg) => assert!(msg.contains("db error")),
            _ => panic!("Expected Internal error"),
        }
    }
}
