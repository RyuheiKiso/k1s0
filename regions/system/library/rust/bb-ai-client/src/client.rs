// HTTP経由でAIバックエンドと通信するクライアント実装
// reqwestを使用してREST APIにリクエストを送信する

use crate::traits::AiClient;
use crate::types::{AiClientError, AiModel, CompleteRequest, CompleteResponse, EmbedRequest, EmbedResponse};

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
            base_url,
            api_key,
            client,
        }
    }
}

#[async_trait::async_trait]
impl AiClient for HttpAiClient {
    /// チャット補完エンドポイントにPOSTリクエストを送信する
    async fn complete(&self, req: &CompleteRequest) -> Result<CompleteResponse, AiClientError> {
        // /v1/chat/completions エンドポイントにリクエストを送信する
        let url = format!("{}/v1/chat/completions", self.base_url);
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

    /// 埋め込みエンドポイントにPOSTリクエストを送信する
    async fn embed(&self, req: &EmbedRequest) -> Result<EmbedResponse, AiClientError> {
        // /v1/embeddings エンドポイントにリクエストを送信する
        let url = format!("{}/v1/embeddings", self.base_url);
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

    /// モデル一覧エンドポイントにGETリクエストを送信する
    async fn list_models(&self) -> Result<Vec<AiModel>, AiClientError> {
        // /v1/models エンドポイントにリクエストを送信する
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
            .json::<Vec<AiModel>>()
            .await
            .map_err(|e| AiClientError::JsonError(e.to_string()))
    }
}
