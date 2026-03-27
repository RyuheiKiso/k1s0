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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::usage_record::UsageRecord;
    use crate::domain::repository::usage_repository::MockUsageRepository;

    // テスト用の使用量レコードを生成するヘルパー
    fn sample_record(
        id: &str,
        tenant_id: &str,
        prompt: i32,
        completion: i32,
        cost: f64,
    ) -> UsageRecord {
        UsageRecord::new(
            id.to_string(),
            tenant_id.to_string(),
            "gpt-4".to_string(),
            prompt,
            completion,
            cost,
        )
    }

    // 正常系: レコードが存在する場合に集計値が正しく計算される
    #[tokio::test]
    async fn test_get_usage_with_records() {
        let mut mock_repo = MockUsageRepository::new();
        let records = vec![
            sample_record("r1", "tenant-1", 100, 50, 0.005),
            sample_record("r2", "tenant-1", 200, 100, 0.015),
        ];
        let records_clone = records.clone();
        mock_repo
            .expect_find_by_tenant()
            .times(1)
            .returning(move |_, _, _| records_clone.clone());

        let uc = GetUsageUseCase::new(Arc::new(mock_repo));
        let output = uc
            .execute(GetUsageInput {
                tenant_id: "tenant-1".to_string(),
                start: "2024-01-01T00:00:00Z".to_string(),
                end: "2024-12-31T23:59:59Z".to_string(),
            })
            .await;

        assert_eq!(output.records.len(), 2);
        // 合計トークン数の正確性を検証する
        assert_eq!(output.total_prompt_tokens, 300);
        assert_eq!(output.total_completion_tokens, 150);
        // 合計コストの正確性を検証する（浮動小数点誤差を考慮）
        assert!((output.total_cost_usd - 0.020).abs() < 0.0001);
    }

    // 境界値: レコードが存在しない場合に集計値がゼロになる
    #[tokio::test]
    async fn test_get_usage_empty() {
        let mut mock_repo = MockUsageRepository::new();
        mock_repo
            .expect_find_by_tenant()
            .times(1)
            .returning(|_, _, _| Vec::new());

        let uc = GetUsageUseCase::new(Arc::new(mock_repo));
        let output = uc
            .execute(GetUsageInput {
                tenant_id: "empty-tenant".to_string(),
                start: "2024-01-01T00:00:00Z".to_string(),
                end: "2024-12-31T23:59:59Z".to_string(),
            })
            .await;

        assert!(output.records.is_empty());
        assert_eq!(output.total_prompt_tokens, 0);
        assert_eq!(output.total_completion_tokens, 0);
        assert_eq!(output.total_cost_usd, 0.0);
    }

    // 集計値検証: 複数レコードの合計トークン数が正確に計算される
    #[tokio::test]
    async fn test_get_usage_aggregation_accuracy() {
        let mut mock_repo = MockUsageRepository::new();
        // 3件のレコードで合計値を確認する
        let records = vec![
            sample_record("r1", "tenant-a", 1000, 500, 0.05),
            sample_record("r2", "tenant-a", 2000, 1000, 0.10),
            sample_record("r3", "tenant-a", 3000, 1500, 0.15),
        ];
        let records_clone = records.clone();
        mock_repo
            .expect_find_by_tenant()
            .times(1)
            .returning(move |_, _, _| records_clone.clone());

        let uc = GetUsageUseCase::new(Arc::new(mock_repo));
        let output = uc
            .execute(GetUsageInput {
                tenant_id: "tenant-a".to_string(),
                start: "2024-01-01T00:00:00Z".to_string(),
                end: "2024-12-31T23:59:59Z".to_string(),
            })
            .await;

        assert_eq!(output.total_prompt_tokens, 6000);
        assert_eq!(output.total_completion_tokens, 3000);
        assert!((output.total_cost_usd - 0.30).abs() < 0.0001);
    }
}
