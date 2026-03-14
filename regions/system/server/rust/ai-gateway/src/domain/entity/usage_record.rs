// 使用量レコードのエンティティ定義。
// テナントごとのLLM API使用量とコストを追跡する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 使用量レコードを表すエンティティ。
/// 各APIリクエストのトークン使用量とコストを記録する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    /// レコードの一意識別子
    pub id: String,
    /// テナントID
    pub tenant_id: String,
    /// 使用されたモデルID
    pub model_id: String,
    /// 入力（プロンプト）トークン数
    pub prompt_tokens: i32,
    /// 出力（完了）トークン数
    pub completion_tokens: i32,
    /// 合計コスト（USD）
    pub cost_usd: f64,
    /// レコード作成日時
    pub created_at: DateTime<Utc>,
}

impl UsageRecord {
    /// 新しい使用量レコードインスタンスを生成する。
    pub fn new(
        id: String,
        tenant_id: String,
        model_id: String,
        prompt_tokens: i32,
        completion_tokens: i32,
        cost_usd: f64,
    ) -> Self {
        Self {
            id,
            tenant_id,
            model_id,
            prompt_tokens,
            completion_tokens,
            cost_usd,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_usage_record() {
        let record = UsageRecord::new(
            "record-1".to_string(),
            "tenant-1".to_string(),
            "model-1".to_string(),
            100,
            50,
            0.005,
        );
        assert_eq!(record.id, "record-1");
        assert_eq!(record.tenant_id, "tenant-1");
        assert_eq!(record.prompt_tokens, 100);
        assert_eq!(record.completion_tokens, 50);
    }
}
