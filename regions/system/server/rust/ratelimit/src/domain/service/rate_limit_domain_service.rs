use crate::domain::entity::{Algorithm, RateLimitDecision, RateLimitRule};

pub struct RateLimitDomainService;

impl RateLimitDomainService {
    pub fn validate_scope(scope: &str) -> Result<(), String> {
        if scope.is_empty() {
            return Err("scope is required".to_string());
        }
        Ok(())
    }

    pub fn validate_identifier(identifier: &str) -> Result<(), String> {
        if identifier.is_empty() {
            return Err("identifier is required".to_string());
        }
        Ok(())
    }

    pub fn validate_rule_input(
        scope: &str,
        identifier_pattern: &str,
        limit: u32,
        window_seconds: u32,
    ) -> Result<(), String> {
        if scope.is_empty() {
            return Err("scope is required".to_string());
        }
        if identifier_pattern.is_empty() {
            return Err("identifier_pattern is required".to_string());
        }
        if limit == 0 {
            return Err("limit must be positive".to_string());
        }
        if window_seconds == 0 {
            return Err("window_seconds must be positive".to_string());
        }
        Ok(())
    }

    pub fn effective_limit_and_window(
        matched_rule: Option<&RateLimitRule>,
        default_limit: u32,
        default_window_seconds: u32,
        requested_window_seconds: i64,
    ) -> (u32, u32) {
        matched_rule
            .map(|rule| (rule.limit, rule.window_seconds))
            .unwrap_or((
                default_limit,
                if requested_window_seconds > 0 {
                    requested_window_seconds as u32
                } else {
                    default_window_seconds
                },
            ))
    }

    pub fn resolve_algorithm(matched_rule: Option<&RateLimitRule>) -> Algorithm {
        matched_rule
            .map(|rule| rule.algorithm.clone())
            .unwrap_or(Algorithm::TokenBucket)
    }

    pub fn fail_open_decision(
        scope: &str,
        identifier: &str,
        limit: u32,
        window_seconds: u32,
        rule_id: Option<String>,
    ) -> RateLimitDecision {
        let now = chrono::Utc::now().timestamp();
        RateLimitDecision {
            allowed: true,
            limit: i64::from(limit),
            remaining: i64::from(limit),
            reset_at: now + i64::from(window_seconds),
            reason: "fail-open: backend unavailable".to_string(),
            scope: scope.to_string(),
            identifier: identifier.to_string(),
            used: 0,
            rule_id: rule_id.unwrap_or_default(),
        }
    }
}
