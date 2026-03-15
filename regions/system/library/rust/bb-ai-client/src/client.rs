// HTTP経由でAIバックエンドと通信するクライアント実装
// reqwestを使用してREST APIにリクエストを送信する

use crate::traits::AiClient;
use crate::types::{
    AiClientError, CompleteRequest, CompleteResponse, EmbedRequest, EmbedResponse, ModelInfo,
};

// HTTP AIクライアント構造体
// ベースURLとAPIキーを保持し、reqwestクライアントで通信を行う
pub struct HttpAiClient {
    /// APIのベースURL
    base_url: String,
    /// 認証用APIキー
    api_key: String,
    /// HTTPクライアントインスタンス
    client: reqwest::Client,
}

impl HttpAiClient {
    /// 新しいHttpAiClientインスタンスを生成する
    pub fn new(base_url: String, api_key: String) -> Self {
        // reqwestクライアントを初期化する
        let client = reqwest::Client::new();
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            client,
        }
    }
}

#[async_trait::async_trait]
impl AiClient for HttpAiClient {
    /// AI ゲートウェイの /v1/complete エンドポイントにPOSTリクエストを送信する
    async fn complete(&self, req: &CompleteRequest) -> Result<CompleteResponse, AiClientError> {
        let url = format!("{}/v1/complete", self.base_url);
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(req)
            .send()
            .await
            .map_err(|e| AiClientError::HttpError(e.to_string()))?;

        // HTTPステータスコードを確認する
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AiClientError::HttpError(format!(
                "status={}, body={}",
                status, body
            )));
        }

        // レスポンスボディをJSON形式でデシリアライズする
        response
            .json::<CompleteResponse>()
            .await
            .map_err(|e| AiClientError::JsonError(e.to_string()))
    }

    /// AI ゲートウェイの /v1/embed エンドポイントにPOSTリクエストを送信する
    async fn embed(&self, req: &EmbedRequest) -> Result<EmbedResponse, AiClientError> {
        let url = format!("{}/v1/embed", self.base_url);
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(req)
            .send()
            .await
            .map_err(|e| AiClientError::HttpError(e.to_string()))?;

        // HTTPステータスコードを確認する
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AiClientError::HttpError(format!(
                "status={}, body={}",
                status, body
            )));
        }

        // レスポンスボディをJSON形式でデシリアライズする
        response
            .json::<EmbedResponse>()
            .await
            .map_err(|e| AiClientError::JsonError(e.to_string()))
    }

    /// AI ゲートウェイの /v1/models エンドポイントにGETリクエストを送信する
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AiClientError> {
        let url = format!("{}/v1/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|e| AiClientError::HttpError(e.to_string()))?;

        // HTTPステータスコードを確認する
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AiClientError::HttpError(format!(
                "status={}, body={}",
                status, body
            )));
        }

        // レスポンスボディをJSON形式でデシリアライズする
        response
            .json::<Vec<ModelInfo>>()
            .await
            .map_err(|e| AiClientError::JsonError(e.to_string()))
    }
}
