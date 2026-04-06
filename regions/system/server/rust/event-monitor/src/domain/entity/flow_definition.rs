use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub event_type: String,
    pub source: String,
    #[serde(default)]
    pub source_filter: Option<String>,
    pub timeout_seconds: i32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSlo {
    pub target_completion_seconds: i32,
    pub target_success_rate: f64,
    pub alert_on_violation: bool,
}

/// フロー定義ドメインエンティティ。テナント分離のため tenant_id を保持する。
#[derive(Debug, Clone)]
pub struct FlowDefinition {
    pub id: Uuid,
    /// テナント識別子（RLS による行レベルセキュリティのキーとなる）
    pub tenant_id: String,
    pub name: String,
    pub description: String,
    pub domain: String,
    pub steps: Vec<FlowStep>,
    pub slo: FlowSlo,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FlowDefinition {
    /// 新しいフロー定義を生成する。tenant_id はシステム管理者が管理するため通常 "system" を使用する。
    pub fn new(
        tenant_id: String,
        name: String,
        description: String,
        domain: String,
        steps: Vec<FlowStep>,
        slo: FlowSlo,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            name,
            description,
            domain,
            steps,
            slo,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}
