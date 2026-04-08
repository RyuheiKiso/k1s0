// サービスカタログのサービスエンティティ。サービスの重要度・ライフサイクルを定義する。
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 文字列パースエラー型（thiserror ベースで型安全なエラー分類を実現する）
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

/// Service はサービスカタログに登録されたサービスを表す。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Service {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub team_id: Uuid,
    pub tier: ServiceTier,
    pub lifecycle: ServiceLifecycle,
    pub repository_url: Option<String>,
    pub api_endpoint: Option<String>,
    pub healthcheck_url: Option<String>,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `ServiceTier` はサービスの重要度レベルを表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ServiceTier {
    Critical,
    Standard,
    Internal,
}

impl std::fmt::Display for ServiceTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceTier::Critical => write!(f, "critical"),
            ServiceTier::Standard => write!(f, "standard"),
            ServiceTier::Internal => write!(f, "internal"),
        }
    }
}

// ServiceTier の文字列パース実装（型安全な ParseError を使用する）
impl std::str::FromStr for ServiceTier {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "critical" => Ok(ServiceTier::Critical),
            "standard" => Ok(ServiceTier::Standard),
            "internal" => Ok(ServiceTier::Internal),
            _ => Err(ParseError::InvalidValue(format!(
                "invalid service tier: {s}"
            ))),
        }
    }
}

/// `ServiceLifecycle` はサービスのライフサイクルステージを表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ServiceLifecycle {
    Development,
    Staging,
    Production,
    Deprecated,
    Decommissioned,
}

impl std::fmt::Display for ServiceLifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceLifecycle::Development => write!(f, "development"),
            ServiceLifecycle::Staging => write!(f, "staging"),
            ServiceLifecycle::Production => write!(f, "production"),
            ServiceLifecycle::Deprecated => write!(f, "deprecated"),
            ServiceLifecycle::Decommissioned => write!(f, "decommissioned"),
        }
    }
}

// ServiceLifecycle の文字列パース実装（型安全な ParseError を使用する）
impl std::str::FromStr for ServiceLifecycle {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" => Ok(ServiceLifecycle::Development),
            "staging" => Ok(ServiceLifecycle::Staging),
            "production" => Ok(ServiceLifecycle::Production),
            "deprecated" => Ok(ServiceLifecycle::Deprecated),
            "decommissioned" => Ok(ServiceLifecycle::Decommissioned),
            _ => Err(ParseError::InvalidValue(format!(
                "invalid service lifecycle: {s}"
            ))),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_service_tier_display_and_parse() {
        assert_eq!(ServiceTier::Critical.to_string(), "critical");
        assert_eq!(ServiceTier::Standard.to_string(), "standard");
        assert_eq!(ServiceTier::Internal.to_string(), "internal");

        assert_eq!(
            "critical".parse::<ServiceTier>().unwrap(),
            ServiceTier::Critical
        );
        assert_eq!(
            "STANDARD".parse::<ServiceTier>().unwrap(),
            ServiceTier::Standard
        );
        assert!("unknown".parse::<ServiceTier>().is_err());
    }

    #[test]
    fn test_service_lifecycle_display_and_parse() {
        assert_eq!(ServiceLifecycle::Production.to_string(), "production");
        assert_eq!(ServiceLifecycle::Deprecated.to_string(), "deprecated");

        assert_eq!(
            "production".parse::<ServiceLifecycle>().unwrap(),
            ServiceLifecycle::Production
        );
        assert_eq!(
            "STAGING".parse::<ServiceLifecycle>().unwrap(),
            ServiceLifecycle::Staging
        );
        assert!("unknown".parse::<ServiceLifecycle>().is_err());
    }
}
