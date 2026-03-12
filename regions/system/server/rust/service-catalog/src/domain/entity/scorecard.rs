use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Scorecard はサービスの品質スコアカードを表す。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Scorecard {
    pub service_id: Uuid,
    pub documentation_score: f64,
    pub test_coverage_score: f64,
    pub slo_compliance_score: f64,
    pub security_score: f64,
    pub overall_score: f64,
    pub evaluated_at: DateTime<Utc>,
}
