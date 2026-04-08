use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// `ServiceDoc` はサービスに関連するドキュメントを表す。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceDoc {
    pub id: Uuid,
    pub service_id: Uuid,
    pub title: String,
    pub url: String,
    pub doc_type: DocType,
    pub created_at: DateTime<Utc>,
}

/// `DocType` はドキュメントの種類を表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DocType {
    Runbook,
    ApiSpec,
    Architecture,
    UserGuide,
    Other,
}

impl std::fmt::Display for DocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocType::Runbook => write!(f, "runbook"),
            DocType::ApiSpec => write!(f, "apispec"),
            DocType::Architecture => write!(f, "architecture"),
            DocType::UserGuide => write!(f, "userguide"),
            DocType::Other => write!(f, "other"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// DocType の Display が小文字文字列を返す
    #[test]
    fn display_lowercase() {
        assert_eq!(DocType::Runbook.to_string(), "runbook");
        assert_eq!(DocType::ApiSpec.to_string(), "apispec");
        assert_eq!(DocType::Architecture.to_string(), "architecture");
        assert_eq!(DocType::UserGuide.to_string(), "userguide");
        assert_eq!(DocType::Other.to_string(), "other");
    }

    /// serde で小文字にシリアライズ・デシリアライズできる
    #[test]
    fn serde_roundtrip() {
        let dt = DocType::ApiSpec;
        let json = serde_json::to_string(&dt).unwrap();
        assert_eq!(json, "\"apispec\"");
        let decoded: DocType = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, DocType::ApiSpec);
    }
}
