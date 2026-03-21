// モデル一覧ユースケースの実装。
// 利用可能なAIモデルの一覧を取得する。

use std::sync::Arc;

use serde::Serialize;

use crate::domain::entity::model::AiModel;
use crate::domain::repository::ModelRepository;

/// モデル一覧レスポンス
#[derive(Debug, Serialize)]
pub struct ListModelsOutput {
    /// モデル一覧
    pub models: Vec<AiModel>,
}

/// モデル一覧ユースケース。
/// リポジトリから全モデル情報を取得して返す。
pub struct ListModelsUseCase {
    model_repo: Arc<dyn ModelRepository>,
}

impl ListModelsUseCase {
    /// 新しいモデル一覧ユースケースを生成する。
    pub fn new(model_repo: Arc<dyn ModelRepository>) -> Self {
        Self { model_repo }
    }

    /// モデル一覧を取得する。
    pub async fn execute(&self) -> ListModelsOutput {
        let models = self.model_repo.find_all().await;
        ListModelsOutput { models }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::model::AiModel;
    use crate::domain::repository::model_repository::MockModelRepository;

    // サンプルのAIモデルを生成するヘルパー
    fn sample_model(id: &str) -> AiModel {
        AiModel::new(
            id.to_string(),
            format!("model-{}", id),
            "openai".to_string(),
            128000,
            true,
            0.03,
            0.06,
        )
    }

    // 正常系: リポジトリから全モデルが返される
    #[tokio::test]
    async fn test_list_models_returns_all() {
        let mut mock_repo = MockModelRepository::new();
        let models = vec![sample_model("m1"), sample_model("m2"), sample_model("m3")];
        let models_clone = models.clone();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move || models_clone.clone());

        let uc = ListModelsUseCase::new(Arc::new(mock_repo));
        let output = uc.execute().await;

        assert_eq!(output.models.len(), 3);
        assert_eq!(output.models[0].id, "m1");
        assert_eq!(output.models[2].id, "m3");
    }

    // 境界値: モデルが存在しない場合に空リストが返される
    #[tokio::test]
    async fn test_list_models_empty() {
        let mut mock_repo = MockModelRepository::new();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(|| Vec::new());

        let uc = ListModelsUseCase::new(Arc::new(mock_repo));
        let output = uc.execute().await;

        assert!(output.models.is_empty());
    }
}
