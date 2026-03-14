// LLMクライアントの実装。
// reqwestを使用してOpenAI互換APIにHTTPリクエストを送信する。

use serde::{Deserialize, Serialize};
use tracing::info;

/// チャットメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// ロール（system, user, assistant）
    pub role: String,
    /// メッセージ内容
    pub content: String,
}

/// 補完レスポンス
#[derive(Debug)]
pub struct CompleteResponse {
    /// 生成されたテキスト
    pub content: String,
    /// 入力トークン数
    pub prompt_tokens: i32,
    /// 出力トークン数
    pub completion_tokens: i32,
}

/// エンベディングレスポンス
#[derive(Debug)]
pub struct EmbedResponse {
    /// 各入力テキストに対応するベクトル
    pub embeddings: Vec<Vec<f32>>,
}

/// LLMクライアント。
/// OpenAI互換APIへのHTTPリクエストを管理する。
pub struct LlmClient {
    /// HTTPクライアント
    client: reqwest::Client,
    /// APIベースURL
    base_url: String,
    /// APIキー
    api_key: String,
}

impl LlmClient {
    /// 新しいLLMクライアントを生成する。
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            api_key,
        }
    }

    /// テキスト補完リクエストを送信する。
    /// OpenAI互換の /chat/completions エンドポイントを呼び出す。
    pub async fn complete(
        &self,
        model: &str,
        messages: &[Message],
        max_tokens: i32,
    ) -> anyhow::Result<CompleteResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens,
        });

        info!(model = %model, url = %url, "LLM補完リクエスト送信");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API error: status={}, body={}", status, body);
        }

        let body: OpenAiChatResponse = response.json().await?;

        let content = body
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(CompleteResponse {
            content,
            prompt_tokens: body.usage.prompt_tokens,
            completion_tokens: body.usage.completion_tokens,
        })
    }

    /// エンベディングリクエストを送信する。
    /// OpenAI互換の /embeddings エンドポイントを呼び出す。
    pub async fn embed(
        &self,
        model: &str,
        inputs: &[String],
    ) -> anyhow::Result<EmbedResponse> {
        let url = format!("{}/embeddings", self.base_url);

        let request_body = serde_json::json!({
            "model": model,
            "input": inputs,
        });

        info!(model = %model, url = %url, "LLMエンベディングリクエスト送信");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API error: status={}, body={}", status, body);
        }

        let body: OpenAiEmbeddingResponse = response.json().await?;

        let embeddings = body.data.into_iter().map(|d| d.embedding).collect();

        Ok(EmbedResponse { embeddings })
    }
}

// --- OpenAI互換APIレスポンス構造体 ---

/// OpenAI Chat Completionレスポンス
#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

/// OpenAI選択肢
#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

/// OpenAIメッセージ
#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    content: String,
}

/// OpenAI使用量
#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: i32,
    completion_tokens: i32,
}

/// OpenAI Embeddingレスポンス
#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
}

/// OpenAI Embeddingデータ
#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
}
