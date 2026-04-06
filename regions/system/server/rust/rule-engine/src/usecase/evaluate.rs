use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::rule::{EvaluationLog, EvaluationMode, Rule};
use crate::domain::repository::{EvaluationLogRepository, RuleRepository, RuleSetRepository};
use crate::domain::service::condition_evaluator::ConditionEvaluator;
use crate::domain::service::condition_parser::ConditionParser;

#[derive(Debug, Clone)]
pub struct EvaluateInput {
    /// CRITICAL-RUST-001 監査対応: テナント分離のために追加。JWT/ヘッダーから抽出したテナント ID を渡す。
    pub tenant_id: String,
    pub rule_set: String, // "{domain}.{name}" format
    pub input: serde_json::Value,
    pub context: serde_json::Value,
    pub dry_run: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MatchedRuleInfo {
    pub id: Uuid,
    pub name: String,
    pub priority: i32,
    pub result: serde_json::Value,
}

#[derive(Debug)]
pub struct EvaluateOutput {
    pub evaluation_id: Uuid,
    pub rule_set: String,
    pub rule_set_version: u32,
    pub matched_rules: Vec<MatchedRuleInfo>,
    pub result: serde_json::Value,
    pub default_applied: bool,
    pub cached: bool,
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum EvaluateError {
    #[error("rule set not found: {0}")]
    RuleSetNotFound(String),
    #[error("evaluation error: {0}")]
    EvaluationError(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct EvaluateUseCase {
    rule_set_repo: Arc<dyn RuleSetRepository>,
    rule_repo: Arc<dyn RuleRepository>,
    eval_log_repo: Arc<dyn EvaluationLogRepository>,
    /// 正規表現コンパイル結果キャッシュ付き評価器（RUST-HIGH-003 対応）
    condition_evaluator: ConditionEvaluator,
}

impl EvaluateUseCase {
    pub fn new(
        rule_set_repo: Arc<dyn RuleSetRepository>,
        rule_repo: Arc<dyn RuleRepository>,
        eval_log_repo: Arc<dyn EvaluationLogRepository>,
    ) -> Self {
        Self {
            rule_set_repo,
            rule_repo,
            eval_log_repo,
            // ConditionEvaluator は regex LruCache を内部で持つため、ユースケース生成時に一度だけ初期化する
            condition_evaluator: ConditionEvaluator::new(),
        }
    }

    pub async fn execute(&self, input: &EvaluateInput) -> Result<EvaluateOutput, EvaluateError> {
        // Parse "{domain}.{name}"
        let (domain, name) = Self::parse_rule_set_ref(&input.rule_set)?;

        let rule_set = self
            .rule_set_repo
            .find_by_domain_and_name(&domain, &name)
            .await
            .map_err(|e| EvaluateError::Internal(e.to_string()))?
            .ok_or_else(|| EvaluateError::RuleSetNotFound(input.rule_set.clone()))?;

        // Load rules
        let mut rules = self
            .rule_repo
            .find_by_ids(&rule_set.rule_ids)
            .await
            .map_err(|e| EvaluateError::Internal(e.to_string()))?;

        // Filter enabled, sort by priority (lower = higher priority)
        rules.retain(|r| r.enabled);
        rules.sort_by_key(|r| r.priority);

        let now = chrono::Utc::now();
        let evaluation_id = Uuid::new_v4();

        // L-002 監査対応: evaluate_rules が async になったため .await が必要
        let (matched_rules, result, default_applied) = self.evaluate_rules(
            &rules,
            &rule_set.evaluation_mode,
            &input.input,
            &rule_set.default_result,
        ).await?;

        // Log evaluation (unless dry_run)
        if !input.dry_run {
            let input_hash = Self::hash_input(&input.input);
            let log = EvaluationLog {
                id: evaluation_id,
                tenant_id: input.tenant_id.clone(),
                rule_set_name: input.rule_set.clone(),
                rule_set_version: rule_set.current_version,
                matched_rule_id: matched_rules.first().map(|m| m.id),
                input_hash,
                result: result.clone(),
                context: input.context.clone(),
                evaluated_at: now,
            };
            if let Err(e) = self.eval_log_repo.create(&log).await {
                tracing::warn!(error = %e, "failed to write evaluation log");
            }
        }

        Ok(EvaluateOutput {
            evaluation_id,
            rule_set: input.rule_set.clone(),
            rule_set_version: rule_set.current_version,
            matched_rules,
            result,
            default_applied,
            cached: false,
            evaluated_at: now,
        })
    }

    fn parse_rule_set_ref(rule_set_ref: &str) -> Result<(String, String), EvaluateError> {
        let parts: Vec<&str> = rule_set_ref.splitn(2, '.').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(EvaluateError::EvaluationError(format!(
                "rule_set must be in '{{domain}}.{{name}}' format, got: '{}'",
                rule_set_ref
            )));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// ルール評価ループ。ConditionEvaluator のキャッシュを再利用するためインスタンスメソッドとする
    /// L-002 監査対応: ConditionEvaluator::evaluate が async になったため async fn にする
    async fn evaluate_rules(
        &self,
        rules: &[Rule],
        mode: &EvaluationMode,
        input: &serde_json::Value,
        default_result: &serde_json::Value,
    ) -> Result<(Vec<MatchedRuleInfo>, serde_json::Value, bool), EvaluateError> {
        let mut matched = Vec::new();

        for rule in rules {
            let condition = ConditionParser::parse(&rule.when_condition)
                .map_err(EvaluateError::EvaluationError)?;

            // L-002 監査対応: tokio::sync::Mutex の .lock().await が内部で呼ばれるため .await が必要
            let is_match = self
                .condition_evaluator
                .evaluate(&condition, input)
                .await
                .map_err(EvaluateError::EvaluationError)?;

            if is_match {
                matched.push(MatchedRuleInfo {
                    id: rule.id,
                    name: rule.name.clone(),
                    priority: rule.priority,
                    result: rule.then_result.clone(),
                });

                if *mode == EvaluationMode::FirstMatch {
                    break;
                }
            }
        }

        if matched.is_empty() {
            Ok((matched, default_result.clone(), true))
        } else if *mode == EvaluationMode::FirstMatch {
            let result = matched[0].result.clone();
            Ok((matched, result, false))
        } else {
            // AllMatch: 全マッチ結果を配列にマージして返す
            let results: Vec<serde_json::Value> =
                matched.iter().map(|m| m.result.clone()).collect();
            Ok((matched, serde_json::Value::Array(results), false))
        }
    }

    fn hash_input(input: &serde_json::Value) -> String {
        use sha2::{Digest, Sha256};
        let bytes = serde_json::to_vec(input).unwrap_or_default();
        let hash = Sha256::digest(&bytes);
        format!("sha256:{}", hex::encode(hash))
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        super::hex_encode(bytes.as_ref())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::rule::{EvaluationMode, Rule, RuleSet};
    use crate::domain::repository::{
        evaluation_log_repository::MockEvaluationLogRepository,
        rule_repository::MockRuleRepository, rule_set_repository::MockRuleSetRepository,
    };

    fn make_rule_set(mode: EvaluationMode) -> RuleSet {
        let mut rs = RuleSet::new(
            "system".to_string(),
            "discount".to_string(),
            "Discount rules".to_string(),
            "sales".to_string(),
            mode,
            serde_json::json!({"discount": 0}),
            vec![],
        );
        rs.current_version = 1;
        rs
    }

    fn make_rule(
        name: &str,
        priority: i32,
        condition: serde_json::Value,
        result: serde_json::Value,
    ) -> Rule {
        Rule::new(
            "system".to_string(),
            name.to_string(),
            "desc".to_string(),
            priority,
            condition,
            result,
        )
    }

    fn make_uc(
        rule_set_repo: MockRuleSetRepository,
        rule_repo: MockRuleRepository,
        log_repo: MockEvaluationLogRepository,
    ) -> EvaluateUseCase {
        EvaluateUseCase::new(
            Arc::new(rule_set_repo),
            Arc::new(rule_repo),
            Arc::new(log_repo),
        )
    }

    /// "{domain}.{name}" 形式でないと EvaluationError を返す
    #[tokio::test]
    async fn invalid_rule_set_ref_format() {
        let uc = make_uc(
            MockRuleSetRepository::new(),
            MockRuleRepository::new(),
            MockEvaluationLogRepository::new(),
        );
        let result = uc
            .execute(&EvaluateInput {
                tenant_id: "system".to_string(),
                rule_set: "no-dot-here".to_string(),
                input: serde_json::json!({}),
                context: serde_json::json!({}),
                dry_run: true,
            })
            .await;
        assert!(matches!(result, Err(EvaluateError::EvaluationError(_))));
    }

    /// RuleSet が存在しない場合は RuleSetNotFound を返す
    #[tokio::test]
    async fn rule_set_not_found() {
        let mut rs_mock = MockRuleSetRepository::new();
        rs_mock
            .expect_find_by_domain_and_name()
            .returning(|_, _| Ok(None));
        let uc = make_uc(
            rs_mock,
            MockRuleRepository::new(),
            MockEvaluationLogRepository::new(),
        );
        let result = uc
            .execute(&EvaluateInput {
                tenant_id: "system".to_string(),
                rule_set: "sales.discount".to_string(),
                input: serde_json::json!({}),
                context: serde_json::json!({}),
                dry_run: true,
            })
            .await;
        assert!(matches!(result, Err(EvaluateError::RuleSetNotFound(_))));
    }

    /// FirstMatch モードで最初にマッチしたルール結果を返す
    #[tokio::test]
    async fn first_match_returns_first_rule_result() {
        let rule = make_rule(
            "high-value",
            1,
            serde_json::json!({"field": "amount", "operator": "gte", "value": 100}),
            serde_json::json!({"discount": 20}),
        );
        let rule_id = rule.id;
        let mut rs = make_rule_set(EvaluationMode::FirstMatch);
        rs.rule_ids = vec![rule_id];

        let mut rs_mock = MockRuleSetRepository::new();
        rs_mock
            .expect_find_by_domain_and_name()
            .returning(move |_, _| Ok(Some(rs.clone())));

        let mut r_mock = MockRuleRepository::new();
        r_mock
            .expect_find_by_ids()
            .returning(move |_| Ok(vec![rule.clone()]));

        let mut log_mock = MockEvaluationLogRepository::new();
        log_mock.expect_create().returning(|_| Ok(()));

        let uc = make_uc(rs_mock, r_mock, log_mock);
        let output = uc
            .execute(&EvaluateInput {
                tenant_id: "system".to_string(),
                rule_set: "sales.discount".to_string(),
                input: serde_json::json!({"amount": 150}),
                context: serde_json::json!({}),
                dry_run: false,
            })
            .await
            .unwrap();

        assert_eq!(output.matched_rules.len(), 1);
        assert_eq!(output.result, serde_json::json!({"discount": 20}));
        assert!(!output.default_applied);
    }

    /// マッチするルールがない場合はデフォルト結果を返す
    #[tokio::test]
    async fn no_match_returns_default_result() {
        let rule = make_rule(
            "high-value",
            1,
            serde_json::json!({"field": "amount", "operator": "gte", "value": 100}),
            serde_json::json!({"discount": 20}),
        );
        let rule_id = rule.id;
        let mut rs = make_rule_set(EvaluationMode::FirstMatch);
        rs.rule_ids = vec![rule_id];

        let mut rs_mock = MockRuleSetRepository::new();
        rs_mock
            .expect_find_by_domain_and_name()
            .returning(move |_, _| Ok(Some(rs.clone())));

        let mut r_mock = MockRuleRepository::new();
        r_mock
            .expect_find_by_ids()
            .returning(move |_| Ok(vec![rule.clone()]));

        let uc = make_uc(rs_mock, r_mock, MockEvaluationLogRepository::new());
        let output = uc
            .execute(&EvaluateInput {
                tenant_id: "system".to_string(),
                rule_set: "sales.discount".to_string(),
                input: serde_json::json!({"amount": 50}), // 100未満なのでマッチしない
                context: serde_json::json!({}),
                dry_run: true,
            })
            .await
            .unwrap();

        assert!(output.matched_rules.is_empty());
        assert_eq!(output.result, serde_json::json!({"discount": 0}));
        assert!(output.default_applied);
    }
}
