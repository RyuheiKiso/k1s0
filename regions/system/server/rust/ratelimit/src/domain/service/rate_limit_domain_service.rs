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

    /// 有効なリミットとウィンドウ秒数を決定する。
    /// LOW-11 対応: ルールが未設定の場合、外部リクエストの `window_secs` を使用せずに
    /// サーバー側の `default_window_seconds` を採用する。
    /// これにより、クライアントが任意のウィンドウを指定してレートリミットを迂回することを防ぐ。
    /// ルールが設定されている場合は、ルール側の `window_seconds` が常に優先される。
    #[must_use] 
    pub fn effective_limit_and_window(
        matched_rule: Option<&RateLimitRule>,
        default_limit: u32,
        default_window_seconds: u32,
        _requested_window_seconds: i64,
    ) -> (u32, u32) {
        matched_rule
            .map_or((default_limit, default_window_seconds), |rule| (rule.limit, rule.window_seconds))
    }

    #[must_use] 
    pub fn resolve_algorithm(matched_rule: Option<&RateLimitRule>) -> Algorithm {
        matched_rule
            .map_or(Algorithm::TokenBucket, |rule| rule.algorithm.clone())
    }

    #[must_use] 
    pub fn fail_open_decision(
        scope: &str,
        identifier: &str,
        limit: u32,
        window_seconds: u32,
        rule_id: Option<String>,
    ) -> RateLimitDecision {
        let now = chrono::Utc::now();
        RateLimitDecision {
            allowed: true,
            limit: i64::from(limit),
            remaining: i64::from(limit),
            reset_at: now + chrono::Duration::seconds(i64::from(window_seconds)),
            reason: "fail-open: backend unavailable".to_string(),
            scope: scope.to_string(),
            identifier: identifier.to_string(),
            used: 0,
            rule_id: rule_id.unwrap_or_default(),
        }
    }
}
