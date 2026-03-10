use std::sync::Arc;

use crate::domain::entity::platform::Platform;
use crate::domain::entity::version::AppVersion;
use crate::domain::repository::VersionRepository;

/// GetLatestError は最新バージョン取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GetLatestError {
    #[error("no version found for app={0} platform={1} arch={2}")]
    NotFound(String, String, String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// GetLatestUseCase は最新バージョン取得ユースケース。
pub struct GetLatestUseCase {
    version_repo: Arc<dyn VersionRepository>,
}

impl GetLatestUseCase {
    pub fn new(version_repo: Arc<dyn VersionRepository>) -> Self {
        Self { version_repo }
    }

    pub async fn execute(
        &self,
        app_id: &str,
        platform: &Platform,
        arch: &str,
    ) -> Result<AppVersion, GetLatestError> {
        match self.version_repo.find_latest(app_id, platform, arch).await {
            Ok(Some(v)) => Ok(v),
            Ok(None) => Err(GetLatestError::NotFound(
                app_id.to_string(),
                platform.to_string(),
                arch.to_string(),
            )),
            Err(e) => Err(GetLatestError::Internal(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::version_repository::MockVersionRepository;

    #[tokio::test]
    async fn test_get_latest_success() {
        let mut mock = MockVersionRepository::new();
        mock.expect_find_latest().returning(|_, _, _| {
            Ok(Some(AppVersion {
                id: uuid::Uuid::new_v4(),
                app_id: "cli".to_string(),
                version: "2.0.0".to_string(),
                platform: Platform::Linux,
                arch: "amd64".to_string(),
                size_bytes: Some(15_000_000),
                checksum_sha256: "latest_hash".to_string(),
                s3_key: "cli/2.0.0/linux/amd64/k1s0".to_string(),
                release_notes: None,
                mandatory: false,
                published_at: chrono::Utc::now(),
                created_at: chrono::Utc::now(),
            }))
        });

        let uc = GetLatestUseCase::new(Arc::new(mock));
        let result = uc.execute("cli", &Platform::Linux, "amd64").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version, "2.0.0");
    }

    #[tokio::test]
    async fn test_get_latest_not_found() {
        let mut mock = MockVersionRepository::new();
        mock.expect_find_latest().returning(|_, _, _| Ok(None));

        let uc = GetLatestUseCase::new(Arc::new(mock));
        let result = uc.execute("cli", &Platform::Macos, "arm64").await;
        assert!(matches!(result, Err(GetLatestError::NotFound(_, _, _))));
    }
}
