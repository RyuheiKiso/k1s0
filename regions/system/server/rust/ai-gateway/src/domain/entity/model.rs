// AIモデルのエンティティ定義。
// LLMプロバイダーが提供する各モデルの情報を表現する。

use serde::{Deserialize, Serialize};

/// AIモデルを表すエンティティ。
/// プロバイダーごとのモデル名、コンテキストウィンドウサイズ、コスト情報を保持する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModel {
    /// モデルの一意識別子
    pub id: String,
    /// モデル名（例: gpt-4, claude-3-opus）
    pub name: String,
    /// プロバイダー名（例: openai, anthropic）
    pub provider: String,
    /// コンテキストウィンドウのトークン数上限
    pub context_window: i32,
    /// モデルの有効/無効フラグ
    pub enabled: bool,
    /// 入力1000トークンあたりのコスト（USD）
    pub cost_per_1k_input: f64,
    /// 出力1000トークンあたりのコスト（USD）
    pub cost_per_1k_output: f64,
}

impl AiModel {
    /// 新しいAIモデルインスタンスを生成する。
    #[must_use] 
    pub fn new(
        id: String,
        name: String,
        provider: String,
        context_window: i32,
        enabled: bool,
        cost_per_1k_input: f64,
        cost_per_1k_output: f64,
    ) -> Self {
        Self {
            id,
            name,
            provider,
            context_window,
            enabled,
            cost_per_1k_input,
            cost_per_1k_output,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_model() {
        let model = AiModel::new(
            "model-1".to_string(),
            "gpt-4".to_string(),
            "openai".to_string(),
            128000,
            true,
            0.03,
            0.06,
        );
        assert_eq!(model.id, "model-1");
        assert_eq!(model.name, "gpt-4");
        assert_eq!(model.provider, "openai");
        assert_eq!(model.context_window, 128000);
        assert!(model.enabled);
    }
}
