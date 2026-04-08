// インメモリAIクライアント実装（テスト用）
// 固定レスポンスを返すことで、外部APIに依存しないテストを実現する

use crate::traits::AiClient;
use crate::types::{
    AiClientError, CompleteRequest, CompleteResponse, EmbedRequest, EmbedResponse, ModelInfo,
};
use tokio::sync::Mutex;

// テスト用インメモリAIクライアント
// 事前に設定したレスポンスを順番に返す
pub struct InMemoryAiClient {
    /// チャット補完用の固定レスポンスキュー
    responses: Mutex<Vec<CompleteResponse>>,
    /// 埋め込み用の固定レスポンスキュー
    embed_responses: Mutex<Vec<EmbedResponse>>,
    /// モデル一覧
    models: Vec<ModelInfo>,
}

impl InMemoryAiClient {
    /// `固定レスポンスを持つInMemoryAiClientを生成する`
    #[must_use] 
    pub fn new(responses: Vec<CompleteResponse>, embed_responses: Vec<EmbedResponse>) -> Self {
        Self {
            responses: Mutex::new(responses),
            embed_responses: Mutex::new(embed_responses),
            models: vec![],
        }
    }

    /// `モデル一覧付きのInMemoryAiClientを生成する`
    #[must_use] 
    pub fn with_models(
        responses: Vec<CompleteResponse>,
        embed_responses: Vec<EmbedResponse>,
        models: Vec<ModelInfo>,
    ) -> Self {
        Self {
            responses: Mutex::new(responses),
            embed_responses: Mutex::new(embed_responses),
            models,
        }
    }
}

#[async_trait::async_trait]
impl AiClient for InMemoryAiClient {
    /// キューから次のチャット補完レスポンスを返す
    async fn complete(&self, _req: &CompleteRequest) -> Result<CompleteResponse, AiClientError> {
        // レスポンスキューから先頭の要素を取り出す
        let mut responses = self.responses.lock().await;
        if responses.is_empty() {
            return Err(AiClientError::Unavailable(
                "No more responses in queue".to_string(),
            ));
        }
        Ok(responses.remove(0))
    }

    /// キューから次の埋め込みレスポンスを返す
    async fn embed(&self, _req: &EmbedRequest) -> Result<EmbedResponse, AiClientError> {
        // 埋め込みレスポンスキューから先頭の要素を取り出す
        let mut responses = self.embed_responses.lock().await;
        if responses.is_empty() {
            return Err(AiClientError::Unavailable(
                "No more embed responses in queue".to_string(),
            ));
        }
        Ok(responses.remove(0))
    }

    /// 設定されたモデル一覧を返す
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AiClientError> {
        Ok(self.models.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::AiClient;

    fn make_response(content: &str) -> CompleteResponse {
        CompleteResponse {
            id: "resp-1".to_string(),
            model: "test-model".to_string(),
            content: content.to_string(),
            usage: crate::types::Usage {
                input_tokens: 10,
                output_tokens: 5,
            },
        }
    }

    fn make_embed_response() -> EmbedResponse {
        EmbedResponse {
            model: "embed-model".to_string(),
            embeddings: vec![vec![0.1, 0.2, 0.3]],
        }
    }

    fn make_complete_req() -> CompleteRequest {
        CompleteRequest {
            model: "test-model".to_string(),
            messages: vec![crate::types::ChatMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            max_tokens: None,
            temperature: None,
            stream: None,
        }
    }

    /// complete がキューから正しくレスポンスを返す
    #[tokio::test]
    async fn complete_returns_queued_response() {
        let client = InMemoryAiClient::new(vec![make_response("world")], vec![]);
        let resp = client.complete(&make_complete_req()).await.unwrap();
        assert_eq!(resp.content, "world");
        assert_eq!(resp.model, "test-model");
    }

    /// complete がキュー枯渇時に Unavailable エラーを返す
    #[tokio::test]
    async fn complete_empty_queue_returns_unavailable() {
        let client = InMemoryAiClient::new(vec![], vec![]);
        let err = client.complete(&make_complete_req()).await.unwrap_err();
        assert!(matches!(err, AiClientError::Unavailable(_)));
    }

    /// embed がキューから正しくレスポンスを返す
    #[tokio::test]
    async fn embed_returns_queued_response() {
        let client = InMemoryAiClient::new(vec![], vec![make_embed_response()]);
        let req = EmbedRequest {
            model: "embed-model".to_string(),
            texts: vec!["hello".to_string()],
        };
        let resp = client.embed(&req).await.unwrap();
        assert_eq!(resp.model, "embed-model");
        assert_eq!(resp.embeddings.len(), 1);
    }

    /// embed がキュー枯渇時に Unavailable エラーを返す
    #[tokio::test]
    async fn embed_empty_queue_returns_unavailable() {
        let client = InMemoryAiClient::new(vec![], vec![]);
        let req = EmbedRequest {
            model: "embed-model".to_string(),
            texts: vec![],
        };
        let err = client.embed(&req).await.unwrap_err();
        assert!(matches!(err, AiClientError::Unavailable(_)));
    }

    /// list_models がモデル一覧を返す
    #[tokio::test]
    async fn list_models_returns_configured_models() {
        let models = vec![ModelInfo {
            id: "m1".to_string(),
            name: "Model 1".to_string(),
            description: "desc".to_string(),
        }];
        let client = InMemoryAiClient::with_models(vec![], vec![], models);
        let result = client.list_models().await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "m1");
    }

    /// list_models がモデル未設定の場合空リストを返す
    #[tokio::test]
    async fn list_models_empty_returns_empty_vec() {
        let client = InMemoryAiClient::new(vec![], vec![]);
        let result = client.list_models().await.unwrap();
        assert!(result.is_empty());
    }

    /// 複数レスポンスがキュー順に返される
    #[tokio::test]
    async fn complete_multiple_responses_in_order() {
        let client = InMemoryAiClient::new(
            vec![make_response("first"), make_response("second")],
            vec![],
        );
        let r1 = client.complete(&make_complete_req()).await.unwrap();
        let r2 = client.complete(&make_complete_req()).await.unwrap();
        assert_eq!(r1.content, "first");
        assert_eq!(r2.content, "second");
    }
}
