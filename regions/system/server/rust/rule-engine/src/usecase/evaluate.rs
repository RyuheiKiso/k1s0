use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::rule::{EvaluationLog, EvaluationMode, Rule};
use crate::domain::repository::{EvaluationLogRepository, RuleRepository, RuleSetRepository};
use crate::domain::service::condition_evaluator::ConditionEvaluator;
use crate::domain::service::condition_parser::ConditionParser;

#[derive(Debug, Clone)]
pub struct EvaluateInput {
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

        let (matched_rules, result, default_applied) = Self::evaluate_rules(
            &rules,
            &rule_set.evaluation_mode,
            &input.input,
            &rule_set.default_result,
        )?;

        // Log evaluation (unless dry_run)
        if !input.dry_run {
            let input_hash = Self::hash_input(&input.input);
            let log = EvaluationLog {
                id: evaluation_id,
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

    fn evaluate_rules(
        rules: &[Rule],
        mode: &EvaluationMode,
        input: &serde_json::Value,
        default_result: &serde_json::Value,
    ) -> Result<(Vec<MatchedRuleInfo>, serde_json::Value, bool), EvaluateError> {
        let mut matched = Vec::new();

        for rule in rules {
            let condition = ConditionParser::parse(&rule.when_condition)
                .map_err(EvaluateError::EvaluationError)?;

            let is_match = ConditionEvaluator::evaluate(&condition, input)
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
            // AllMatch: merge results into array
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
