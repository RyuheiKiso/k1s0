// 使用量取得ユースケースの実装。
// テナントごとのAPI使用量を期間指定で集計する。

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::entity::usage_record::UsageRecord;
use crate::domain::repository::UsageRepository;

/// 使用量取得リクエスト
#[derive(Debug, Deserialize)]
pub struct GetUsageInput {
    /// テナントID
    pub tenant_id: String,
    /// 期間開始（ISO 8601形式）
    pub start: String,
    /// 期間終了（ISO 8601形式）
    pub end: String,
}

/// 使用量取得レスポンス
#[derive(Debug, Serialize)]
pub struct GetUsageOutput {
    /// 使用量レコード一覧
    pub records: Vec<UsageRecord>,
    /// 合計入力トークン数
    pub total_prompt_tokens: i64,
    /// 合計出力トークン数
    pub total_completion_tokens: i64,
    /// 合計コスト（USD）
    pub total_cost_usd: f64,
}

/// 使用量取得ユースケース。
/// 指定テナントの期間内使用量を集計して返す。
pub struct GetUsageUseCase {
    usage_repo: Arc<dyn UsageRepository>,
}

impl GetUsageUseCase {
    /// 新しい使用量取得ユースケースを生成する。
    pub fn new(usage_repo: Arc<dyn UsageRepository>) -> Self {
        Self { usage_repo }
    }

    /// 使用量を取得・集計する。
    pub async fn execute(&self, input: GetUsageInput) -> GetUsageOutput {
        let records = self
            .usage_repo
            .find_by_tenant(&input.tenant_id, &input.start, &input.end)
            .await;

        // 集計
        let total_prompt_tokens: i64 = records.iter().map(|r| r.prompt_tokens as i64).sum();
        let total_completion_tokens: i64 = records.iter().map(|r| r.completion_tokens as i64).sum();
        let total_cost_usd: f64 = records.iter().map(|r| r.cost_usd).sum();

        GetUsageOutput {
            records,
            total_prompt_tokens,
            total_completion_tokens,
            total_cost_usd,
        }
    }
}
