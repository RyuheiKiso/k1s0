// ステータス定義バージョン取得ユースケース。
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::status_definition_version::StatusDefinitionVersion;
use crate::domain::repository::version_repository::VersionRepository;

pub struct GetStatusDefinitionVersionsUseCase {
    repo: Arc<dyn VersionRepository>,
}

impl GetStatusDefinitionVersionsUseCase {
    pub fn new(repo: Arc<dyn VersionRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        status_definition_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<(Vec<StatusDefinitionVersion>, i64)> {
        let versions = self
            .repo
            .find_by_status_definition(status_definition_id, limit, offset)
            .await?;
        let total = self
            .repo
            .count_by_status_definition(status_definition_id)
            .await?;
        Ok((versions, total))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::version_repository::MockVersionRepository;
    use chrono::Utc;
    use mockall::predicate::*;

    // テスト用バージョンデータを生成するヘルパー関数
    fn sample_version(status_def_id: Uuid, version_number: i32) -> StatusDefinitionVersion {
        StatusDefinitionVersion {
            id: Uuid::new_v4(),
            status_definition_id: status_def_id,
            version_number,
            before_data: None,
            after_data: Some(serde_json::json!({"code": "OPEN"})),
            changed_by: "admin".to_string(),
            change_reason: None,
            created_at: Utc::now(),
        }
    }

    // バージョン一覧取得の正常系：複数バージョンを正しく返すことを確認する
    // 前提: リポジトリが3件のバージョンを返す
    // 期待: (versions, total) のタプルが返され、件数が一致する
    #[tokio::test]
    async fn test_list_success() {
        let mut mock_repo = MockVersionRepository::new();
        let sd_id = Uuid::new_v4();
        let versions = vec![
            sample_version(sd_id, 1),
            sample_version(sd_id, 2),
            sample_version(sd_id, 3),
        ];
        let versions_clone = versions.clone();
        mock_repo
            .expect_find_by_status_definition()
            .with(eq(sd_id), eq(10_i64), eq(0_i64))
            .times(1)
            .returning(move |_, _, _| Ok(versions_clone.clone()));
        mock_repo
            .expect_count_by_status_definition()
            .with(eq(sd_id))
            .times(1)
            .returning(|_| Ok(3));
        let uc = GetStatusDefinitionVersionsUseCase::new(Arc::new(mock_repo));
        let result = uc.list(sd_id, 10, 0).await;
        assert!(result.is_ok());
        let (list, total) = result.unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(total, 3);
    }

    // バージョン一覧取得の正常系：バージョンが0件の場合も正しく動作することを確認する
    // 前提: リポジトリが空のベクターを返す
    // 期待: ([], 0) が返される
    #[tokio::test]
    async fn test_list_empty() {
        let mut mock_repo = MockVersionRepository::new();
        let sd_id = Uuid::new_v4();
        mock_repo
            .expect_find_by_status_definition()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));
        mock_repo
            .expect_count_by_status_definition()
            .times(1)
            .returning(|_| Ok(0));
        let uc = GetStatusDefinitionVersionsUseCase::new(Arc::new(mock_repo));
        let result = uc.list(sd_id, 10, 0).await;
        assert!(result.is_ok());
        let (list, total) = result.unwrap();
        assert_eq!(list.len(), 0);
        assert_eq!(total, 0);
    }

    // バージョン一覧取得の異常系（リポジトリエラー）：DBエラーが伝播することを確認する
    // 前提: find_by_status_definition がエラーを返す
    // 期待: Err が返される
    #[tokio::test]
    async fn test_list_repository_error() {
        let mut mock_repo = MockVersionRepository::new();
        let sd_id = Uuid::new_v4();
        mock_repo
            .expect_find_by_status_definition()
            .times(1)
            .returning(|_, _, _| Err(anyhow::anyhow!("DB error")));
        let uc = GetStatusDefinitionVersionsUseCase::new(Arc::new(mock_repo));
        let result = uc.list(sd_id, 10, 0).await;
        assert!(result.is_err());
    }

    // ページネーション：offset と limit が正しくリポジトリに渡されることを確認する
    // 前提: limit=5, offset=10 でリクエストする
    // 期待: リポジトリが同じ引数で呼ばれる
    #[tokio::test]
    async fn test_list_pagination() {
        let mut mock_repo = MockVersionRepository::new();
        let sd_id = Uuid::new_v4();
        mock_repo
            .expect_find_by_status_definition()
            .with(eq(sd_id), eq(5_i64), eq(10_i64))
            .times(1)
            .returning(|_, _, _| Ok(vec![]));
        mock_repo
            .expect_count_by_status_definition()
            .times(1)
            .returning(|_| Ok(15));
        let uc = GetStatusDefinitionVersionsUseCase::new(Arc::new(mock_repo));
        let result = uc.list(sd_id, 5, 10).await;
        assert!(result.is_ok());
        let (_, total) = result.unwrap();
        assert_eq!(total, 15);
    }
}
