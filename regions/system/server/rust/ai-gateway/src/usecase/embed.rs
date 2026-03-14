// エンベディングユースケースの実装。
// LLMクライアントを使用してテキストのベクトル埋め込みを取得する。

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::infrastructure::llm_client::LlmClient;

/// エンベディングリクエスト
#[derive(Debug, Deserialize)]
pub struct EmbedInput {
    /// 使用するモデルID
    pub model: String,
    /// 埋め込み対象のテキスト一覧
    pub inputs: Vec<String>,
}

/// エンベディングレスポンス
#[derive(Debug, Serialize)]
pub struct EmbedOutput {
    /// 使用されたモデルID
    pub model: String,
    /// 各入力テキストに対応するベクトル
    pub embeddings: Vec<Vec<f32>>,
}

/// エンベディングユースケースのエラー
#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    #[error("LLMリクエストエラー: {0}")]
    LlmError(String),
}

/// エンベディングユースケース。
/// LLMクライアント経由でテキストのベクトル埋め込みを取得する。
pub struct EmbedUseCase {
    llm_client: Arc<LlmClient>,
}

impl EmbedUseCase {
    /// 新しいエンベディングユースケースを生成する。
    pub fn new(llm_client: Arc<LlmClient>) -> Self {
        Self { llm_client }
    }

    /// エンベディングリクエストを処理する。
    pub async fn execute(&self, input: EmbedInput) -> Result<EmbedOutput, EmbedError> {
        let response = self
            .llm_client
            .embed(&input.model, &input.inputs)
            .await
            .map_err(|e| EmbedError::LlmError(e.to_string()))?;

        Ok(EmbedOutput {
            model: input.model,
            embeddings: response.embeddings,
        })
    }
}
