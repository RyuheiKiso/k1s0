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
