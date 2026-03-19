use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// レートリミットのアルゴリズム。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Algorithm {
    TokenBucket,
    FixedWindow,
    SlidingWindow,
    LeakyBucket,
}

impl Algorithm {
    pub fn as_str(&self) -> &str {
        match self {
            Algorithm::TokenBucket => "token_bucket",
            Algorithm::FixedWindow => "fixed_window",
            Algorithm::SlidingWindow => "sliding_window",
            Algorithm::LeakyBucket => "leaky_bucket",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "token_bucket" => Ok(Algorithm::TokenBucket),
            "fixed_window" => Ok(Algorithm::FixedWindow),
            "sliding_window" => Ok(Algorithm::SlidingWindow),
            "leaky_bucket" => Ok(Algorithm::LeakyBucket),
            _ => Err(format!("unknown algorithm: {}", s)),
        }
    }
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// レートリミットルール。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRule {
    pub id: Uuid,
    pub name: String,
    pub scope: String,
    pub identifier_pattern: String,
    pub limit: u32,
    pub window_seconds: u32,
    pub algorithm: Algorithm,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RateLimitRule {
    pub fn new(
        scope: String,
        identifier_pattern: String,
        limit: u32,
        window_seconds: u32,
        algorithm: Algorithm,
    ) -> Self {
        let now = Utc::now();
        let name = format!("{}:{}", scope, identifier_pattern);
        Self {
            id: Uuid::new_v4(),
            name,
            scope,
            identifier_pattern,
            limit,
            window_seconds,
            algorithm,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// レートリミットチェックの結果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitDecision {
    pub allowed: bool,
    pub scope: String,
    pub identifier: String,
    pub limit: i64,
    pub used: i64,
    pub remaining: i64,
    pub reset_at: DateTime<Utc>,
    pub rule_id: String,
    pub reason: String,
}

impl RateLimitDecision {
    pub fn allowed(limit: i64, remaining: i64, reset_at: DateTime<Utc>) -> Self {
        Self {
            allowed: true,
            scope: String::new(),
            identifier: String::new(),
            limit,
            used: (limit - remaining).max(0),
            remaining,
            reset_at,
            rule_id: String::new(),
            reason: String::new(),
        }
    }

    pub fn denied(limit: i64, remaining: i64, reset_at: DateTime<Utc>, reason: String) -> Self {
        Self {
            allowed: false,
            scope: String::new(),
            identifier: String::new(),
            limit,
            used: (limit - remaining).max(0),
            remaining,
            reset_at,
            rule_id: String::new(),
            reason,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_algorithm_from_str() {
        assert_eq!(
            Algorithm::from_str("token_bucket").unwrap(),
            Algorithm::TokenBucket
        );
        assert_eq!(
            Algorithm::from_str("fixed_window").unwrap(),
            Algorithm::FixedWindow
        );
        assert_eq!(
            Algorithm::from_str("sliding_window").unwrap(),
            Algorithm::SlidingWindow
        );
        assert_eq!(
            Algorithm::from_str("leaky_bucket").unwrap(),
            Algorithm::LeakyBucket
        );
        assert!(Algorithm::from_str("unknown").is_err());
    }

    #[test]
    fn test_algorithm_as_str() {
        assert_eq!(Algorithm::TokenBucket.as_str(), "token_bucket");
        assert_eq!(Algorithm::FixedWindow.as_str(), "fixed_window");
        assert_eq!(Algorithm::SlidingWindow.as_str(), "sliding_window");
        assert_eq!(Algorithm::LeakyBucket.as_str(), "leaky_bucket");
    }

    #[test]
    fn test_rate_limit_rule_new() {
        let rule = RateLimitRule::new(
            "service".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        );
        assert_eq!(rule.scope, "service");
        assert_eq!(rule.name, "service:global");
        assert_eq!(rule.identifier_pattern, "global");
        assert_eq!(rule.limit, 100);
        assert_eq!(rule.window_seconds, 60);
        assert_eq!(rule.algorithm, Algorithm::TokenBucket);
        assert!(rule.enabled);
    }

    #[test]
    fn test_rate_limit_decision_allowed() {
        let reset_at = Utc.timestamp_opt(1700000000, 0).single().unwrap();
        let decision = RateLimitDecision::allowed(100, 99, reset_at);
        assert!(decision.allowed);
        assert_eq!(decision.limit, 100);
        assert_eq!(decision.used, 1);
        assert_eq!(decision.remaining, 99);
        assert_eq!(decision.reset_at.timestamp(), 1700000000);
        assert!(decision.reason.is_empty());
    }

    #[test]
    fn test_rate_limit_decision_denied() {
        let reset_at = Utc.timestamp_opt(1700000060, 0).single().unwrap();
        let decision =
            RateLimitDecision::denied(100, 0, reset_at, "rate limit exceeded".to_string());
        assert!(!decision.allowed);
        assert_eq!(decision.limit, 100);
        assert_eq!(decision.used, 100);
        assert_eq!(decision.remaining, 0);
        assert_eq!(decision.reason, "rate limit exceeded");
    }
}
