use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SubjectType {
    Tenant,
    User,
    ApiKey,
}

impl SubjectType {
    pub fn as_str(&self) -> &str {
        match self {
            SubjectType::Tenant => "tenant",
            SubjectType::User => "user",
            SubjectType::ApiKey => "api_key",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "tenant" => Some(SubjectType::Tenant),
            "user" => Some(SubjectType::User),
            "api_key" => Some(SubjectType::ApiKey),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Period {
    Daily,
    Monthly,
}

impl Period {
    pub fn as_str(&self) -> &str {
        match self {
            Period::Daily => "daily",
            Period::Monthly => "monthly",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "daily" => Some(Period::Daily),
            "monthly" => Some(Period::Monthly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaPolicy {
    pub id: String,
    pub name: String,
    pub subject_type: SubjectType,
    pub subject_id: String,
    pub limit: u64,
    pub period: Period,
    pub enabled: bool,
    pub alert_threshold_percent: Option<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl QuotaPolicy {
    pub fn new(
        name: String,
        subject_type: SubjectType,
        subject_id: String,
        limit: u64,
        period: Period,
        enabled: bool,
        alert_threshold_percent: Option<u8>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("quota_{}", uuid::Uuid::new_v4().simple()),
            name,
            subject_type,
            subject_id,
            limit,
            period,
            enabled,
            alert_threshold_percent,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaUsage {
    pub quota_id: String,
    pub subject_type: SubjectType,
    pub subject_id: String,
    pub period: Period,
    pub limit: u64,
    pub used: u64,
    pub remaining: u64,
    pub usage_percent: f64,
    pub exceeded: bool,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub reset_at: DateTime<Utc>,
}

impl QuotaUsage {
    pub fn new(policy: &QuotaPolicy, used: u64, period_start: DateTime<Utc>, period_end: DateTime<Utc>, reset_at: DateTime<Utc>) -> Self {
        let remaining = if used >= policy.limit { 0 } else { policy.limit - used };
        let usage_percent = if policy.limit == 0 {
            100.0
        } else {
            (used as f64 / policy.limit as f64) * 100.0
        };
        Self {
            quota_id: policy.id.clone(),
            subject_type: policy.subject_type.clone(),
            subject_id: policy.subject_id.clone(),
            period: policy.period.clone(),
            limit: policy.limit,
            used,
            remaining,
            usage_percent,
            exceeded: used >= policy.limit,
            period_start,
            period_end,
            reset_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementResult {
    pub quota_id: String,
    pub used: u64,
    pub remaining: u64,
    pub usage_percent: f64,
    pub exceeded: bool,
    pub allowed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_type_roundtrip() {
        assert_eq!(SubjectType::from_str("tenant"), Some(SubjectType::Tenant));
        assert_eq!(SubjectType::from_str("user"), Some(SubjectType::User));
        assert_eq!(SubjectType::from_str("api_key"), Some(SubjectType::ApiKey));
        assert_eq!(SubjectType::from_str("unknown"), None);
        assert_eq!(SubjectType::Tenant.as_str(), "tenant");
        assert_eq!(SubjectType::User.as_str(), "user");
        assert_eq!(SubjectType::ApiKey.as_str(), "api_key");
    }

    #[test]
    fn test_period_roundtrip() {
        assert_eq!(Period::from_str("daily"), Some(Period::Daily));
        assert_eq!(Period::from_str("monthly"), Some(Period::Monthly));
        assert_eq!(Period::from_str("weekly"), None);
        assert_eq!(Period::Daily.as_str(), "daily");
        assert_eq!(Period::Monthly.as_str(), "monthly");
    }

    #[test]
    fn test_quota_policy_new() {
        let policy = QuotaPolicy::new(
            "test-policy".to_string(),
            SubjectType::Tenant,
            "tenant-abc".to_string(),
            10000,
            Period::Daily,
            true,
            Some(80),
        );
        assert!(policy.id.starts_with("quota_"));
        assert_eq!(policy.name, "test-policy");
        assert_eq!(policy.subject_type, SubjectType::Tenant);
        assert_eq!(policy.subject_id, "tenant-abc");
        assert_eq!(policy.limit, 10000);
        assert_eq!(policy.period, Period::Daily);
        assert!(policy.enabled);
        assert_eq!(policy.alert_threshold_percent, Some(80));
    }

    #[test]
    fn test_quota_usage_new_under_limit() {
        let policy = QuotaPolicy::new(
            "test".to_string(),
            SubjectType::User,
            "user-1".to_string(),
            1000,
            Period::Daily,
            true,
            None,
        );
        let now = Utc::now();
        let usage = QuotaUsage::new(&policy, 500, now, now, now);
        assert_eq!(usage.used, 500);
        assert_eq!(usage.remaining, 500);
        assert!((usage.usage_percent - 50.0).abs() < f64::EPSILON);
        assert!(!usage.exceeded);
    }

    #[test]
    fn test_quota_usage_new_at_limit() {
        let policy = QuotaPolicy::new(
            "test".to_string(),
            SubjectType::Tenant,
            "tenant-1".to_string(),
            100,
            Period::Monthly,
            true,
            None,
        );
        let now = Utc::now();
        let usage = QuotaUsage::new(&policy, 100, now, now, now);
        assert_eq!(usage.remaining, 0);
        assert!((usage.usage_percent - 100.0).abs() < f64::EPSILON);
        assert!(usage.exceeded);
    }

    #[test]
    fn test_quota_usage_new_over_limit() {
        let policy = QuotaPolicy::new(
            "test".to_string(),
            SubjectType::ApiKey,
            "key-1".to_string(),
            50,
            Period::Daily,
            true,
            None,
        );
        let now = Utc::now();
        let usage = QuotaUsage::new(&policy, 60, now, now, now);
        assert_eq!(usage.remaining, 0);
        assert!(usage.exceeded);
    }

    #[test]
    fn test_quota_usage_zero_limit() {
        let policy = QuotaPolicy::new(
            "test".to_string(),
            SubjectType::Tenant,
            "tenant-1".to_string(),
            0,
            Period::Daily,
            true,
            None,
        );
        let now = Utc::now();
        let usage = QuotaUsage::new(&policy, 0, now, now, now);
        assert!((usage.usage_percent - 100.0).abs() < f64::EPSILON);
    }
}
