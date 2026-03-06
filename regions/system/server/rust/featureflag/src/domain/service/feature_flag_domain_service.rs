use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::domain::entity::evaluation::EvaluationContext;
use crate::domain::entity::feature_flag::{FeatureFlag, FlagRule, FlagVariant};

pub struct FeatureFlagDomainService;

impl FeatureFlagDomainService {
    pub fn validate_flag_key(flag_key: &str) -> Result<(), String> {
        let key = flag_key.trim();
        if key.is_empty() {
            return Err("flag_key is required".to_string());
        }
        if key.len() > 128 {
            return Err("flag_key must be 128 characters or fewer".to_string());
        }
        Ok(())
    }

    pub fn validate_variants(variants: &[FlagVariant]) -> Result<(), String> {
        if variants.is_empty() {
            return Ok(());
        }
        if variants.iter().any(|v| v.weight < 0) {
            return Err("variant weight must be non-negative".to_string());
        }
        let total_weight: i32 = variants.iter().map(|v| v.weight).sum();
        if total_weight <= 0 {
            return Err("variant weight total must be greater than 0".to_string());
        }
        Ok(())
    }

    pub fn evaluate(
        flag: &FeatureFlag,
        context: &EvaluationContext,
    ) -> (bool, Option<String>, String) {
        if !flag.enabled {
            return (false, None, "flag is disabled".to_string());
        }

        if let Some(rule_variant) = Self::match_rule_variant(&flag.rules, &flag.variants, context) {
            return (true, Some(rule_variant), "flag is enabled".to_string());
        }

        let variant = Self::select_weighted_variant(&flag.variants, context);
        (true, variant, "flag is enabled".to_string())
    }

    fn match_rule_variant(
        rules: &[FlagRule],
        variants: &[FlagVariant],
        context: &EvaluationContext,
    ) -> Option<String> {
        for rule in rules {
            let Some(actual_value) = Self::resolve_attribute(context, &rule.attribute) else {
                continue;
            };
            if Self::matches(rule, actual_value) && variants.iter().any(|v| v.name == rule.variant)
            {
                return Some(rule.variant.clone());
            }
        }
        None
    }

    fn resolve_attribute<'a>(context: &'a EvaluationContext, attribute: &str) -> Option<&'a str> {
        match attribute {
            "user_id" => context.user_id.as_deref(),
            "tenant_id" => context.tenant_id.as_deref(),
            key => context.attributes.get(key).map(|v| v.as_str()),
        }
    }

    fn matches(rule: &FlagRule, actual_value: &str) -> bool {
        match rule.operator.as_str() {
            "eq" => actual_value == rule.value,
            "contains" => actual_value.contains(&rule.value),
            "in" => rule
                .value
                .split(',')
                .map(|s| s.trim())
                .any(|expected| !expected.is_empty() && expected == actual_value),
            _ => false,
        }
    }

    fn select_weighted_variant(
        variants: &[FlagVariant],
        context: &EvaluationContext,
    ) -> Option<String> {
        let weighted: Vec<&FlagVariant> = variants.iter().filter(|v| v.weight > 0).collect();
        if weighted.is_empty() {
            return variants.first().map(|v| v.name.clone());
        }

        let total_weight: i64 = weighted.iter().map(|v| i64::from(v.weight)).sum();
        if total_weight <= 0 {
            return variants.first().map(|v| v.name.clone());
        }

        let seed = context
            .user_id
            .as_deref()
            .or(context.tenant_id.as_deref())
            .unwrap_or("anonymous");
        let bucket = (Self::stable_hash(seed) % (total_weight as u64)) as i64;

        let mut cumulative = 0i64;
        for variant in weighted {
            cumulative += i64::from(variant.weight);
            if bucket < cumulative {
                return Some(variant.name.clone());
            }
        }

        variants.first().map(|v| v.name.clone())
    }

    fn stable_hash(value: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}
