use std::sync::Arc;

use crate::domain::entity::api_registration::CompatibilityResult;
use crate::domain::repository::{ApiSchemaRepository, ApiSchemaVersionRepository};
use crate::domain::service::api_registry_service::ApiRegistryDomainService;

#[derive(Debug, Clone)]
pub struct CheckCompatibilityInput {
    pub name: String,
    pub content: String,
    pub base_version: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CheckCompatibilityOutput {
    pub name: String,
    pub base_version: u32,
    pub result: CompatibilityResult,
}

#[derive(Debug, thiserror::Error)]
pub enum CheckCompatibilityError {
    #[error("schema not found: {0}")]
    SchemaNotFound(String),
    #[error("schema version not found: {name}@{version}")]
    VersionNotFound { name: String, version: u32 },
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CheckCompatibilityUseCase {
    schema_repo: Arc<dyn ApiSchemaRepository>,
    version_repo: Arc<dyn ApiSchemaVersionRepository>,
    domain_service: ApiRegistryDomainService,
}

impl CheckCompatibilityUseCase {
    pub fn new(
        schema_repo: Arc<dyn ApiSchemaRepository>,
        version_repo: Arc<dyn ApiSchemaVersionRepository>,
    ) -> Self {
        Self {
            schema_repo,
            version_repo,
            domain_service: ApiRegistryDomainService::new(),
        }
    }

    pub async fn execute(
        &self,
        input: &CheckCompatibilityInput,
    ) -> Result<CheckCompatibilityOutput, CheckCompatibilityError> {
        let schema = self
            .schema_repo
            .find_by_name(&input.name)
            .await
            .map_err(|e| CheckCompatibilityError::Internal(e.to_string()))?
            .ok_or_else(|| CheckCompatibilityError::SchemaNotFound(input.name.clone()))?;

        let base_version_num = input.base_version.unwrap_or(schema.latest_version);

        let base = self
            .version_repo
            .find_by_name_and_version(&input.name, base_version_num)
            .await
            .map_err(|e| CheckCompatibilityError::Internal(e.to_string()))?
            .ok_or_else(|| CheckCompatibilityError::VersionNotFound {
                name: input.name.clone(),
                version: base_version_num,
            })?;

        let result = self.domain_service.check_compatibility(
            &base.schema_type,
            &base.content,
            &input.content,
        );

        Ok(CheckCompatibilityOutput {
            name: input.name.clone(),
            base_version: base_version_num,
            result,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::api_registration::{ApiSchema, ApiSchemaVersion, SchemaType};
    use crate::domain::repository::api_repository::{
        MockApiSchemaRepository, MockApiSchemaVersionRepository,
    };

    #[tokio::test]
    async fn success() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock.expect_find_by_name().returning(|_| {
            Ok(Some(ApiSchema::new(
                "test-api".to_string(),
                "Test API".to_string(),
                SchemaType::OpenApi,
            )))
        });

        let mut version_mock = MockApiSchemaVersionRepository::new();
        version_mock
            .expect_find_by_name_and_version()
            .returning(|_, _| {
                Ok(Some(ApiSchemaVersion::new(
                    "test-api".to_string(),
                    1,
                    SchemaType::OpenApi,
                    "openapi: 3.0.3".to_string(),
                    "user-001".to_string(),
                )))
            });

        let uc = CheckCompatibilityUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = CheckCompatibilityInput {
            name: "test-api".to_string(),
            content: "openapi: 3.0.3\nversion: 2.0.0".to_string(),
            base_version: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.name, "test-api");
        assert_eq!(output.base_version, 1);
        assert!(output.result.compatible);
    }

    #[tokio::test]
    async fn breaking_change_detected() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock.expect_find_by_name().returning(|_| {
            Ok(Some(ApiSchema::new(
                "test-api".to_string(),
                "Test API".to_string(),
                SchemaType::OpenApi,
            )))
        });

        let mut version_mock = MockApiSchemaVersionRepository::new();
        version_mock
            .expect_find_by_name_and_version()
            .returning(|_, _| {
                Ok(Some(ApiSchemaVersion::new(
                    "test-api".to_string(),
                    1,
                    SchemaType::OpenApi,
                    "openapi: 3.0.3\npaths:\n  /api/v1/users:\n    get:\n      summary: Users\n  /api/v1/orders:\n    get:\n      summary: Orders\n".to_string(),
                    "user-001".to_string(),
                )))
            });

        let uc = CheckCompatibilityUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = CheckCompatibilityInput {
            name: "test-api".to_string(),
            content: "openapi: 3.0.3\npaths:\n  /api/v1/users:\n    get:\n      summary: Users\n".to_string(),
            base_version: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.result.compatible);
        assert!(!output.result.breaking_changes.is_empty());
        assert_eq!(output.result.breaking_changes[0].change_type, "path_removed");
    }

    #[tokio::test]
    async fn not_found() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock
            .expect_find_by_name()
            .returning(|_| Ok(None));

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = CheckCompatibilityUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = CheckCompatibilityInput {
            name: "nonexistent".to_string(),
            content: "openapi: 3.0.3".to_string(),
            base_version: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CheckCompatibilityError::SchemaNotFound(name) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected SchemaNotFound error"),
        }
    }

    #[tokio::test]
    async fn error() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock
            .expect_find_by_name()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = CheckCompatibilityUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = CheckCompatibilityInput {
            name: "test-api".to_string(),
            content: "openapi: 3.0.3".to_string(),
            base_version: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CheckCompatibilityError::Internal(msg) => assert!(msg.contains("db error")),
            _ => panic!("Expected Internal error"),
        }
    }
}
