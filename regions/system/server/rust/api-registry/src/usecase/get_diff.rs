use std::sync::Arc;

use crate::domain::entity::api_registration::SchemaDiff;
use crate::domain::repository::{ApiSchemaRepository, ApiSchemaVersionRepository};
use crate::domain::service::api_registry_service::ApiRegistryDomainService;

#[derive(Debug, Clone)]
pub struct GetDiffInput {
    pub name: String,
    pub from_version: Option<u32>,
    pub to_version: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct GetDiffOutput {
    pub name: String,
    pub from_version: u32,
    pub to_version: u32,
    pub breaking_changes: bool,
    pub diff: SchemaDiff,
}

#[derive(Debug, thiserror::Error)]
pub enum GetDiffError {
    #[error("schema not found: {0}")]
    SchemaNotFound(String),
    #[error("schema version not found: {name}@{version}")]
    VersionNotFound { name: String, version: u32 },
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetDiffUseCase {
    schema_repo: Arc<dyn ApiSchemaRepository>,
    version_repo: Arc<dyn ApiSchemaVersionRepository>,
    domain_service: ApiRegistryDomainService,
}

impl GetDiffUseCase {
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

    pub async fn execute(&self, input: &GetDiffInput) -> Result<GetDiffOutput, GetDiffError> {
        let schema = self
            .schema_repo
            .find_by_name(&input.name)
            .await
            .map_err(|e| GetDiffError::Internal(e.to_string()))?
            .ok_or_else(|| GetDiffError::SchemaNotFound(input.name.clone()))?;

        let to_version = input.to_version.unwrap_or(schema.latest_version);
        let from_version = input
            .from_version
            .unwrap_or_else(|| to_version.saturating_sub(1).max(1));

        if from_version >= to_version {
            return Err(GetDiffError::ValidationError(
                "'from' version must be less than 'to' version".to_string(),
            ));
        }

        let from_ver = self
            .version_repo
            .find_by_name_and_version(&input.name, from_version)
            .await
            .map_err(|e| GetDiffError::Internal(e.to_string()))?
            .ok_or_else(|| GetDiffError::VersionNotFound {
                name: input.name.clone(),
                version: from_version,
            })?;

        let to_ver = self
            .version_repo
            .find_by_name_and_version(&input.name, to_version)
            .await
            .map_err(|e| GetDiffError::Internal(e.to_string()))?
            .ok_or_else(|| GetDiffError::VersionNotFound {
                name: input.name.clone(),
                version: to_version,
            })?;

        let diff = self.domain_service.compute_diff(
            &from_ver.schema_type,
            &from_ver.content,
            &to_ver.content,
        );

        let compat = self.domain_service.check_compatibility(
            &from_ver.schema_type,
            &from_ver.content,
            &to_ver.content,
        );

        Ok(GetDiffOutput {
            name: input.name.clone(),
            from_version,
            to_version,
            breaking_changes: !compat.compatible,
            diff,
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
            let mut schema = ApiSchema::new(
                "test-api".to_string(),
                "Test API".to_string(),
                SchemaType::OpenApi,
            );
            schema.latest_version = 3;
            Ok(Some(schema))
        });

        let mut version_mock = MockApiSchemaVersionRepository::new();
        version_mock
            .expect_find_by_name_and_version()
            .returning(|name, ver| {
                Ok(Some(ApiSchemaVersion::new(
                    name.to_string(),
                    ver,
                    SchemaType::OpenApi,
                    format!("openapi: 3.0.3\nversion: {}.0.0", ver),
                    "user-001".to_string(),
                )))
            });

        let uc = GetDiffUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = GetDiffInput {
            name: "test-api".to_string(),
            from_version: Some(2),
            to_version: Some(3),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.name, "test-api");
        assert_eq!(output.from_version, 2);
        assert_eq!(output.to_version, 3);
    }

    #[tokio::test]
    async fn diff_with_breaking_changes() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock.expect_find_by_name().returning(|_| {
            let mut schema = ApiSchema::new(
                "test-api".to_string(),
                "Test API".to_string(),
                SchemaType::OpenApi,
            );
            schema.latest_version = 3;
            Ok(Some(schema))
        });

        let mut version_mock = MockApiSchemaVersionRepository::new();
        version_mock
            .expect_find_by_name_and_version()
            .returning(|name, ver| {
                let content = if ver == 2 {
                    "openapi: 3.0.3\npaths:\n  /api/v1/users:\n    get:\n      summary: Users\n  /api/v1/orders:\n    get:\n      summary: Orders\n".to_string()
                } else {
                    "openapi: 3.0.3\npaths:\n  /api/v1/users:\n    get:\n      summary: Users updated\n".to_string()
                };
                Ok(Some(ApiSchemaVersion::new(
                    name.to_string(),
                    ver,
                    SchemaType::OpenApi,
                    content,
                    "user-001".to_string(),
                )))
            });

        let uc = GetDiffUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = GetDiffInput {
            name: "test-api".to_string(),
            from_version: Some(2),
            to_version: Some(3),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.breaking_changes);
    }

    #[tokio::test]
    async fn not_found() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock
            .expect_find_by_name()
            .returning(|_| Ok(None));

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = GetDiffUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = GetDiffInput {
            name: "nonexistent".to_string(),
            from_version: Some(1),
            to_version: Some(2),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetDiffError::SchemaNotFound(name) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected SchemaNotFound error"),
        }
    }

    #[tokio::test]
    async fn validation_error_from_gte_to() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock.expect_find_by_name().returning(|_| {
            let mut schema = ApiSchema::new(
                "test-api".to_string(),
                "Test API".to_string(),
                SchemaType::OpenApi,
            );
            schema.latest_version = 3;
            Ok(Some(schema))
        });

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = GetDiffUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = GetDiffInput {
            name: "test-api".to_string(),
            from_version: Some(3),
            to_version: Some(2),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetDiffError::ValidationError(msg) => {
                assert!(msg.contains("'from' version must be less than 'to' version"))
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn error() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock
            .expect_find_by_name()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = GetDiffUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = GetDiffInput {
            name: "test-api".to_string(),
            from_version: Some(1),
            to_version: Some(2),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetDiffError::Internal(msg) => assert!(msg.contains("db error")),
            _ => panic!("Expected Internal error"),
        }
    }
}
