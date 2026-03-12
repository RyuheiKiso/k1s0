use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// ServiceDoc はサービスに関連するドキュメントを表す。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceDoc {
    pub id: Uuid,
    pub service_id: Uuid,
    pub title: String,
    pub url: String,
    pub doc_type: DocType,
    pub created_at: DateTime<Utc>,
}

/// DocType はドキュメントの種類を表す。
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
