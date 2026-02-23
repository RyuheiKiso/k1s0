use std::sync::Arc;

use crate::domain::entity::api_registration::ApiSchema;
use crate::domain::repository::ApiSchemaRepository;

#[derive(Debug, Clone)]
pub struct ListSchemasInput {
    pub schema_type: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListSchemasOutput {
    pub schemas: Vec<ApiSchema>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListSchemasError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListSchemasUseCase {
    schema_repo: Arc<dyn ApiSchemaRepository>,
}

impl ListSchemasUseCase {
    pub fn new(schema_repo: Arc<dyn ApiSchemaRepository>) -> Self {
        Self { schema_repo }
    }

    pub async fn execute(
        &self,
        input: &ListSchemasInput,
    ) -> Result<ListSchemasOutput, ListSchemasError> {
        let (schemas, total_count) = self
            .schema_repo
            .find_all(input.schema_type.clone(), input.page, input.page_size)
            .await
            .map_err(|e| ListSchemasError::Internal(e.to_string()))?;

        let has_next = (input.page as u64) * (input.page_size as u64) < total_count;

        Ok(ListSchemasOutput {
            schemas,
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
    use crate::domain::entity::api_registration::SchemaType;
    use crate::domain::repository::api_repository::MockApiSchemaRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockApiSchemaRepository::new();
        mock.expect_find_all().returning(|_, _, _| {
            Ok((
                vec![ApiSchema::new(
                    "test-api".to_string(),
                    "Test API".to_string(),
                    SchemaType::OpenApi,
                )],
                1,
            ))
        });

        let uc = ListSchemasUseCase::new(Arc::new(mock));
        let input = ListSchemasInput {
            schema_type: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.schemas.len(), 1);
        assert_eq!(output.total_count, 1);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn success_with_filter() {
        let mut mock = MockApiSchemaRepository::new();
        mock.expect_find_all().returning(|schema_type, _, _| {
            assert_eq!(schema_type, Some("openapi".to_string()));
            Ok((Vec::new(), 0))
        });

        let uc = ListSchemasUseCase::new(Arc::new(mock));
        let input = ListSchemasInput {
            schema_type: Some("openapi".to_string()),
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.schemas.is_empty());
    }

    #[tokio::test]
    async fn error() {
        let mut mock = MockApiSchemaRepository::new();
        mock.expect_find_all()
            .returning(|_, _, _| Err(anyhow::anyhow!("db error")));

        let uc = ListSchemasUseCase::new(Arc::new(mock));
        let input = ListSchemasInput {
            schema_type: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ListSchemasError::Internal(msg) => assert!(msg.contains("db error")),
        }
    }
}
