// テキスト補完ユースケースの実装。
// ガードレール検査 → ルーティング → LLM呼び出し → 使用量記録の一連の処理を行う。

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::domain::entity::usage_record::UsageRecord;
use crate::domain::repository::UsageRepository;
use crate::domain::service::guardrail_service::GuardrailService;
use crate::domain::service::routing_service::RoutingService;
use crate::infrastructure::llm_client::LlmClient;

/// 補完リクエスト
#[derive(Debug, Deserialize)]
pub struct CompleteInput {
    /// 使用するモデルID
    pub model: String,
    /// メッセージ一覧
    pub messages: Vec<MessageInput>,
    /// 最大出力トークン数
    pub max_tokens: i32,
    /// テナントID
    pub tenant_id: String,
    /// ルーティング戦略（オプション）
    #[serde(default = "default_strategy")]
    pub strategy: String,
}

fn default_strategy() -> String {
    "default".to_string()
}

/// メッセージの入力DTO
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageInput {
    /// ロール（system, user, assistant）
    pub role: String,
    /// メッセージ内容
    pub content: String,
}

/// 補完レスポンス
#[derive(Debug, Serialize)]
pub struct CompleteOutput {
    /// 選択されたモデルID
    pub model: String,
    /// 生成されたテキスト
    pub content: String,
    /// 入力トークン数
    pub prompt_tokens: i32,
    /// 出力トークン数
    pub completion_tokens: i32,
}

/// 補完ユースケースのエラー
#[derive(Debug, thiserror::Error)]
pub enum CompleteError {
    #[error("ガードレール違反: {0}")]
    GuardrailViolation(String),
    #[error("モデルが見つかりません: {0}")]
    ModelNotFound(String),
    #[error("LLMリクエストエラー: {0}")]
    LlmError(String),
    #[error("内部エラー: {0}")]
    #[allow(dead_code)]
    Internal(String),
}

/// テキスト補完ユースケース。
/// ガードレールで安全性を確認し、ルーティングサービスでモデルを選択し、
/// LLMクライアントでリクエストを送信し、使用量を記録する。
pub struct CompleteUseCase {
    guardrail: Arc<GuardrailService>,
    routing: Arc<RoutingService>,
    llm_client: Arc<LlmClient>,
    usage_repo: Arc<dyn UsageRepository>,
}

impl CompleteUseCase {
    /// 新しい補完ユースケースを生成する。
    pub fn new(
        guardrail: Arc<GuardrailService>,
        routing: Arc<RoutingService>,
        llm_client: Arc<LlmClient>,
        usage_repo: Arc<dyn UsageRepository>,
    ) -> Self {
        Self {
            guardrail,
            routing,
            llm_client,
            usage_repo,
        }
    }

    /// 補完リクエストを処理する。
    pub async fn execute(&self, input: CompleteInput) -> Result<CompleteOutput, CompleteError> {
        // ガードレール検査：全メッセージの内容を検査
        for msg in &input.messages {
            self.guardrail
                .check_prompt(&msg.content)
                .map_err(CompleteError::GuardrailViolation)?;
        }

        // ルーティング：最適なモデルを選択
        let selected_model = self
            .routing
            .select_model(&input.model, &input.strategy)
            .await
            .ok_or_else(|| CompleteError::ModelNotFound(input.model.clone()))?;

        info!(
            model = %selected_model,
            strategy = %input.strategy,
            "モデルを選択"
        );

        // LLMクライアントでリクエスト送信
        let messages: Vec<crate::infrastructure::llm_client::Message> = input
            .messages
            .iter()
            .map(|m| crate::infrastructure::llm_client::Message {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let response = self
            .llm_client
            .complete(&selected_model, &messages, input.max_tokens)
            .await
            .map_err(|e| CompleteError::LlmError(e.to_string()))?;

        // 使用量レコードを保存
        let record = UsageRecord::new(
            Uuid::new_v4().to_string(),
            input.tenant_id,
            selected_model.clone(),
            response.prompt_tokens,
            response.completion_tokens,
            0.0, // コスト計算はモデル情報と組み合わせて行う
        );

        if let Err(e) = self.usage_repo.save(&record).await {
            tracing::warn!(error = %e, "使用量レコードの保存に失敗");
        }

        Ok(CompleteOutput {
            model: selected_model,
            content: response.content,
            prompt_tokens: response.prompt_tokens,
            completion_tokens: response.completion_tokens,
        })
    }
}
