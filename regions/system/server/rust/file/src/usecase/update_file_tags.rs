use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::file::FileMetadata;
use crate::domain::repository::FileMetadataRepository;

#[derive(Debug, Clone)]
pub struct UpdateFileTagsInput {
    pub file_id: String,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateFileTagsError {
    #[error("file not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateFileTagsUseCase {
    repo: Arc<dyn FileMetadataRepository>,
}

impl UpdateFileTagsUseCase {
    pub fn new(repo: Arc<dyn FileMetadataRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &UpdateFileTagsInput,
    ) -> Result<FileMetadata, UpdateFileTagsError> {
        let mut file = self
            .repo
            .find_by_id(&input.file_id)
            .await
            .map_err(|e| UpdateFileTagsError::Internal(e.to_string()))?
            .ok_or_else(|| UpdateFileTagsError::NotFound(input.file_id.clone()))?;

        file.update_tags(input.tags.clone());

        self.repo
            .update(&file)
            .await
            .map_err(|e| UpdateFileTagsError::Internal(e.to_string()))?;

        Ok(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::file_repository::MockFileMetadataRepository;

    fn sample_file() -> FileMetadata {
        FileMetadata::new(
            "file_001".to_string(),
            "report.pdf".to_string(),
            2048,
            "application/pdf".to_string(),
            "tenant-abc".to_string(),
            "user-001".to_string(),
            HashMap::new(),
            "tenant-abc/report.pdf".to_string(),
        )
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockFileMetadataRepository::new();
        let file = sample_file();
        let return_file = file.clone();

        mock.expect_find_by_id()
            .withf(|id| id == "file_001")
            .returning(move |_| Ok(Some(return_file.clone())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = UpdateFileTagsUseCase::new(Arc::new(mock));
        let mut new_tags = HashMap::new();
        new_tags.insert("reviewed".to_string(), "true".to_string());

        let input = UpdateFileTagsInput {
            file_id: "file_001".to_string(),
            tags: new_tags.clone(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.tags, new_tags);
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockFileMetadataRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = UpdateFileTagsUseCase::new(Arc::new(mock));
        let input = UpdateFileTagsInput {
            file_id: "missing".to_string(),
            tags: HashMap::new(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateFileTagsError::NotFound(id) => assert_eq!(id, "missing"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockFileMetadataRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = UpdateFileTagsUseCase::new(Arc::new(mock));
        let input = UpdateFileTagsInput {
            file_id: "file_001".to_string(),
            tags: HashMap::new(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            UpdateFileTagsError::Internal(msg) => assert!(msg.contains("db error")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
