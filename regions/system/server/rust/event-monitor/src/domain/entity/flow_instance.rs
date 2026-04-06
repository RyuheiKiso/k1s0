use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum FlowInstanceStatus {
    InProgress,
    Completed,
    Failed,
    Timeout,
}

impl FlowInstanceStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Timeout => "timeout",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "timeout" => Self::Timeout,
            _ => Self::InProgress,
        }
    }
}

/// フローインスタンスドメインエンティティ。テナント分離のため tenant_id を保持する。
#[derive(Debug, Clone)]
pub struct FlowInstance {
    pub id: Uuid,
    /// テナント識別子（RLS による行レベルセキュリティのキーとなる）
    pub tenant_id: String,
    pub flow_id: Uuid,
    pub correlation_id: String,
    pub status: FlowInstanceStatus,
    pub current_step_index: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
}

impl FlowInstance {
    /// 新しいフローインスタンスを生成する。
    /// tenant_id はイベントと同じテナントに属するため、Kafka ヘッダーから伝播した値を使用する。
    pub fn new(tenant_id: String, flow_id: Uuid, correlation_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            flow_id,
            correlation_id,
            status: FlowInstanceStatus::InProgress,
            current_step_index: 0,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
        }
    }
}
