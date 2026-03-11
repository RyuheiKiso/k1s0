use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// HealthStatus はサービスのヘルスチェック結果を表す。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthStatus {
    pub service_id: Uuid,
    pub status: HealthState,
    pub message: Option<String>,
    pub response_time_ms: Option<i64>,
    pub checked_at: DateTime<Utc>,
}

/// HealthState はサービスのヘルス状態を表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl std::fmt::Display for HealthState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthState::Healthy => write!(f, "healthy"),
            HealthState::Degraded => write!(f, "degraded"),
            HealthState::Unhealthy => write!(f, "unhealthy"),
            HealthState::Unknown => write!(f, "unknown"),
        }
    }
}
