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
    #[error("LLM request error: {0}")]
    LlmError(String),
}

/// エンベディングユースケース。
/// LLMクライアント経由でテキストのベクトル埋め込みを取得する。
pub struct EmbedUseCase {
    llm_client: Arc<LlmClient>,
}

impl EmbedUseCase {
    /// 新しいエンベディングユースケースを生成する。
    #[must_use] 
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::llm_client::LlmClient;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // 正常系: エンベディングリクエストが成功し、ベクトルが返される
    #[tokio::test]
    async fn test_embed_success() {
        // wiremockでOpenAI互換APIをモックする
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"embedding": [0.1_f32, 0.2_f32, 0.3_f32]},
                    {"embedding": [0.4_f32, 0.5_f32, 0.6_f32]}
                ]
            })))
            .mount(&mock_server)
            .await;

        let llm_client = Arc::new(LlmClient::new(mock_server.uri(), "test-key".to_string()));
        let uc = EmbedUseCase::new(llm_client);

        let result = uc
            .execute(EmbedInput {
                model: "text-embedding-ada-002".to_string(),
                inputs: vec!["テキスト1".to_string(), "テキスト2".to_string()],
            })
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.model, "text-embedding-ada-002");
        // 2つの入力に対して2つのベクトルが返される
        assert_eq!(output.embeddings.len(), 2);
        assert_eq!(output.embeddings[0], vec![0.1_f32, 0.2_f32, 0.3_f32]);
        assert_eq!(output.embeddings[1], vec![0.4_f32, 0.5_f32, 0.6_f32]);
    }

    // 異常系: LLMリクエストが失敗した場合にLlmErrorが返る
    #[tokio::test]
    async fn test_embed_llm_error() {
        // wiremockでエラーレスポンスを返すモックサーバーを設定する
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
            .mount(&mock_server)
            .await;

        let llm_client = Arc::new(LlmClient::new(mock_server.uri(), "test-key".to_string()));
        let uc = EmbedUseCase::new(llm_client);

        let result = uc
            .execute(EmbedInput {
                model: "text-embedding-ada-002".to_string(),
                inputs: vec!["テキスト".to_string()],
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.err().unwrap(), EmbedError::LlmError(_)));
    }
}
