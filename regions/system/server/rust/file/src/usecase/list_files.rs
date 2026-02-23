use std::sync::Arc;

use crate::domain::entity::file::FileMetadata;
use crate::domain::repository::FileMetadataRepository;

#[derive(Debug, Clone)]
pub struct ListFilesInput {
    pub tenant_id: Option<String>,
    pub owner_id: Option<String>,
    pub mime_type: Option<String>,
    pub tag: Option<(String, String)>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListFilesOutput {
    pub files: Vec<FileMetadata>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListFilesError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListFilesUseCase {
    repo: Arc<dyn FileMetadataRepository>,
}

impl ListFilesUseCase {
    pub fn new(repo: Arc<dyn FileMetadataRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &ListFilesInput) -> Result<ListFilesOutput, ListFilesError> {
        let (files, total_count) = self
            .repo
            .find_all(
                input.tenant_id.clone(),
                input.owner_id.clone(),
                input.mime_type.clone(),
                input.tag.clone(),
                input.page,
                input.page_size,
            )
            .await
            .map_err(|e| ListFilesError::Internal(e.to_string()))?;

        let has_next = (input.page as u64 * input.page_size as u64) < total_count;

        Ok(ListFilesOutput {
            files,
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
    use crate::domain::repository::file_repository::MockFileMetadataRepository;
    use std::collections::HashMap;

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
        let files = vec![file];
        let return_files = files.clone();

        mock.expect_find_all()
            .returning(move |_, _, _, _, _, _| Ok((return_files.clone(), 1)));

        let uc = ListFilesUseCase::new(Arc::new(mock));
        let input = ListFilesInput {
            tenant_id: Some("tenant-abc".to_string()),
            owner_id: None,
            mime_type: None,
            tag: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.files.len(), 1);
        assert_eq!(output.total_count, 1);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn empty_result() {
        let mut mock = MockFileMetadataRepository::new();
        mock.expect_find_all()
            .returning(|_, _, _, _, _, _| Ok((vec![], 0)));

        let uc = ListFilesUseCase::new(Arc::new(mock));
        let input = ListFilesInput {
            tenant_id: None,
            owner_id: None,
            mime_type: None,
            tag: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.files.is_empty());
        assert_eq!(output.total_count, 0);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockFileMetadataRepository::new();
        mock.expect_find_all()
            .returning(|_, _, _, _, _, _| Err(anyhow::anyhow!("db error")));

        let uc = ListFilesUseCase::new(Arc::new(mock));
        let input = ListFilesInput {
            tenant_id: None,
            owner_id: None,
            mime_type: None,
            tag: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ListFilesError::Internal(msg) => assert!(msg.contains("db error")),
        }
    }
}
