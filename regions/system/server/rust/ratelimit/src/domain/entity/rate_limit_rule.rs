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
}

impl Algorithm {
    pub fn as_str(&self) -> &str {
        match self {
            Algorithm::TokenBucket => "token_bucket",
            Algorithm::FixedWindow => "fixed_window",
            Algorithm::SlidingWindow => "sliding_window",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "token_bucket" => Ok(Algorithm::TokenBucket),
            "fixed_window" => Ok(Algorithm::FixedWindow),
            "sliding_window" => Ok(Algorithm::SlidingWindow),
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
    pub key: String,
    pub limit: i64,
    pub window_secs: i64,
    pub algorithm: Algorithm,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RateLimitRule {
    pub fn new(name: String, key: String, limit: i64, window_secs: i64, algorithm: Algorithm) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            key,
            limit,
            window_secs,
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
    pub remaining: i64,
    pub reset_at: i64,
    pub reason: String,
}

impl RateLimitDecision {
    pub fn allowed(remaining: i64, reset_at: i64) -> Self {
        Self {
            allowed: true,
            remaining,
            reset_at,
            reason: String::new(),
        }
    }

    pub fn denied(remaining: i64, reset_at: i64, reason: String) -> Self {
        Self {
            allowed: false,
            remaining,
            reset_at,
            reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_from_str() {
        assert_eq!(Algorithm::from_str("token_bucket").unwrap(), Algorithm::TokenBucket);
        assert_eq!(Algorithm::from_str("fixed_window").unwrap(), Algorithm::FixedWindow);
        assert_eq!(Algorithm::from_str("sliding_window").unwrap(), Algorithm::SlidingWindow);
        assert!(Algorithm::from_str("unknown").is_err());
    }

    #[test]
    fn test_algorithm_as_str() {
        assert_eq!(Algorithm::TokenBucket.as_str(), "token_bucket");
        assert_eq!(Algorithm::FixedWindow.as_str(), "fixed_window");
        assert_eq!(Algorithm::SlidingWindow.as_str(), "sliding_window");
    }

    #[test]
    fn test_rate_limit_rule_new() {
        let rule = RateLimitRule::new(
            "api-global".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        );
        assert_eq!(rule.name, "api-global");
        assert_eq!(rule.key, "global");
        assert_eq!(rule.limit, 100);
        assert_eq!(rule.window_secs, 60);
        assert_eq!(rule.algorithm, Algorithm::TokenBucket);
        assert!(rule.enabled);
    }

    #[test]
    fn test_rate_limit_decision_allowed() {
        let decision = RateLimitDecision::allowed(99, 1700000000);
        assert!(decision.allowed);
        assert_eq!(decision.remaining, 99);
        assert_eq!(decision.reset_at, 1700000000);
        assert!(decision.reason.is_empty());
    }

    #[test]
    fn test_rate_limit_decision_denied() {
        let decision = RateLimitDecision::denied(0, 1700000060, "rate limit exceeded".to_string());
        assert!(!decision.allowed);
        assert_eq!(decision.remaining, 0);
        assert_eq!(decision.reason, "rate limit exceeded");
    }
}
