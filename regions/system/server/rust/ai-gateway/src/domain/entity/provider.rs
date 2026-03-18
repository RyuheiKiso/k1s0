// AIプロバイダーのエンティティ定義。
// OpenAI、Anthropicなどの外部LLMプロバイダー情報を表現する。

use serde::{Deserialize, Serialize};

/// AIプロバイダーを表すエンティティ。
/// API接続先のベースURLや有効状態を保持する。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Provider {
    /// プロバイダーの一意識別子
    pub id: String,
    /// プロバイダー名（例: openai, anthropic）
    pub name: String,
    /// APIのベースURL
    pub api_base_url: String,
    /// プロバイダーの有効/無効フラグ
    pub enabled: bool,
}

#[allow(dead_code)]
impl Provider {
    /// 新しいプロバイダーインスタンスを生成する。
    pub fn new(id: String, name: String, api_base_url: String, enabled: bool) -> Self {
        Self {
            id,
            name,
            api_base_url,
            enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_provider() {
        let provider = Provider::new(
            "provider-1".to_string(),
            "openai".to_string(),
            "https://api.openai.com/v1".to_string(),
            true,
        );
        assert_eq!(provider.id, "provider-1");
        assert_eq!(provider.name, "openai");
        assert!(provider.enabled);
    }
}
