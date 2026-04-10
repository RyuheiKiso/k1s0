// AI Gatewayクライアント
// bb-ai-clientのHttpAiClientをラップし、AI Gatewayサーバーとの通信を行う

use async_trait::async_trait;

use k1s0_bb_ai_client::traits::AiClient;
use k1s0_bb_ai_client::types::{
    AiClientError, CompleteRequest, CompleteResponse, EmbedRequest, EmbedResponse, ModelInfo,
};
use k1s0_bb_ai_client::HttpAiClient;

/// `AiGatewayClient` はAI Gatewayサーバーへの接続を管理する
pub struct AiGatewayClient {
    /// 内部のHTTP AIクライアント
    inner: HttpAiClient,
}

impl AiGatewayClient {
    /// `新しいAiGatewayClientを生成する`
    /// endpointはAI `GatewayサーバーのベースURL`
    #[must_use]
    pub fn new(endpoint: &str) -> Self {
        // API Keyは内部通信のため空文字列を使用する
        let inner = HttpAiClient::new(endpoint.to_string(), String::new());
        Self { inner }
    }
}

#[async_trait]
impl AiClient for AiGatewayClient {
    /// チャット補完リクエストをAI Gatewayに委譲する
    async fn complete(&self, req: &CompleteRequest) -> Result<CompleteResponse, AiClientError> {
        self.inner.complete(req).await
    }

    /// 埋め込みリクエストをAI Gatewayに委譲する
    async fn embed(&self, req: &EmbedRequest) -> Result<EmbedResponse, AiClientError> {
        self.inner.embed(req).await
    }

    /// モデル一覧取得をAI Gatewayに委譲する
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AiClientError> {
        self.inner.list_models().await
    }
}
