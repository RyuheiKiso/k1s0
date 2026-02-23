use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuotaStatus {
    pub allowed: bool,
    pub remaining: u64,
    pub limit: u64,
    pub reset_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuotaUsage {
    pub quota_id: String,
    pub used: u64,
    pub limit: u64,
    pub period: QuotaPeriod,
    pub reset_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuotaPolicy {
    pub quota_id: String,
    pub limit: u64,
    pub period: QuotaPeriod,
    pub reset_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QuotaPeriod {
    Hourly,
    Daily,
    Monthly,
    Custom(u64),
}
