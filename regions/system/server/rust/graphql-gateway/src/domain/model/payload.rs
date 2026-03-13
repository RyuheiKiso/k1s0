use async_graphql::SimpleObject;

use super::{
    AuditLog, CatalogService, FeatureFlag, Job, JobExecution, NotificationChannel, NotificationLog,
    NotificationTemplate, Session, Tenant, WorkflowDefinition, WorkflowInstance, WorkflowTask,
};

/// GraphQL UserError: フィールドレベルエラーの構造化表現
#[derive(Debug, Clone, SimpleObject)]
pub struct UserError {
    pub field: Option<Vec<String>>,
    pub message: String,
}

// --- Tenant ---

#[derive(Debug, Clone, SimpleObject)]
pub struct CreateTenantPayload {
    pub tenant: Option<Tenant>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct UpdateTenantPayload {
    pub tenant: Option<Tenant>,
    pub errors: Vec<UserError>,
}

// --- FeatureFlag ---

#[derive(Debug, Clone, SimpleObject)]
pub struct SetFeatureFlagPayload {
    pub feature_flag: Option<FeatureFlag>,
    pub errors: Vec<UserError>,
}

// --- ServiceCatalog ---

#[derive(Debug, Clone, SimpleObject)]
pub struct RegisterServicePayload {
    pub service: Option<CatalogService>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct UpdateServicePayload {
    pub service: Option<CatalogService>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct DeleteServicePayload {
    pub success: bool,
    pub errors: Vec<UserError>,
}

// --- Auth ---

#[derive(Debug, Clone, SimpleObject)]
pub struct RecordAuditLogPayload {
    pub audit_log: Option<AuditLog>,
    pub errors: Vec<UserError>,
}

// --- Session ---

/// createSession は token を返す（list/get では省略）
#[derive(Debug, Clone, SimpleObject)]
pub struct CreateSessionPayload {
    pub session: Option<Session>,
    pub token: Option<String>,
    pub errors: Vec<UserError>,
}

/// refreshSession は新しい token を返す
#[derive(Debug, Clone, SimpleObject)]
pub struct RefreshSessionPayload {
    pub session: Option<Session>,
    pub token: Option<String>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct RevokeSessionPayload {
    pub success: bool,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct RevokeAllSessionsPayload {
    pub revoked_count: i32,
    pub errors: Vec<UserError>,
}

// --- Vault ---

#[derive(Debug, Clone, SimpleObject)]
pub struct SetSecretPayload {
    pub path: Option<String>,
    pub version: Option<i64>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct RotateSecretPayload {
    pub path: Option<String>,
    pub new_version: Option<i64>,
    pub rotated: bool,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct DeleteSecretPayload {
    pub success: bool,
    pub errors: Vec<UserError>,
}

// --- Scheduler ---

#[derive(Debug, Clone, SimpleObject)]
pub struct CreateJobPayload {
    pub job: Option<Job>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct UpdateJobPayload {
    pub job: Option<Job>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct DeleteJobPayload {
    pub success: bool,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct PauseJobPayload {
    pub job: Option<Job>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ResumeJobPayload {
    pub job: Option<Job>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct TriggerJobPayload {
    pub execution: Option<JobExecution>,
    pub errors: Vec<UserError>,
}

// --- Notification ---

#[derive(Debug, Clone, SimpleObject)]
pub struct SendNotificationPayload {
    pub notification_id: Option<String>,
    pub status: Option<String>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct RetryNotificationPayload {
    pub notification: Option<NotificationLog>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct CreateChannelPayload {
    pub channel: Option<NotificationChannel>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct UpdateChannelPayload {
    pub channel: Option<NotificationChannel>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct DeleteChannelPayload {
    pub success: bool,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct CreateTemplatePayload {
    pub template: Option<NotificationTemplate>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct UpdateTemplatePayload {
    pub template: Option<NotificationTemplate>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct DeleteTemplatePayload {
    pub success: bool,
    pub errors: Vec<UserError>,
}

// --- Workflow ---

#[derive(Debug, Clone, SimpleObject)]
pub struct CreateWorkflowPayload {
    pub workflow: Option<WorkflowDefinition>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct UpdateWorkflowPayload {
    pub workflow: Option<WorkflowDefinition>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct DeleteWorkflowPayload {
    pub success: bool,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct StartInstancePayload {
    pub instance: Option<WorkflowInstance>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct CancelInstancePayload {
    pub instance: Option<WorkflowInstance>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ReassignTaskPayload {
    pub task: Option<WorkflowTask>,
    pub previous_assignee_id: Option<String>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ApproveTaskPayload {
    pub task_id: Option<String>,
    pub status: Option<String>,
    pub next_task_id: Option<String>,
    pub instance_status: Option<String>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct RejectTaskPayload {
    pub task_id: Option<String>,
    pub status: Option<String>,
    pub next_task_id: Option<String>,
    pub instance_status: Option<String>,
    pub errors: Vec<UserError>,
}
