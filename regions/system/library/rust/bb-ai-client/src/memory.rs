// インメモリAIクライアント実装（テスト用）
// 固定レスポンスを返すことで、外部APIに依存しないテストを実現する

use crate::traits::AiClient;
use crate::types::{AiClientError, AiModel, CompleteRequest, CompleteResponse, EmbedRequest, EmbedResponse};
use tokio::sync::Mutex;

// テスト用インメモリAIクライアント
// 事前に設定したレスポンスを順番に返す
pub struct InMemoryAiClient {
    /// チャット補完用の固定レスポンスキュー
    responses: Mutex<Vec<CompleteResponse>>,
    /// 埋め込み用の固定レスポンスキュー
    embed_responses: Mutex<Vec<EmbedResponse>>,
}

impl InMemoryAiClient {
    /// 固定レスポンスを持つInMemoryAiClientを生成する
    pub fn new(responses: Vec<CompleteResponse>, embed_responses: Vec<EmbedResponse>) -> Self {
        Self {
            responses: Mutex::new(responses),
            embed_responses: Mutex::new(embed_responses),
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

    /// テスト用の空のモデル一覧を返す
    async fn list_models(&self) -> Result<Vec<AiModel>, AiClientError> {
        // テスト用に空のリストを返す
        Ok(vec![])
    }
}
