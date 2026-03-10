use std::sync::Arc;

use crate::domain::entity::version::AppVersion;
use crate::domain::repository::VersionRepository;

/// CreateVersionUseCase はアプリバージョン作成ユースケース。
pub struct CreateVersionUseCase {
    version_repo: Arc<dyn VersionRepository>,
}

impl CreateVersionUseCase {
    pub fn new(version_repo: Arc<dyn VersionRepository>) -> Self {
        Self { version_repo }
    }

    pub async fn execute(&self, version: &AppVersion) -> anyhow::Result<AppVersion> {
        self.version_repo.create(version).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::platform::Platform;
    use crate::domain::repository::version_repository::MockVersionRepository;

    #[tokio::test]
    async fn test_create_version_success() {
        let mut mock = MockVersionRepository::new();
        mock.expect_create().returning(|v| {
            Ok(AppVersion {
                id: uuid::Uuid::new_v4(),
                ..v.clone()
            })
        });

        let uc = CreateVersionUseCase::new(Arc::new(mock));
        let version = AppVersion {
            id: uuid::Uuid::new_v4(),
            app_id: "cli".to_string(),
            version: "1.1.0".to_string(),
            platform: Platform::Linux,
            arch: "amd64".to_string(),
            size_bytes: Some(10_000_000),
            checksum_sha256: "sha256hash".to_string(),
            s3_key: "cli/1.1.0/linux/amd64/k1s0".to_string(),
            release_notes: Some("New feature".to_string()),
            mandatory: false,
            published_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        };

        let result = uc.execute(&version).await;
        assert!(result.is_ok());
    }
}
