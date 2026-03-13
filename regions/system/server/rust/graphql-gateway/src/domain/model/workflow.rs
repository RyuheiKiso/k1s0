use async_graphql::SimpleObject;

/// ワークフローステップ定義
#[derive(Debug, Clone, SimpleObject)]
pub struct WorkflowStep {
    pub step_id: String,
    pub name: String,
    pub step_type: String,
    pub assignee_role: Option<String>,
    pub timeout_hours: Option<i32>,
    pub on_approve: Option<String>,
    pub on_reject: Option<String>,
}

/// ワークフロー定義
#[derive(Debug, Clone, SimpleObject)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: i32,
    pub enabled: bool,
    pub steps: Vec<WorkflowStep>,
    pub created_at: String,
    pub updated_at: String,
}

/// ワークフローインスタンス
#[derive(Debug, Clone, SimpleObject)]
pub struct WorkflowInstance {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub title: String,
    pub initiator_id: String,
    pub current_step_id: Option<String>,
    pub status: String,
    pub context_json: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
}

/// ワークフロータスク
#[derive(Debug, Clone, SimpleObject)]
pub struct WorkflowTask {
    pub id: String,
    pub instance_id: String,
    pub step_id: String,
    pub step_name: String,
    pub assignee_id: Option<String>,
    pub status: String,
    pub due_at: Option<String>,
    pub comment: Option<String>,
    pub actor_id: Option<String>,
    pub decided_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
