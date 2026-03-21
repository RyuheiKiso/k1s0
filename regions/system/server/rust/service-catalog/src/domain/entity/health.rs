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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// HealthState の Display が小文字文字列を返す
    #[test]
    fn display_lowercase() {
        assert_eq!(HealthState::Healthy.to_string(), "healthy");
        assert_eq!(HealthState::Degraded.to_string(), "degraded");
        assert_eq!(HealthState::Unhealthy.to_string(), "unhealthy");
        assert_eq!(HealthState::Unknown.to_string(), "unknown");
    }

    /// serde で小文字にシリアライズ・デシリアライズできる
    #[test]
    fn serde_roundtrip() {
        let state = HealthState::Degraded;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"degraded\"");
        let decoded: HealthState = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, HealthState::Degraded);
    }
}
