use std::sync::Arc;

use crate::domain::entity::platform::Platform;
use crate::domain::repository::VersionRepository;

/// DeleteVersionUseCase はアプリバージョン削除ユースケース。
pub struct DeleteVersionUseCase {
    version_repo: Arc<dyn VersionRepository>,
}

impl DeleteVersionUseCase {
    pub fn new(version_repo: Arc<dyn VersionRepository>) -> Self {
        Self { version_repo }
    }

    pub async fn execute(
        &self,
        app_id: &str,
        version: &str,
        platform: &Platform,
        arch: &str,
    ) -> anyhow::Result<()> {
        self.version_repo
            .delete(app_id, version, platform, arch)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::version_repository::MockVersionRepository;

    #[tokio::test]
    async fn test_delete_version_success() {
        let mut mock = MockVersionRepository::new();
        mock.expect_delete().returning(|_, _, _, _| Ok(()));

        let uc = DeleteVersionUseCase::new(Arc::new(mock));
        let result = uc
            .execute("cli", "1.0.0", &Platform::Linux, "amd64")
            .await;
        assert!(result.is_ok());
    }
}
