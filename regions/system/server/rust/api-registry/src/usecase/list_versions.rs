use std::sync::Arc;

use crate::domain::entity::api_registration::ApiSchemaVersion;
use crate::domain::repository::{ApiSchemaRepository, ApiSchemaVersionRepository};

#[derive(Debug, Clone)]
pub struct ListVersionsInput {
    pub name: String,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListVersionsOutput {
    pub name: String,
    pub versions: Vec<ApiSchemaVersion>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListVersionsError {
    #[error("schema not found: {0}")]
    NotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListVersionsUseCase {
    schema_repo: Arc<dyn ApiSchemaRepository>,
    version_repo: Arc<dyn ApiSchemaVersionRepository>,
}

impl ListVersionsUseCase {
    pub fn new(
        schema_repo: Arc<dyn ApiSchemaRepository>,
        version_repo: Arc<dyn ApiSchemaVersionRepository>,
    ) -> Self {
        Self {
            schema_repo,
            version_repo,
        }
    }

    pub async fn execute(
        &self,
        input: &ListVersionsInput,
    ) -> Result<ListVersionsOutput, ListVersionsError> {
        let schema = self
            .schema_repo
            .find_by_name(&input.name)
            .await
            .map_err(|e| ListVersionsError::Internal(e.to_string()))?
            .ok_or_else(|| ListVersionsError::NotFound(input.name.clone()))?;

        let (versions, total_count) = self
            .version_repo
            .find_all_by_name(&input.name, input.page, input.page_size)
            .await
            .map_err(|e| ListVersionsError::Internal(e.to_string()))?;

        let has_next = (input.page as u64) * (input.page_size as u64) < total_count;

        Ok(ListVersionsOutput {
            name: schema.name,
            versions,
            total_count,
            page: input.page,
            page_size: input.page_size,
            has_next,
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
        version_mock.expect_find_all_by_name().returning(|_, _, _| {
            Ok((
                vec![ApiSchemaVersion::new(
                    "test-api".to_string(),
                    1,
                    SchemaType::OpenApi,
                    "openapi: 3.0.3".to_string(),
                    "user-001".to_string(),
                )],
                1,
            ))
        });

        let uc = ListVersionsUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = ListVersionsInput {
            name: "test-api".to_string(),
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.name, "test-api");
        assert_eq!(output.versions.len(), 1);
    }

    #[tokio::test]
    async fn not_found() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock
            .expect_find_by_name()
            .returning(|_| Ok(None));

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = ListVersionsUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = ListVersionsInput {
            name: "nonexistent".to_string(),
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ListVersionsError::NotFound(name) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn error() {
        let mut schema_mock = MockApiSchemaRepository::new();
        schema_mock
            .expect_find_by_name()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let version_mock = MockApiSchemaVersionRepository::new();

        let uc = ListVersionsUseCase::new(Arc::new(schema_mock), Arc::new(version_mock));
        let input = ListVersionsInput {
            name: "test-api".to_string(),
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ListVersionsError::Internal(msg) => assert!(msg.contains("db error")),
            _ => panic!("Expected Internal error"),
        }
    }
}
