use std::sync::Arc;

use crate::domain::entity::version::AppVersion;
use crate::domain::repository::VersionRepository;

/// ListVersionsUseCase はアプリバージョン一覧取得ユースケース。
pub struct ListVersionsUseCase {
    version_repo: Arc<dyn VersionRepository>,
}

impl ListVersionsUseCase {
    pub fn new(version_repo: Arc<dyn VersionRepository>) -> Self {
        Self { version_repo }
    }

    pub async fn execute(&self, app_id: &str) -> anyhow::Result<Vec<AppVersion>> {
        self.version_repo.list_by_app(app_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::platform::Platform;
    use crate::domain::repository::version_repository::MockVersionRepository;

    #[tokio::test]
    async fn test_list_versions_success() {
        let mut mock = MockVersionRepository::new();
        mock.expect_list_by_app()
            .withf(|id| id == "cli")
            .returning(|_| {
                Ok(vec![AppVersion {
                    id: uuid::Uuid::new_v4(),
                    app_id: "cli".to_string(),
                    version: "1.0.0".to_string(),
                    platform: Platform::Linux,
                    arch: "amd64".to_string(),
                    size_bytes: Some(10_000_000),
                    checksum_sha256: "abc123".to_string(),
                    s3_key: "cli/1.0.0/linux/amd64/k1s0".to_string(),
                    release_notes: None,
                    mandatory: false,
                    published_at: chrono::Utc::now(),
                    created_at: chrono::Utc::now(),
                }])
            });

        let uc = ListVersionsUseCase::new(Arc::new(mock));
        let result = uc.execute("cli").await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].version, "1.0.0");
    }
}
